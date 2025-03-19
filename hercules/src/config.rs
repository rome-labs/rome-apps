use crate::api::admin::{start_rpc_server, HerculesAdmin};
use rome_sdk::rome_evm_client::indexer::config::{BlockParserConfig, BlockProducerConfig, EthereumStorageConfig, SolanaStorageConfig};
use rome_sdk::rome_evm_client::indexer::{RollupIndexer, SolanaBlockLoader, StandaloneIndexer};
use rome_sdk::rome_solana::config::SolanaConfig;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::slot_history::Slot;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::task::JoinHandle;

const DEFAULT_BLOCK_LOADER_BATCH_SIZE: Slot = 128;
const BLOCK_RETRIES: usize = 10;
const TX_RETRIES: usize = 10;
const RETRY_INTERVAL: Duration = Duration::from_secs(10);

#[derive(Clone, serde::Serialize, serde::Deserialize, Debug)]
pub enum HerculesMode {
    Indexer,
    Recovery,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct HerculesConfig {
    pub solana: SolanaConfig,
    pub start_slot: Option<u64>,
    pub end_slot: Option<u64>,
    pub block_parser: BlockParserConfig,
    pub block_producer: Option<BlockProducerConfig>,
    pub solana_storage: SolanaStorageConfig,
    pub ethereum_storage: EthereumStorageConfig,
    pub admin_rpc: SocketAddr,
    pub max_slot_history: Option<Slot>,
    pub block_loader_batch_size: Option<Slot>,
    pub mode: Option<HerculesMode>,
}

impl HerculesConfig {
    pub async fn init(self) -> anyhow::Result<(jsonrpsee::server::ServerHandle, JoinHandle<()>)> {
        let program_id = self.block_parser.program_id;
        let (indexer_started_tx, indexer_started_rx) = tokio::sync::oneshot::channel();
        let solana_client = Arc::new(RpcClient::new_with_commitment(
            self.solana.rpc_url.to_string(),
            CommitmentConfig {
                commitment: self.solana.commitment,
            },
        ));

        let solana_block_storage = self.solana_storage.init()?;
        let block_parser = self.block_parser.init(solana_block_storage.clone());
        let ethereum_block_storage = self.ethereum_storage.init()?;
        let block_producer = self
            .block_producer
            .as_ref()
            .map(|c| c.init().expect("Failed to create block producer"));

        let indexer = StandaloneIndexer {
            solana_block_loader: SolanaBlockLoader {
                solana_block_storage: solana_block_storage.clone(),
                client: solana_client,
                commitment: self.solana.commitment,
                program_id,
                batch_size: self
                    .block_loader_batch_size
                    .unwrap_or(DEFAULT_BLOCK_LOADER_BATCH_SIZE),
                block_retries: BLOCK_RETRIES,
                tx_retries: TX_RETRIES,
                retry_int: RETRY_INTERVAL,
            },
            rollup_indexer: RollupIndexer::new(
                block_parser,
                solana_block_storage.clone(),
                ethereum_block_storage.clone(),
                block_producer,
                self.max_slot_history,
            ),
        };

        let block_production_api_enabled = self.block_producer.is_none();
        let server_jh = start_rpc_server(
            Arc::new(HerculesAdmin::new(
                solana_block_storage,
                ethereum_block_storage,
                Some(indexer_started_rx),
            )),
            self.admin_rpc,
            block_production_api_enabled,
        )
        .await?;

        let indexer_jh = match self.mode.clone().unwrap_or(HerculesMode::Indexer) {
            HerculesMode::Indexer => {
                indexer.start_indexing(self.start_slot, Some(indexer_started_tx), 400)
            }
            HerculesMode::Recovery => indexer.start_recovery(
                self.start_slot
                    .unwrap_or_else(|| panic!("start_slot is required for recovery")),
                self.end_slot,
            ),
        };

        Ok((server_jh, indexer_jh))
    }
}
