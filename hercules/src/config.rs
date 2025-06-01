use crate::api::admin::{start_rpc_server, HerculesAdmin};
use rome_sdk::rome_evm_client::indexer::config::{
    BlockParserConfig, BlockProducerConfig, EthereumStorageConfig, SolanaBlockLoaderConfig,
    SolanaStorageConfig,
};
use rome_sdk::rome_evm_client::indexer::{ProgramResult, RollupIndexer, StandaloneIndexer};
#[allow(unused_imports)]
use solana_sdk::commitment_config::CommitmentLevel;
use solana_sdk::slot_history::Slot;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::task::JoinHandle;

#[derive(Clone, serde::Serialize, serde::Deserialize, Debug)]
pub enum HerculesMode {
    Indexer,
    Recovery,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct HerculesConfig {
    pub start_slot: Option<u64>,
    pub end_slot: Option<u64>,
    pub block_loader: Option<SolanaBlockLoaderConfig>,
    pub solana_storage: SolanaStorageConfig,
    pub ethereum_storage: EthereumStorageConfig,
    pub admin_rpc: SocketAddr,
    pub max_slot_history: Option<Slot>,
    pub block_parser: BlockParserConfig,
    pub block_producer: Option<BlockProducerConfig>,
    pub mode: Option<HerculesMode>,
}

impl HerculesConfig {
    pub async fn init(
        self,
    ) -> anyhow::Result<(
        jsonrpsee::server::ServerHandle,
        JoinHandle<ProgramResult<()>>,
    )> {
        let solana_block_storage = self.solana_storage.init().await?;
        let ethereum_block_storage = self.ethereum_storage.init()?;
        let (indexer_started_tx, indexer_started_rx) = tokio::sync::oneshot::channel();

        let block_parser = self.block_parser.init(
            solana_block_storage.clone(),
            self.block_loader.as_ref().map(|b| b.program_id),
        );
        let solana_block_loader = self
            .block_loader
            .map(|config| config.init(solana_block_storage.clone()));
        let block_producer = self
            .block_producer
            .as_ref()
            .map(|c| c.init().expect("Failed to create block producer"));

        let indexer = StandaloneIndexer {
            solana_block_loader,
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
