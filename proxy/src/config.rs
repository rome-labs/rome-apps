use std::net::SocketAddr;
use rome_sdk::rome_evm_client::resources::PayerConfig;
use rome_sdk::rome_solana::config::SolanaConfig;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ProxyConfig {
    pub chain_id: u64,
    pub start_slot: Option<u64>,
    pub solana: SolanaConfig,
    pub program_id: String,
    pub payers: Vec<PayerConfig>,
    pub proxy_host: SocketAddr,
}
