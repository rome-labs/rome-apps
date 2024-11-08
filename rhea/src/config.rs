use ethers::types::Address;
use rome_sdk::rome_geth::engine::config::GethEngineConfig;
use rome_sdk::rome_geth::indexers::pending_txs::GethPendingTxsIndexer;
use rome_sdk::rome_solana::config::SolanaConfig;
use rome_sdk::rome_evm_client::PayerConfig;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct RheaConfig {
    // Same as proxy
    pub chain_id: u64,
    pub start_slot: Option<u64>,
    pub solana: SolanaConfig,
    pub solana_indexer: Option<SolanaConfig>,
    pub program_id: String,
    pub payers: Vec<PayerConfig>,
    // Geth
    pub geth_engine: GethEngineConfig,
    pub geth_indexer: GethPendingTxsIndexer,
    pub celestia_url: Option<String>,
    pub celestia_token: Option<String>,
    pub fee_recipient: Option<Address>,
}
