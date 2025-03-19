use self::cli::Cli;
use self::config::HerculesConfig;
use anyhow::bail;
use clap::Parser;
use dotenv::dotenv;
use tokio::signal;

mod api;
mod cli;
mod config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    tracing_subscriber::fmt().json().init();
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
