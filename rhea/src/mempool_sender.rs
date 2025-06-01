use ethers::prelude::transaction::eip2718::TypedTransaction;
use ethers::prelude::Signature;
use rome_sdk::rome_evm_client::error::ProgramResult;
use rome_sdk::rome_evm_client::error::RomeEvmError::Custom;
use rome_sdk::rome_geth::types::GethTxPoolTx;
use rome_sdk::{EthSignedTxTuple, RheaTx, Rome};
use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

pub struct MempoolSender {
    sender_addr: String,
    tx_recv: UnboundedReceiver<(u64, GethTxPoolTx)>,
    rome: Arc<Rome>,
}

impl MempoolSender {
    pub fn init(
        sender_addr: String,
        rome: Arc<Rome>,
        sender_ttl: Duration,
        drop_sender_tx: UnboundedSender<String>,
    ) -> UnboundedSender<(u64, GethTxPoolTx)> {
        let (tx_send, tx_recv) = mpsc::unbounded_channel();
        tokio::spawn(
            Self {
                sender_addr,
                tx_recv,
                rome,
            }
            .sender_task(sender_ttl, drop_sender_tx),
        );

        tx_send
    }

    async fn sender_task(mut self, sender_ttl: Duration, drop_sender_tx: UnboundedSender<String>) {
        const BATCH_SIZE: usize = 100;
        const NUM_SEND_TX_RETRIES: usize = 5;
        let mut txs = BTreeMap::new();
        let mut last_processed_nonce = None;

        loop {
            // Try to receive all recent transactions from a queue
            let mut new_events = Vec::with_capacity(BATCH_SIZE);
            let Ok(num_received) = tokio::time::timeout(
                sender_ttl,
                self.tx_recv.recv_many(&mut new_events, BATCH_SIZE),
            )
            .await
            else {
                // Timeout
                if let Err(err) = drop_sender_tx.send(self.sender_addr.clone()) {
                    tracing::warn!("Failed to send sender address to drop_sender_tx: {:?}", err);
                }
                return;
            };

            if num_received == 0 {
                // Sender part of a channel is dropped
                if let Err(err) = drop_sender_tx.send(self.sender_addr.clone()) {
                    tracing::warn!("Failed to send sender address to drop_sender_tx: {:?}", err);
                }
                return;
            }

            // Order transactions by nonce
            for (nonce, tx) in new_events {
                txs.insert(nonce, tx);
            }

            // Send transactions
            while let Some((nonce, geth_tx)) = txs.pop_first() {
                if let Some(last_processed_nonce) = last_processed_nonce {
                    if nonce <= last_processed_nonce {
                        tracing::warn!("SenderQueue {}: Skipping transaction with nonce {} as it is already processed", self.sender_addr, nonce);
                        continue;
                    }
                }

                let tx_hash = geth_tx.hash;
                if let Err(err) = self
                    .send_tx_with_retries(geth_tx, NUM_SEND_TX_RETRIES)
                    .await
                {
                    // Unable to send a transaction with retries - drop mempool sender
                    if let Err(err) = drop_sender_tx.send(self.sender_addr.clone()) {
                        tracing::warn!(
                            "Failed to send sender address to drop_sender_tx: {:?}",
                            err
                        );
                    }

                    tracing::warn!(
                        "SenderQueue {}: Failed to send transaction {}: {:?}",
                        self.sender_addr,
                        tx_hash,
                        err
                    );
                    return;
                }

                last_processed_nonce = Some(nonce);
            }
        }
    }

    #[tracing::instrument(
        name = "rhea::send_tx_with_retries",
        skip(self),
        fields(tx_hash = ?geth_tx.hash)
    )]
    async fn send_tx_with_retries(
        &self,
        geth_tx: GethTxPoolTx,
        num_retries: usize,
    ) -> ProgramResult<()> {
        let mut retries_left = num_retries;
        let signature = Signature {
            r: geth_tx.r,
            s: geth_tx.s,
            v: geth_tx.v.as_u64(),
        };

        match TypedTransaction::try_from(&geth_tx) {
            Ok(tx) => {
                let mut retry_delay = 2;
                loop {
                    let rhea_tx = RheaTx::new(EthSignedTxTuple::new(tx.clone(), signature));
                    match self.rome.compose_rollup_tx(rhea_tx).await {
                        Ok(mut rome_tx) => {
                            if let Err(err) = self.rome.send_and_confirm(&mut *rome_tx).await {
                                tracing::warn!(
                                    "SenderQueue {}: Failed to send transaction {:?}: {:?}",
                                    self.sender_addr,
                                    geth_tx.hash,
                                    err
                                )
                            } else {
                                tracing::info!(
                                    "SenderQueue {}: Transaction {:?} executed in Rome-EVM",
                                    self.sender_addr,
                                    geth_tx.hash,
                                );
                                break Ok(());
                            }
                        }
                        Err(e) => {
                            tracing::warn!(
                                "SenderQueue {}: Failed to compose transaction {:?}: {:?}",
                                self.sender_addr,
                                geth_tx.hash,
                                e
                            );
                        }
                    }

                    if retries_left > 0 {
                        retries_left -= 1;
                        tracing::info!(
                            "SenderQueue {}: Will retry {:?} in {:?} seconds",
                            self.sender_addr,
                            geth_tx.hash,
                            retry_delay
                        );
                        tokio::time::sleep(Duration::from_secs(retry_delay)).await;
                        retry_delay *= 2;
                    } else {
                        break Err(Custom(format!(
                            "SenderQueue {}: No retries left for {:?}",
                            self.sender_addr, geth_tx.hash
                        )));
                    }
                }
            }
            Err(err) => Err(Custom(format!(
                "SenderQueue {}: Failed to convert pool tx into TypedTransaction: {:?}",
                self.sender_addr, err
            ))),
        }
    }
}
