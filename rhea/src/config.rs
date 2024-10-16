use ethers::types::Address;
use rome_sdk::rome_geth::engine::config::GethEngineConfig;
use rome_sdk::rome_geth::indexers::pending_txs::GethPendingTxsIndexer;
use rome_sdk::rome_solana::config::SolanaConfig;
use std::path::PathBuf;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct RheaConfig {
    // Same as proxy
    pub chain_id: u64,
    pub start_slot: Option<u64>,
    pub solana: SolanaConfig,
    pub program_keypair: PathBuf,
    pub payer_keypair: PathBuf,
    pub number_holders: u64,
    // Geth
    pub geth_engine: GethEngineConfig,
    pub geth_indexer: GethPendingTxsIndexer,
    pub celestia_url: Option<String>,
    pub celestia_token: Option<String>,
    pub fee_recipient: Option<Address>,
}
