mod api;
mod cli;
mod config;
mod proxy;

use self::cli::Cli;
use anyhow::anyhow;
use clap::Parser;
use rome_obs::Otel;
use tokio::signal;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // load any .env and init the logger
    dotenv::dotenv().ok();

    Otel::init_from_env("proxy").map_err(|e| anyhow!(e.to_string()))?;

    let config = Cli::parse().load_config().await?;
    let (_server, join_handle) = config.init().await?;

    tokio::select! {
        res = join_handle => {
            anyhow::bail!("RomeEVMClient exit.. {:?}", res)
        },
        _ = signal::ctrl_c() => {
            anyhow::bail!("Shutdown..")
        }
    }
}
