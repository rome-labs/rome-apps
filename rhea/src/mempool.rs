use crate::mempool_sender::MempoolSender;
use ethers::types::TxHash;
use rome_sdk::rome_geth::types::{GethTxPoolResult, GethTxPoolTx};
use rome_sdk::Rome;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::sync::Mutex;

struct MempoolImpl {
    transactions: HashMap<TxHash, (String, u64)>,
    senders: HashMap<String, UnboundedSender<(u64, GethTxPoolTx)>>,
    sender_ttl: Duration,
    rome: Arc<Rome>,
    drop_sender_tx: UnboundedSender<String>,
}

impl MempoolImpl {
    pub fn new(
        rome: Arc<Rome>,
        sender_ttl: Duration,
        drop_sender_tx: UnboundedSender<String>,
    ) -> Self {
        Self {
            transactions: HashMap::new(),
            senders: HashMap::new(),
            rome,
            sender_ttl,
            drop_sender_tx,
        }
    }

    pub async fn update(&mut self, geth_txs: &GethTxPoolResult) -> Vec<TxHash> {
        let mut new_txs = Vec::with_capacity(geth_txs.queued.len() + geth_txs.pending.len());

        for (sender, queued_txs) in geth_txs.queued.iter() {
            for (nonce, tx) in queued_txs {
                if self.add_tx(sender.clone(), *nonce, tx.clone()).await {
                    new_txs.push(tx.hash);
                }
            }
        }

        for (sender, queued_txs) in geth_txs.pending.iter() {
            for (nonce, tx) in queued_txs {
                if self.add_tx(sender.clone(), *nonce, tx.clone()).await {
                    new_txs.push(tx.hash);
                }
            }
        }

        new_txs
    }

    pub async fn remove_txs(&mut self, txs: Vec<TxHash>) {
        for tx in &txs {
            self.transactions.remove(tx);
        }
    }

    pub async fn remove_sender(&mut self, sender_address: String) {
        self.senders.remove(&sender_address);
    }

    async fn add_tx(&mut self, sender: String, nonce: u64, tx: GethTxPoolTx) -> bool {
        let tx_hash = tx.hash;
        let None = self.transactions.insert(tx_hash, (sender.clone(), nonce)) else {
            // Transaction already known
            return false;
        };

        let Err(err) = self
            .senders
            .entry(sender.clone())
            .or_insert_with(|| {
                MempoolSender::init(
                    sender.clone(),
                    self.rome.clone(),
                    self.sender_ttl,
                    self.drop_sender_tx.clone(),
                )
            })
            .send((nonce, tx))
        else {
            // Transaction added to mempool
            return true;
        };

        // Failed to add transaction to mempool
        self.senders.remove(&sender);
        self.transactions.remove(&tx_hash);
        tracing::warn!(
            "Failed to add tx {} to mempool sender {}: {:?}",
            tx_hash,
            sender,
            err
        );

        false
    }
}

pub struct Mempool {
    inner: Arc<Mutex<MempoolImpl>>,
    mempool_ttl: Duration,
}

async fn process_dropped_senders(
    mut drop_sender_rx: UnboundedReceiver<String>,
    mempool_impl: Arc<Mutex<MempoolImpl>>,
) {
    while let Some(sender_address) = drop_sender_rx.recv().await {
        tracing::info!("Removing sender {}", sender_address);
        mempool_impl
            .lock()
            .await
            .remove_sender(sender_address)
            .await;
    }
}

impl Mempool {
    pub fn new(rome: Arc<Rome>, mempool_ttl: Duration) -> Self {
        let (drop_sender_tx, drop_sender_rx) = tokio::sync::mpsc::unbounded_channel();
        let mempool_impl = Arc::new(Mutex::new(MempoolImpl::new(
            rome,
            mempool_ttl,
            drop_sender_tx,
        )));

        tokio::spawn(process_dropped_senders(
            drop_sender_rx,
            mempool_impl.clone(),
        ));

        Self {
            inner: mempool_impl,
            mempool_ttl,
        }
    }

    pub async fn update(&self, geth_txs: &GethTxPoolResult) {
        let txs = self.inner.lock().await.update(geth_txs).await;
        let mempool = self.inner.clone();
        let mempool_ttl = self.mempool_ttl;
        tokio::spawn(async move {
            tokio::time::sleep(mempool_ttl).await;
            mempool.lock().await.remove_txs(txs).await;
        });
    }
}
