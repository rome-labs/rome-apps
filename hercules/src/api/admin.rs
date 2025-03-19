use crate::api::ApiError::Hercules;
use crate::api::{AdminServer, ApiResult, BlockProductionServer, TestingAPIServer};
use anyhow::Context;
use async_trait::async_trait;
use jsonrpsee::server::{ServerBuilder, ServerHandle};
use jsonrpsee::RpcModule;
use rome_sdk::rome_evm_client::indexer::{
    EthereumBlockStorage, ProducedBlocks, ProducerParams, SolanaBlockStorage,
};
use solana_sdk::clock::Slot;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub struct HerculesAdmin {
    solana_block_storage: Arc<dyn SolanaBlockStorage>,
    ethereum_block_storage: Arc<dyn EthereumBlockStorage>,
    indexer_started: Arc<AtomicBool>,
}

impl HerculesAdmin {
    pub fn new(
        solana_block_storage: Arc<dyn SolanaBlockStorage>,
        ethereum_block_storage: Arc<dyn EthereumBlockStorage>,
        indexer_started_rx: Option<tokio::sync::oneshot::Receiver<()>>,
    ) -> Self {
        let instance = Self {
            solana_block_storage,
            ethereum_block_storage,
            indexer_started: Arc::new(AtomicBool::new(indexer_started_rx.is_none())),
        };

        if let Some(indexer_started_rx) = indexer_started_rx {
            let indexer_started = instance.indexer_started.clone();
            tokio::spawn(async move {
                if let Err(err) = indexer_started_rx.await {
                    tracing::warn!("Failed to await on indexer started signal: {:?}", err)
                } else {
                    indexer_started.store(true, Ordering::Relaxed);
                }
            });
        }

        instance
    }
}

#[async_trait]
impl AdminServer for Arc<HerculesAdmin> {
    async fn in_sync(&self) -> ApiResult<bool> {
        if self.solana_block_storage.get_last_slot().await?.is_none() {
            Err(Hercules("SolanaBlockStorage has no slots".to_string()))
        } else {
            Ok(self.indexer_started.load(Ordering::Relaxed))
        }
    }

    async fn last_solana_storage_slot(&self) -> ApiResult<Option<Slot>> {
        Ok(self.solana_block_storage.get_last_slot().await?)
    }

    async fn last_ethereum_storage_slot(&self) -> ApiResult<Option<Slot>> {
        Ok(self.ethereum_block_storage.get_max_slot_produced().await?)
    }
}

#[async_trait]
impl BlockProductionServer for Arc<HerculesAdmin> {
    async fn get_pending_blocks(&self) -> ApiResult<Option<ProducerParams>> {
        Ok(self.ethereum_block_storage.get_pending_blocks().await?)
    }

    async fn blocks_produced(&self, produced_blocks: ProducedBlocks) -> ApiResult<()> {
        if let Some(producer_params) = self.ethereum_block_storage.get_pending_blocks().await? {
            self.ethereum_block_storage
                .blocks_produced(&producer_params, produced_blocks)
                .await?;
        }

        Ok(())
    }
}

#[async_trait]
impl TestingAPIServer for Arc<HerculesAdmin> {
    async fn deposit(&self) -> ApiResult<()> {
        todo!()
    }
}

pub async fn start_rpc_server(
    server: Arc<HerculesAdmin>,
    host: SocketAddr,
    block_production_api_enabled: bool,
) -> anyhow::Result<ServerHandle> {
    tracing::info!("Starting the RPC server at {host}");

    let rpc = ServerBuilder::default()
        .build(host)
        .await
        .context("Unable to start the RPC server")?;

    let mut module = RpcModule::new(());
    module.merge(AdminServer::into_rpc(server.clone()))?;

    if block_production_api_enabled {
        module.merge(BlockProductionServer::into_rpc(server.clone()))?;
    }

    Ok(rpc.start(module))
}
