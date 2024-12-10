use rome_sdk::rome_evm_client::indexer::pg_storage;
use rome_sdk::rome_geth::engine::config::GethEngineConfig;
use rome_sdk::rome_solana::config::SolanaConfig;
use solana_sdk::slot_history::Slot;
use std::net::SocketAddr;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EthereumStorageConfig {
    PgStorage(pg_storage::PgPoolConfig),
    InMemoryStorage,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct HerculesConfig {
    pub solana: SolanaConfig,
    pub program_id: String,
    pub chain_id: u64,
    pub start_slot: Option<u64>,
    pub geth_engine: GethEngineConfig,
    pub solana_storage: pg_storage::PgPoolConfig,
    pub ethereum_storage: EthereumStorageConfig,
    pub admin_rpc: SocketAddr,
    pub max_slot_history: Option<Slot>,
}
