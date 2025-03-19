use clap::Parser;
use rome_sdk::Rome;
use tokio::signal;

use self::cli::Cli;
use self::config::RheaConfig;
use self::service::RheaService;
use anyhow::bail;
use dotenv::dotenv;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;
use std::time::Duration;

mod cli;
mod config;
mod mempool;
mod mempool_sender;
mod service;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    tracing_subscriber::fmt().json().init();

    let config: RheaConfig = Cli::parse().load_config().await?;
    let program_id = Pubkey::from_str(&config.program_id)?;

    let rome = Rome::new_with_config(rome_sdk::RomeConfig {
        solana_config: config.solana.clone(),
        rollups: vec![(config.chain_id, program_id.to_string())]
            .into_iter()
            .collect(),
        payers: config.payers,
    })
    .await?;

    let rhea_service_jh = RheaService::start(
        rome,
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
