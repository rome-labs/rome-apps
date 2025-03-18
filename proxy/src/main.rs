mod api;
mod cli;
mod config;
mod proxy;

use std::sync::Arc;

use self::cli::Cli;
use crate::config::ProxyConfig;
use clap::Parser;
use jsonrpsee::server::ServerHandle;
use proxy::Proxy;
use rome_sdk::rome_evm_client::indexer::config::EthereumStorageConfig;
use rome_sdk::rome_evm_client::indexer::{pg_storage, EthereumBlockStorage};
use rome_sdk::rome_evm_client::{indexer::inmemory, resources::Payer, RomeEVMClient};
use rome_sdk::rome_solana::indexers::clock::SolanaClockIndexer;
use rome_sdk::rome_solana::tower::SolanaTower;
use rome_sdk::rome_solana::types::AsyncAtomicRpcClient;
use solana_sdk::clock::Slot;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;
use tokio::signal;
use tokio::task::JoinHandle;

const DEFAULT_BLOCK_LOADER_BATCH_SIZE: Slot = 128;

async fn get_app_handles<E: EthereumBlockStorage + 'static>(
    rpc_client: AsyncAtomicRpcClient,
    config: &ProxyConfig,
    ethereum_storage_config: Arc<E>,
    inmemory_idx: bool,
) -> anyhow::Result<(ServerHandle, JoinHandle<anyhow::Result<()>>)> {
    let payers = Payer::from_config_list(&config.payers).await?;
    let solana_clock_indexer = SolanaClockIndexer::new(rpc_client.clone()).await?;
    let solana = SolanaTower::new(rpc_client, solana_clock_indexer.get_current_clock());

    // create rome evm client
    let rome_evm_client = Arc::new(RomeEVMClient::new(
        Pubkey::from_str(&config.program_id)?,
        solana,
        config.solana.commitment,
        ethereum_storage_config,
        payers,
    ));

    // Start the proxy server
    let server = Proxy::new(rome_evm_client.clone())
        .start_rpc_server(config.proxy_host)
        .await?;

    let join_handle = if inmemory_idx {
        let start_slot = config.start_slot;
        let max_slot_history = config.max_slot_history;
        let block_loader_batch_size = config
            .block_loader_batch_size
            .unwrap_or(DEFAULT_BLOCK_LOADER_BATCH_SIZE);
        tokio::spawn(async move {
            tokio::select! {
                res = rome_evm_client.start_indexing(
                    Arc::new(inmemory::SolanaBlockStorage::new()),
                    start_slot,
                    None,
                    max_slot_history,
                    block_loader_batch_size,
                ) => {
                    tracing::warn!("Inmemory indexation stopped: {:?}", res);
                },
                res = tokio::spawn(solana_clock_indexer.clone().start()) => {
                    tracing::warn!("Solana clock indexer stopped: {:?}", res);
                }
            }

            Ok(())
        })
    } else {
        tokio::spawn(solana_clock_indexer.clone().start())
    };

    Ok((server, join_handle))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // load any .env and init the logger
    dotenv::dotenv().ok();
    tracing_subscriber::fmt().json().init();
    let config = Cli::parse().load_config().await?;
    let rpc_client = Arc::new(config.solana.clone().into_async_client());

    let (_server, join_handle) = match &config.ethereum_storage {
        EthereumStorageConfig::PgStorage(eth_pg_config) => {
            get_app_handles(
                rpc_client,
                &config,
                Arc::new(pg_storage::EthereumBlockStorage::new(
                    Arc::new(eth_pg_config.create_pool()?),
                    config.chain_id,
                )),
                false,
            )
            .await?
        }
        EthereumStorageConfig::InMemoryStorage => {
            get_app_handles(
                rpc_client,
                &config,
                Arc::new(inmemory::EthereumBlockStorage::new(config.chain_id)),
                true,
            )
            .await?
        }
    };

    tokio::select! {
        res = join_handle => {
            anyhow::bail!("RomeEVMClient exit.. {:?}", res)
        },
        _ = signal::ctrl_c() => {
            anyhow::bail!("Shutdown..")
        }
    }
}
