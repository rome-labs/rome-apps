use rome_sdk::rome_evm_client::indexer;
use rome_sdk::rome_evm_client::resources::PayerConfig;
use rome_sdk::rome_solana::config::SolanaConfig;
use solana_sdk::clock::Slot;
use std::net::SocketAddr;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ProxyConfig {
    pub chain_id: u64,
    pub start_slot: Option<u64>,
    pub solana: SolanaConfig,
    pub program_id: String,
    pub payers: Vec<PayerConfig>,
    pub proxy_host: SocketAddr,
    pub max_slot_history: Option<Slot>,
    pub ethereum_storage: indexer::config::EthereumStorageConfig,
    pub block_loader_batch_size: Option<Slot>,
}
