use rome_sdk::rome_evm_client::PayerConfig;
use rome_sdk::rome_geth::indexers::pending_txs::GethPendingTxsIndexer;
use solana_sdk::commitment_config::CommitmentLevel;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct RheaConfig {
    pub rpc_urls: Vec<url::Url>,
    pub commitment: CommitmentLevel,
    pub program_id: String,
    pub chain_id: u64,
    pub payers: Vec<PayerConfig>,
    pub geth_indexer: GethPendingTxsIndexer,
    pub mempool_ttl: Option<u64>,
}
