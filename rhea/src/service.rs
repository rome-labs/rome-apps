use std::sync::Arc;
use tokio::sync::Mutex;

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
    state: Arc<Mutex<RheaState>>,
}

struct RheaState {
    last_block_timestamp: Option<u64>, // To track the previous block's timestamp in seconds
    timestamp_offset: u64,             // To handle offset in milliseconds
}

impl RheaService {
    /// Create a new RheaService from RheaConfig
    pub fn new(rome: Rome, geth_engine: GethEngine, indexer: Indexer) -> anyhow::Result<Self> {
        Ok(Self {
            rome,
            geth_engine,
            indexer,
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

    async fn state_advance_loop(&self, mut block_rx: BlockReceiver) {
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
                let _ = self
                    .geth_engine
                    .advance_rollup_state(transactions, adjusted_timestamp)
                    .await;
                // tracing::info!("Advanced rollup state: {:?}", result);
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

        tokio::select! {
            res = self.start_indexer(start_slot, &block_tx, idx_started_tx) => {
                anyhow::bail!("Indexer Reader Exited: {:?}", res);
            },
            res = self.subscribe_to_rollup(idx_started_rx, geth_http_addr, geth_poll_interval_ms, geth_pending_tx) => {
                anyhow::bail!("Rollup subscription error: {:?}", res);
            }
            res = self.mempool_loop(geth_pending_rx) => {
                anyhow::bail!("Geth pending transactions channel closed unexpectedly: {:?}", res);
            },
            res = self.state_advance_loop(block_rx) => {
                anyhow::bail!("Indexer blocks channel closed unexpectedly {:?}", res);
            }
        }
    }
}
