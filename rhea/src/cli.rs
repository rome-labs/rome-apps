use std::path::PathBuf;

use rome_sdk::rome_utils::config::ReadableConfig;

use crate::config::RheaConfig;

#[derive(clap::Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// Path to the config file
    #[clap(short = 'c', long)]
    pub config: Option<PathBuf>,
}

impl Cli {
    /// Get the path to the config file
    /// - from cli
    /// - from env
    /// - default
    pub fn get_config_path(&self) -> anyhow::Result<PathBuf> {
        self.config
            .clone()
            .or_else(|| std::env::var("RHEA_CONFIG").ok().map(PathBuf::from))
            .ok_or_else(|| anyhow::anyhow!("Config file path not found"))
    }

    pub async fn load_config(&self) -> anyhow::Result<RheaConfig> {
        RheaConfig::read(&self.get_config_path()?).await
    }
}
