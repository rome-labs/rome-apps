use ethers::types::{transaction::eip2718::TypedTransaction, Signature};
use rome_sdk::rome_geth::indexers::pending_txs::GethPendingTxsIndexer;
use rome_sdk::rome_geth::types::{GethTxPoolReceiver, GethTxPoolSender, GethTxPoolTx};
use rome_sdk::rome_utils::services::ServiceRunner;
use rome_sdk::{EthSignedTxTuple, RheaTx, Rome};
use std::collections::{BTreeMap, HashSet};
use std::sync::Arc;
use tokio::sync::mpsc;

/// Listens to geth mempool channel and
/// sends transaction to proxy
/// then does a fork choice on geth
pub struct RheaService;

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

    async fn process_sender_txs(
        rome: Arc<Rome>,
        txs: BTreeMap<u64, GethTxPoolTx>,
    ) -> HashSet<String> {
        let mut completed = HashSet::new();
        for (_, geth_tx) in txs {
            let signature = Signature {
                r: geth_tx.r,
                s: geth_tx.s,
                v: geth_tx.v.as_u64(),
            };

            match TypedTransaction::try_from(&geth_tx) {
                Ok(tx) => {
                    let rhea_tx = RheaTx::new(EthSignedTxTuple::new(tx, signature));
                    match rome.compose_rollup_tx(rhea_tx).await {
                        Ok(mut rome_tx) => {
                            if let Err(err) = rome.send_and_confirm(&mut *rome_tx).await {
                                tracing::warn!("Failed to send transaction: {:?}", err)
                            }
                        }
                        Err(e) => {
                            tracing::warn!("Failed to compose transaction: {:?}", e);
                        }
                    }
                }
                Err(err) => {
                    tracing::warn!("Failed to convert pool tx into TypedTransaction: {:?}", err);
                }
            }

            completed.insert(geth_tx.hash.to_string());
        }

        completed
    }

    /// Listen to the mempool channel and send transactions to the rollup
    async fn mempool_loop(rome: Arc<Rome>, mut geth_rx: GethTxPoolReceiver) {
        let mut sent_txs = HashSet::<String>::new();
        while let Some(res) = geth_rx.recv().await {
            let Some(result) = &res.result else {
                tracing::warn!("Error in response: {:?}", res.error);
                continue;
            };

            let mut to_process = BTreeMap::new();
            result.queued.iter().for_each(|(sender, queued_txs)| {
                queued_txs.iter().for_each(|(nonce, tx)| {
                    if !sent_txs.contains(&tx.hash.to_string()) {
                        to_process
                            .entry(sender.clone())
                            .or_insert_with(BTreeMap::new)
                            .insert(*nonce, tx.clone());
                    }
                })
            });

            result.pending.iter().for_each(|(sender, pending_txs)| {
                pending_txs.iter().for_each(|(nonce, tx)| {
                    if !sent_txs.contains(&tx.hash.to_string()) {
                        to_process
                            .entry(sender.clone())
                            .or_insert_with(BTreeMap::new)
                            .insert(*nonce, tx.clone());
                    }
                });
            });

            let futures = to_process.into_values().map(|sender_txs| {
                tokio::spawn(Self::process_sender_txs(rome.clone(), sender_txs))
            });

            for result in futures::future::join_all(futures).await {
                match result {
                    Ok(completed_txs) => sent_txs.extend(completed_txs),
                    Err(err) => {
                        tracing::warn!("Failed to send transactions: {:?}", err);
                    }
                }
            }
        }
    }

    /// Start the Rhea service
    pub async fn start(rome: Rome, geth_indexer: GethPendingTxsIndexer) -> anyhow::Result<()> {
        tracing::info!("Starting Rhea Service...");

        let (geth_pending_tx, geth_pending_rx) = mpsc::unbounded_channel();
        let geth_jh = Self::subscribe_to_rollup(geth_indexer, geth_pending_tx);
        let mempool_jh = Self::mempool_loop(Arc::new(rome), geth_pending_rx);

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
