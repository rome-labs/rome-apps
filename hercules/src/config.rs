use crate::api::admin::{start_rpc_server, HerculesAdmin};
use rome_sdk::rome_evm_client::indexer::config::{
    RollupIndexerConfig, SolanaBlockLoaderConfig, StorageConfig,
};
use rome_sdk::rome_evm_client::indexer::{ProgramResult, StandaloneIndexer};
#[allow(unused_imports)]
use solana_sdk::commitment_config::CommitmentLevel;
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
    pub storage: StorageConfig,
    pub block_loader: Option<SolanaBlockLoaderConfig>,
    pub admin_rpc: SocketAddr,
    pub rollup_indexer: Option<RollupIndexerConfig>,
    pub mode: Option<HerculesMode>,
}

const INDEXING_INT_MS: u64 = 400;

impl HerculesConfig {
    pub async fn init(
        self,
    ) -> anyhow::Result<(
        jsonrpsee::server::ServerHandle,
        JoinHandle<ProgramResult<()>>,
    )> {
        let (solana_block_storage, ethereum_block_storage) = self.storage.init().await?;
        let (indexer_started_tx, indexer_started_rx) = tokio::sync::oneshot::channel();

        let solana_block_loader = self
            .block_loader
            .map(|config| config.init(solana_block_storage.clone()));

        let block_production_api_enabled = self
            .rollup_indexer
            .as_ref()
            .map_or(false, |config| config.block_production_api_enabled());

        let rollup_indexer = self.rollup_indexer.map(|config| {
            config.init(
                solana_block_storage.clone(),
                ethereum_block_storage.clone(),
                solana_block_loader.as_ref().map(|b| b.program_id),
            )
        });

        let indexer = StandaloneIndexer {
            solana_block_loader,
            rollup_indexer,
        };

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
                indexer.start_indexing(self.start_slot, Some(indexer_started_tx), INDEXING_INT_MS)
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
