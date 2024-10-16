use ethers::types::{transaction::eip2718::TypedTransaction, Signature};
use rome_da::celestia::types::DaSubmissionBlock;
use rome_da::celestia::RomeDaClient;
use rome_sdk::rome_evm_client::indexer::indexer::{BlockReceiver, BlockSender, Indexer};
use rome_sdk::rome_geth::engine::GethEngine;
use rome_sdk::rome_geth::indexers::pending_txs::GethPendingTxsIndexer;
use rome_sdk::rome_geth::types::{GethTxPoolReceiver, GethTxPoolSender};
use rome_sdk::rome_utils::services::ServiceRunner;
use rome_sdk::{EthSignedTxTuple, RheaTx, Rome};
use solana_sdk::clock::Slot;
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::Mutex;
use tokio::sync::{mpsc, oneshot};
use tokio::task;

/// Listens to geth mempool channel and
/// sends transaction to neon proxy
/// then does a fork choice on geth
pub struct RheaService;

impl RheaService {
    /// Start the indexer
    /// and notify the caller when it's started
    async fn start_indexer(
        indexer: Arc<Indexer>,
        start_slot: Slot,
        block_tx: BlockSender,
        idx_started_tx: oneshot::Sender<()>,
    ) {
        tracing::info!("Starting Indexer...");

        indexer
            .start(start_slot, 400, block_tx, Some(idx_started_tx))
            .await
    }

    /// Subscribe to rollup and listen to pending transactions
    async fn subscribe_to_rollup(
        idx_started_rx: oneshot::Receiver<()>,
        geth_indexer: GethPendingTxsIndexer,
        geth_pending_tx: GethTxPoolSender,
    ) -> anyhow::Result<()> {
        tracing::info!("Geth Indexer waiting for indexer to start");

        match idx_started_rx.await {
            Ok(_) => {
                tracing::info!("Starting Geth Mempool Indexer");

                geth_indexer
                    .listen(geth_pending_tx, ServiceRunner::default())
                    .await
            }
            Err(err) => anyhow::bail!("Failed to subscribe to rollup: {:?}", err),
        }
    }

    /// Listen to the mempool channel and send transactions to the rollup
    async fn mempool_loop(rome: Rome, mut geth_rx: GethTxPoolReceiver) {
        while let Some(tx) = geth_rx.recv().await {
            tracing::info!("Received transaction: {:?}", tx);

            let signature = Signature {
                r: tx.r,
                s: tx.s,
                v: tx.v.as_u64(),
            };

            match TypedTransaction::try_from(&tx) {
                Ok(tx) => {
                    let rhea_tx = RheaTx::new(EthSignedTxTuple::new(tx, signature));
                    match rome.compose_rollup_tx(rhea_tx).await {
                        Ok(mut rome_tx) => match rome.send_and_confirm(&mut *rome_tx).await {
                            Ok(sig) => tracing::info!("Sent and confirmed tx: {:?}", sig),
                            Err(err) => tracing::warn!("Failed to send transaction: {:?}", err),
                        },
                        Err(e) => {
                            tracing::warn!("Failed to compose transaction: {:?}", e);
                        }
                    }
                }
                Err(err) => {
                    tracing::warn!("Failed to convert pool tx into TypedTransaction: {:?}", err);
                }
            }
        }
    }

    /// Listen to the block channel and advance the rollup state
    async fn state_advance_loop(
        indexer: Arc<Indexer>,
        geth_engine: GethEngine,
        mut block_rx: BlockReceiver,
    ) {
        // To track the previous block's timestamp in seconds
        let mut last_block_timestamp: Option<u64> = None;
        // To handle offset in milliseconds
        let mut timestamp_offset: u64 = 0;

        while let Some(block) = block_rx.recv().await {
            let block_timestamp = block.timestamp.as_u64();

            let adjusted_timestamp = {
                match last_block_timestamp {
                    Some(last_timestamp) => {
                        if block_timestamp == last_timestamp {
                            timestamp_offset += 400;
                            last_timestamp * 1000 + timestamp_offset
                        } else {
                            timestamp_offset = 0;
                            block_timestamp * 1000
                        }
                    }
                    None => block_timestamp * 1000, // First block, no adjustment needed
                }
            };

            last_block_timestamp = Some(block_timestamp);

            let lock = indexer.get_transaction_storage();
            let lock = lock.read().await;

            let transactions = block.get_transactions(&lock);

            drop(lock);

            if !transactions.is_empty() {
                let result = geth_engine
                    .advance_rollup_state(&transactions, adjusted_timestamp)
                    .await;

                tracing::info!("Advanced rollup state: {:?}", result);
            }
        }
    }

    async fn batch_da_submissions(
        mut da_rx: UnboundedReceiver<DaSubmissionBlock>,
        da_client: Option<RomeDaClient>,
    ) {
        if let Some(da_client) = da_client {
            let buffer = Arc::new(Mutex::new(Vec::new()));
            let buffer_clone = buffer.clone();

            task::spawn({
                async move {
                    while let Some(event) = da_rx.recv().await {
                        let mut buffer = buffer.lock().await;
                        buffer.push(event);
                    }
                }
            });

            loop {
                let mut buffer_ = buffer_clone.lock().await;
                if !buffer_.is_empty() {
                    let blocks = buffer_.split_off(0);
                    match da_client.submit_blocks(&blocks).await {
                        Ok(_) => tracing::info!("Submitted {:?} blocks", blocks.len()),
                        Err(e) => tracing::error!("Failed to submit blocks: {:?}", e),
                    }
                }

                drop(buffer_);
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            }
        }
    }

    /// Start the Rhea service
    pub async fn start(
        rome: Rome,
        geth_engine: GethEngine,
        indexer: Arc<Indexer>,
        start_slot: Slot,
        geth_indexer: GethPendingTxsIndexer,
        da_client: Option<RomeDaClient>,
    ) -> anyhow::Result<()> {
        tracing::info!("Starting Rhea Service...");

        let (geth_pending_tx, geth_pending_rx) = mpsc::unbounded_channel();
        let (block_tx, block_rx) = mpsc::unbounded_channel();
        let (idx_started_tx, idx_started_rx) = oneshot::channel();

        let indexer_jh = Self::start_indexer(indexer.clone(), start_slot, block_tx, idx_started_tx);
        let geth_jh = Self::subscribe_to_rollup(idx_started_rx, geth_indexer, geth_pending_tx);
        let mempool_jh = Self::mempool_loop(rome, geth_pending_rx);
        let state_advance_jh = Self::state_advance_loop(indexer, geth_engine, block_rx);

        // tokio spawn all
        let indexer_jh = tokio::spawn(indexer_jh);
        let geth_jh = tokio::spawn(geth_jh);
        let mempool_jh = tokio::spawn(mempool_jh);
        let state_advance_jh = tokio::spawn(state_advance_jh);

        if da_client.is_some() {
            // todo: remove unnecessary channel
            let (_da_tx, da_rx) = mpsc::unbounded_channel::<DaSubmissionBlock>();
            let da_submission_jh = Self::batch_da_submissions(da_rx, da_client);

            tokio::select! {
                res = indexer_jh => {
                    anyhow::bail!("Indexer Reader Exited: {:?}", res);
                },
                res = geth_jh => {
                    anyhow::bail!("Rollup subscription error: {:?}", res);
                }
                res = mempool_jh => {
                    anyhow::bail!("Geth pending transactions channel closed unexpectedly: {:?}", res);
                },
                res = state_advance_jh => {
                    anyhow::bail!("Indexer blocks channel closed unexpectedly {:?}", res);
                }
                res = da_submission_jh => {
                    anyhow::bail!("DA submission channel closed unexpectedly: {:?}", res);
                }
            }
        } else {
            tokio::select! {
                res = indexer_jh => {
                    anyhow::bail!("Indexer Reader Exited: {:?}", res);
                },
                res = geth_jh => {
                    anyhow::bail!("Rollup subscription error: {:?}", res);
                }
                res = mempool_jh => {
                    anyhow::bail!("Geth pending transactions channel closed unexpectedly: {:?}", res);
                },
                res = state_advance_jh => {
                    anyhow::bail!("Indexer blocks channel closed unexpectedly {:?}", res);
                }
            }
        }
    }
}
