use clap::Parser;
use tokio::signal;

use self::cli::Cli;
use self::config::RheaConfig;
use self::service::RheaService;
use crate::rome_sender::RomeSender;
use anyhow::{anyhow, bail};
use dotenv::dotenv;
use rome_obs::Otel;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;
use std::time::Duration;

mod cli;
mod config;
mod mempool;
mod mempool_sender;
mod rome_sender;
mod service;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    Otel::init_from_env("rhea").map_err(|e| anyhow!(e.to_string()))?;

    let config: RheaConfig = Cli::parse().load_config().await?;
    let program_id = Pubkey::from_str(&config.program_id)?;

    let rhea_service_jh = RheaService::start(
        RomeSender::new(
            config.rpc_urls,
            config.commitment,
            &vec![(config.chain_id, program_id.to_string())]
                .into_iter()
                .collect(),
            &config.payers,
        )
        .await
        .unwrap(),
        config.geth_indexer,
        config.mempool_ttl.map(Duration::from_secs),
    );

    tokio::select! {
        res = rhea_service_jh => {
            bail!("Rhea Service Exited: {:?}", res);
        },
        res = signal::ctrl_c() => {
            bail!("Shutting down Rhea: {:?}", res);
        },
    }
}
