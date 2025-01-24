use crate::mempool::Mempool;
use rome_sdk::rome_geth::indexers::pending_txs::GethPendingTxsIndexer;
use rome_sdk::rome_geth::types::{GethTxPoolReceiver, GethTxPoolSender};
use rome_sdk::rome_utils::services::ServiceRunner;
use rome_sdk::Rome;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;

/// Listens to geth mempool channel and
/// sends transaction to proxy
/// then does a fork choice on geth
pub struct RheaService;

const DEFAULT_MEMPOOL_TTL_SEC: u64 = 300;

impl RheaService {
    /// Subscribe to rollup and listen to pending transactions
    async fn subscribe_to_rollup(
        geth_indexer: GethPendingTxsIndexer,
        geth_pending_tx: GethTxPoolSender,
    ) -> anyhow::Result<()> {
        tracing::info!("Starting Geth Mempool Indexer");
        geth_indexer
            .listen(geth_pending_tx, ServiceRunner::default())
            .await
    }

    /// Listen to the mempool channel and send transactions to the rollup
    async fn mempool_loop(
        rome: Arc<Rome>,
        mut geth_rx: GethTxPoolReceiver,
        mempool_ttl: Option<Duration>,
    ) {
        let mempool_ttl =
            mempool_ttl.unwrap_or_else(|| Duration::from_secs(DEFAULT_MEMPOOL_TTL_SEC));
        let mempool = Mempool::new(rome.clone(), mempool_ttl);

        while let Some(res) = geth_rx.recv().await {
            let Some(result) = &res.result else {
                tracing::warn!("Error in response: {:?}", res.error);
                continue;
            };

            mempool.update(result).await;
        }
    }

    /// Start the Rhea service
    pub async fn start(
        rome: Rome,
        geth_indexer: GethPendingTxsIndexer,
        mempool_ttl: Option<Duration>,
    ) -> anyhow::Result<()> {
        tracing::info!("Starting Rhea Service...");

        let (geth_pending_tx, geth_pending_rx) = mpsc::unbounded_channel();
        let geth_jh = Self::subscribe_to_rollup(geth_indexer, geth_pending_tx);
        let mempool_jh = Self::mempool_loop(Arc::new(rome), geth_pending_rx, mempool_ttl);

        tokio::select! {
            res = tokio::spawn(geth_jh) => {
                anyhow::bail!("Rollup subscription error: {:?}", res);
            }
            res = tokio::spawn(mempool_jh) => {
                anyhow::bail!("Geth pending transactions channel closed unexpectedly: {:?}", res);
            },
        }
    }
}
