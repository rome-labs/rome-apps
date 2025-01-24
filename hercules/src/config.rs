use rome_sdk::rome_evm_client::indexer;
use rome_sdk::rome_geth::engine::config::GethEngineConfig;
use rome_sdk::rome_solana::config::SolanaConfig;
use solana_sdk::slot_history::Slot;
use std::net::SocketAddr;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct HerculesConfig {
    pub solana: SolanaConfig,
    pub program_id: String,
    pub chain_id: u64,
    pub start_slot: Option<u64>,
    pub geth_engine: GethEngineConfig,
    pub geth_api: String,
    pub solana_storage: indexer::pg_storage::config::PgPoolConfig,
    pub ethereum_storage: indexer::config::EthereumStorageConfig,
    pub admin_rpc: SocketAddr,
    pub max_slot_history: Option<Slot>,
    pub block_loader_batch_size: Option<Slot>,
}
