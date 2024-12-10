use rome_sdk::rome_evm_client::PayerConfig;
use rome_sdk::rome_geth::indexers::pending_txs::GethPendingTxsIndexer;
use rome_sdk::rome_solana::config::SolanaConfig;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct RheaConfig {
    pub solana: SolanaConfig,
    pub program_id: String,
    pub chain_id: u64,
    pub payers: Vec<PayerConfig>,
    pub geth_indexer: GethPendingTxsIndexer,
}
