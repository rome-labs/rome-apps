use anyhow::bail;
use ethers::types::TxHash;
use rome_sdk::rome_evm_client::PayerConfig;
use rome_sdk::rome_solana::config::SolanaConfig;
use rome_sdk::{RheaTx, Rome, RomeConfig};
use solana_sdk::commitment_config::CommitmentLevel;
use std::collections::HashMap;

pub struct RomeSender {
    clients: Vec<Rome>,
}

impl RomeSender {
    pub async fn new(
        client_rpcs: Vec<url::Url>,
        commitment: CommitmentLevel,
        rollups: &HashMap<u64, String>,
        payers: &Vec<PayerConfig>,
    ) -> anyhow::Result<Self> {
        let mut clients = vec![];

        for rpc_url in client_rpcs {
            clients.push(
                Rome::new_with_config(RomeConfig {
                    solana_config: SolanaConfig {
                        rpc_url,
                        commitment,
                    },
                    rollups: rollups.clone(),
                    payers: payers.clone(),
                })
                .await?,
            );
        }

        Ok(Self { clients })
    }

    pub async fn send_transaction<'a>(
        &self,
        hash: &TxHash,
        sender_addr: &str,
        rhea_tx: RheaTx<'a>,
    ) -> anyhow::Result<()> {
        for client in &self.clients {
            match client.compose_rollup_tx(rhea_tx.clone()).await {
                Ok(mut rome_tx) => {
                    if let Err(err) = client.send_and_confirm(&mut *rome_tx).await {
                        tracing::warn!(
                            "SenderQueue {}: Failed to send transaction {:?}: {:?}",
                            sender_addr,
                            hash,
                            err
                        )
                    } else {
                        tracing::info!(
                            "SenderQueue {}: Transaction {:?} executed in Rome-EVM",
                            sender_addr,
                            hash,
                        );
                        return Ok(());
                    }
                }
                Err(e) => {
                    tracing::warn!(
                        "SenderQueue {}: Failed to compose transaction {:?}: {:?}",
                        sender_addr,
                        hash,
                        e
                    );
                }
            }
        }

        bail!("Failed to send transaction")
    }
}
