use clap::Parser;
use tokio::signal;

use self::cli::Cli;
use self::config::HerculesConfig;
use crate::api::admin::HerculesAdmin;
use crate::config::EthereumStorageConfig;
use anyhow::bail;
use dotenv::dotenv;
use rome_sdk::rome_evm_client::indexer::{inmemory, pg_storage, StandaloneIndexer};
use rome_sdk::rome_geth::engine::engine_api_block_producer::EngineAPIBlockProducer;
use rome_sdk::rome_geth::engine::GethEngine;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;
use std::sync::Arc;

mod api;
mod cli;
mod config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    tracing_subscriber::fmt().json().init();
    let config: HerculesConfig = Cli::parse().load_config().await?;
    let geth_engine = GethEngine::new(config.geth_engine);
    let program_id = Pubkey::from_str(&config.program_id)?;

    let solana_client = Arc::new(RpcClient::new_with_commitment(
        config.solana.rpc_url.to_string(),
        CommitmentConfig {
            commitment: config.solana.commitment,
        },
    ));

    let solana_block_storage = Arc::new(pg_storage::SolanaBlockStorage::new(
        config.solana_storage.create_pool()?,
    ));

    let (_admin_server, indexer_jh) = match config.ethereum_storage {
        EthereumStorageConfig::PgStorage(_eth_pg_config) => {
            todo!()
        }
        EthereumStorageConfig::InMemoryStorage => {
            let ethereum_block_storage =
                Arc::new(inmemory::EthereumBlockStorage::new(config.chain_id));

            let (indexer_started_tx, indexer_started_rx) = tokio::sync::oneshot::channel();

            (
                HerculesAdmin::new(
                    solana_block_storage.clone(),
                    ethereum_block_storage.clone(),
                    Some(indexer_started_rx),
                )
                .start_rpc_server(config.admin_rpc)
                .await?,
                StandaloneIndexer {
                    solana_client,
                    commitment_level: config.solana.commitment,
                    rome_evm_pubkey: program_id,
                    solana_block_storage,
                    ethereum_block_storage,
                    block_producer: EngineAPIBlockProducer::new(Arc::new(geth_engine)),
                }
                .start_indexing(
                    config.start_slot,
                    Some(indexer_started_tx),
                    400,
                    config.max_slot_history,
                ),
            )
        }
    };

    // Shutdown on ctrl-c
    tokio::select! {
        res = indexer_jh => {
            bail!("Rhea Service Exited: {:?}", res);
        },
        res = signal::ctrl_c() => {
            bail!("Shutting down Rhea: {:?}", res);
        },
    }
}
