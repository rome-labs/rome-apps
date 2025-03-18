use clap::Parser;
use tokio::signal;

use self::cli::Cli;
use self::config::HerculesConfig;
use crate::api::admin::HerculesAdmin;
use crate::config::HerculesMode;
use anyhow::bail;
use dotenv::dotenv;
use ethers::providers::Http;
use rome_sdk::rome_evm_client::indexer::{
    config::EthereumStorageConfig, inmemory, pg_storage, EthereumBlockStorage, SolanaBlockStorage,
    StandaloneIndexer,
};
use rome_sdk::rome_geth::engine::engine_api_block_producer::EngineAPIBlockProducer;
use rome_sdk::rome_geth::engine::GethEngine;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::clock::Slot;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;
use std::sync::Arc;

mod api;
mod cli;
mod config;

const DEFAULT_BLOCK_LOADER_BATCH_SIZE: Slot = 128;

async fn get_app_handles<S: SolanaBlockStorage + 'static, E: EthereumBlockStorage + 'static>(
    config: &HerculesConfig,
    solana_block_storage: Arc<S>,
    ethereum_block_storage: Arc<E>,
) -> anyhow::Result<(jsonrpsee::server::ServerHandle, tokio::task::JoinHandle<()>)> {
    let program_id = Pubkey::from_str(&config.program_id)?;
    let geth_engine = GethEngine::new(&config.geth_engine);
    let (indexer_started_tx, indexer_started_rx) = tokio::sync::oneshot::channel();
    let solana_client = Arc::new(RpcClient::new_with_commitment(
        config.solana.rpc_url.to_string(),
        CommitmentConfig {
            commitment: config.solana.commitment,
        },
    ));

    let indexer = StandaloneIndexer {
        solana_client,
        commitment_level: config.solana.commitment,
        rome_evm_pubkey: program_id,
        solana_block_storage: solana_block_storage.clone(),
        ethereum_block_storage: ethereum_block_storage.clone(),
        block_producer: EngineAPIBlockProducer::new(
            Arc::new(geth_engine),
            ethers::providers::Provider::<Http>::try_from(&config.geth_api)?,
        ),
    };

    let server_jh = HerculesAdmin::new(
        solana_block_storage,
        ethereum_block_storage,
        Some(indexer_started_rx),
    )
    .start_rpc_server(config.admin_rpc)
    .await?;

    let block_loader_batch_size = config.block_loader_batch_size.unwrap_or(DEFAULT_BLOCK_LOADER_BATCH_SIZE);

    let indexer_jh = match config.mode.clone().unwrap_or(HerculesMode::Indexer) {
        HerculesMode::Indexer => indexer.start_indexing(
            config.start_slot,
            Some(indexer_started_tx),
            400,
            config.max_slot_history,
            block_loader_batch_size,
        ),
        HerculesMode::Recovery => indexer.start_recovery(
            config.start_slot.unwrap_or_else(|| panic!("start_slot is required for recovery")),
            config.end_slot,
            block_loader_batch_size,
        ),
    };

    Ok((server_jh, indexer_jh))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    tracing_subscriber::fmt().json().init();
    let config: HerculesConfig = Cli::parse().load_config().await?;

    let solana_block_storage = Arc::new(pg_storage::SolanaBlockStorage::new(
        config.solana_storage.create_pool()?,
    ));

    let chain_id = config.chain_id;
    let (_admin_server, indexer_jh) = match &config.ethereum_storage {
        EthereumStorageConfig::PgStorage(eth_pg_config) => {
            get_app_handles(
                &config,
                solana_block_storage,
                Arc::new(pg_storage::EthereumBlockStorage::new(
                    Arc::new(eth_pg_config.create_pool()?),
                    chain_id,
                )),
            )
            .await?
        }
        EthereumStorageConfig::InMemoryStorage => {
            get_app_handles(
                &config,
                solana_block_storage,
                Arc::new(inmemory::EthereumBlockStorage::new(chain_id)),
            )
            .await?
        }
    };

    // Shutdown on ctrl-c
    tokio::select! {
        res = indexer_jh => {
            bail!("Hercules Service Exited: {:?}", res);
        },
        res = signal::ctrl_c() => {
            bail!("Shutting down Hercules: {:?}", res);
        },
    }
}
