use self::cli::Cli;
use self::config::HerculesConfig;
use anyhow::{anyhow, bail};
use clap::Parser;
use dotenv::dotenv;
use rome_obs::Otel;
use tokio::signal;

mod api;
mod cli;
mod config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    Otel::init_from_env("hercules").map_err(|e| anyhow!(e.to_string()))?;

    let config: HerculesConfig = Cli::parse().load_config().await?;

    let (_admin_server, indexer_jh) = config.init().await?;

    // Shutdown on ctrl-c
    tokio::select! {
        res = indexer_jh => {
            bail!("Hercules Service Exited: {:?}", res);
        },
        res = signal::ctrl_c() => {
            bail!("Shutting down Hercules: {:?}", res);
        },
    }
}
