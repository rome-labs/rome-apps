use rome_da::celestia::types::DaSubmissionBlock;
use rome_da::celestia::RomeDaClient;
use std::sync::Arc;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::sync::Mutex;
use tokio::task;

use ethers::types::{transaction::eip2718::TypedTransaction, Signature};
use rome_sdk::rome_evm_client::indexer::indexer::{BlockReceiver, BlockSender, Indexer};
use rome_sdk::rome_geth::abstracted::subscribe_to_rollup;
use rome_sdk::rome_geth::engine::GethEngine;
use rome_sdk::rome_geth::types::{GethTxPoolReceiver, GethTxPoolSender};
use rome_sdk::{EthSignedTxTuple, RheaTx, Rome};
use solana_sdk::clock::Slot;
use tokio::sync::{mpsc, oneshot};
use url::Url;

/// Listens to Geth mempool channel, sends transaction to Solana, and then
/// does a fork choice on Geth
pub struct RheaService {
    rome: Rome,
    geth_engine: GethEngine,
    indexer: Indexer,
    da_client: Option<RomeDaClient>,
    state: Arc<Mutex<RheaState>>,
}

struct RheaState {
    last_block_timestamp: Option<u64>, // To track the previous block's timestamp in seconds
    timestamp_offset: u64,             // To handle offset in milliseconds
}

impl RheaService {
    /// Create a new RheaService from RheaConfig
    pub fn new(
        rome: Rome,
        geth_engine: GethEngine,
        indexer: Indexer,
        da_client: Option<RomeDaClient>,
    ) -> anyhow::Result<Self> {
        Ok(Self {
            rome,
            geth_engine,
            indexer,
            da_client,
            state: Arc::new(Mutex::new(RheaState {
                last_block_timestamp: None,
                timestamp_offset: 0,
            })),
        })
    }

    async fn start_indexer<'a>(
        &'a self,
        start_slot: Slot,
        block_tx: &'a BlockSender,
        idx_started_tx: oneshot::Sender<()>,
    ) {
        self.indexer
            .start(start_slot, 400, &block_tx, move || {
                tracing::info!("Indexer started.");
                if let Err(_) = idx_started_tx.send(()) {
                    tracing::error!("Failed to send indexer started signal");
                }
            })
            .await;
    }

    async fn subscribe_to_rollup(
        &self,
        idx_started_rx: oneshot::Receiver<()>,
        geth_http_addr: Url,
        geth_poll_interval_ms: u64,
        geth_pending_tx: GethTxPoolSender,
    ) -> anyhow::Result<()> {
        match idx_started_rx.await {
            Ok(_) => {
                subscribe_to_rollup(geth_http_addr, geth_poll_interval_ms, geth_pending_tx).await
            }
            Err(err) => anyhow::bail!("Failed to subscribe to rollup: {:?}", err),
        }
    }

    async fn mempool_loop(&self, mut geth_rx: GethTxPoolReceiver) {
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
                    match self.rome.compose_rollup_tx(rhea_tx).await {
                        Ok(rome_tx) => match self.rome.send_and_confirm_tx(rome_tx).await {
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

    async fn state_advance_loop(
        &self,
        mut block_rx: BlockReceiver,
        da_tx: Option<UnboundedSender<DaSubmissionBlock>>,
    ) {
        while let Some(block) = block_rx.recv().await {
            let block_timestamp = block.timestamp.as_u64();
            let adjusted_timestamp = {
                let mut state = self.state.lock().await;
                match state.last_block_timestamp {
                    Some(last_timestamp) => {
                        if block_timestamp == last_timestamp {
                            state.timestamp_offset += 400;
                            last_timestamp * 1000 + state.timestamp_offset
                        } else {
                            state.timestamp_offset = 0;
                            block_timestamp * 1000
                        }
                    }
                    None => block_timestamp * 1000, // First block, no adjustment needed
                }
            };
            {
                let mut state = self.state.lock().await;
                state.last_block_timestamp = Some(block_timestamp);
            }

            if let Some(transactions) = match self.indexer.get_transaction_storage().read() {
                Ok(lock) => Some(block.get_transactions(&lock)),
                Err(e) => {
                    tracing::warn!("Failed to lock transaction sender: {:?}", e);
                    None
                }
            } {
                tracing::info!(
                    "Advancing rollup state for block number: {:?}, transactions: {:?}",
                    block.number,
                    transactions.len()
                );
                let _ = self
                    .geth_engine
                    .advance_rollup_state(&transactions, adjusted_timestamp)
                    .await;

                if let Some(da_tx) = &da_tx {
                    da_tx
                        .send(DaSubmissionBlock {
                            block_number: block.number,
                            timestamp: block.timestamp,
                            transactions: transactions.iter().map(|(tx, _)| tx.clone()).collect(),
                        })
                        .expect("DA submission channel closed");
                }
            }
        }
    }

    async fn batch_da_submissions(&self, mut da_rx: UnboundedReceiver<DaSubmissionBlock>) {
        if let Some(da_client) = &self.da_client {
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
        self,
        start_slot: Slot,
        geth_http_addr: Url,
        geth_poll_interval_ms: u64,
    ) -> anyhow::Result<()> {
        tracing::info!("Starting Rhea Service...");

        let (geth_pending_tx, geth_pending_rx) = mpsc::unbounded_channel();
        let (block_tx, block_rx) = mpsc::unbounded_channel();
        let (idx_started_tx, idx_started_rx) = oneshot::channel();

        let indexer_res = self.start_indexer(start_slot, &block_tx, idx_started_tx);
        let rollup_res = self.subscribe_to_rollup(
            idx_started_rx,
            geth_http_addr,
            geth_poll_interval_ms,
            geth_pending_tx,
        );
        let mempool_res = self.mempool_loop(geth_pending_rx);

        if self.da_client.is_some() {
            let (da_tx, da_rx) = mpsc::unbounded_channel::<DaSubmissionBlock>();
            let state_advance_res = self.state_advance_loop(block_rx, Some(da_tx));
            let da_submission_res = self.batch_da_submissions(da_rx);

            tokio::select! {
                res = indexer_res => {
                    anyhow::bail!("Indexer Reader Exited: {:?}", res);
                },
                res = rollup_res => {
                    anyhow::bail!("Rollup subscription error: {:?}", res);
                }
                res = mempool_res => {
                    anyhow::bail!("Geth pending transactions channel closed unexpectedly: {:?}", res);
                },
                res = state_advance_res => {
                    anyhow::bail!("Indexer blocks channel closed unexpectedly {:?}", res);
                },
                res = da_submission_res => {
                    anyhow::bail!("DA submission channel closed unexpectedly: {:?}", res);
                }
            }
        } else {
            let state_advance_res = self.state_advance_loop(block_rx, None);

            tokio::select! {
                res = indexer_res => {
                    anyhow::bail!("Indexer Reader Exited: {:?}", res);
                },
                res = rollup_res => {
                    anyhow::bail!("Rollup subscription error: {:?}", res);
                }
                res = mempool_res => {
                    anyhow::bail!("Geth pending transactions channel closed unexpectedly: {:?}", res);
                },
                res = state_advance_res => {
                    anyhow::bail!("Indexer blocks channel closed unexpectedly {:?}", res);
                },
            }
        }
    }
}
