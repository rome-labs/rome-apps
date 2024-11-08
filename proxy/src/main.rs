mod api;
mod cli;
mod config;
mod proxy;

use std::sync::Arc;

use self::cli::Cli;
use clap::Parser;
use proxy::Proxy;
use rome_sdk::rome_evm_client::{
    indexer::solana_block_inmemory_storage::SolanaBlockInMemoryStorage,
    RomeEVMClient
};
use rome_sdk::rome_evm_client::{
    indexer::transaction_inmemory_storage::TransactionInMemoryStorage, resources::Payer
};
use rome_sdk::rome_solana::indexers::clock::SolanaClockIndexer;
use rome_sdk::rome_solana::tower::SolanaTower;
use tokio::signal;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // load any .env and init the logger
    dotenv::dotenv().ok();
    tracing_subscriber::fmt().init();

    // Parse the cli arguments and load the config
    let config = Cli::parse().load_config().await?;

    let payers = Payer::from_config_list(&config.payers).await?;

    // Crete solana rpc client
    let rpc_client = Arc::new(config.solana.clone().into_async_client());

    // solana clock indexer
    let solana_clock_indexer = SolanaClockIndexer::new(rpc_client.clone()).await?;
    // start the clock
    let solana_clock_indexer_jh = tokio::spawn(solana_clock_indexer.clone().start());

    // Parse the sync rpc client
    let solana = SolanaTower::new(rpc_client, solana_clock_indexer.get_current_clock());

    // create rome evm client
    let rome_evm_client = Arc::new(RomeEVMClient::new(
        config.chain_id,
        Pubkey::from_str(&config.program_id).unwrap(),
        solana,
        config.solana.commitment,
        SolanaBlockInMemoryStorage::new(),
        TransactionInMemoryStorage::new(),
        payers,
    ));

    // Get the start slot
    let start_slot = config.start_slot.unwrap_or_default();
    let (idx_started_oneshot, idx_started_recv) = tokio::sync::oneshot::channel();

    // Start the indexer
    let indexer_jh = {
        let rome_evm_client = rome_evm_client.clone();

        tokio::spawn(async move {
            tracing::info!("Starting indexer, waiting to catch up..");

            rome_evm_client
                .clone()
                .start_indexing(start_slot, Some(idx_started_oneshot))
                .await;
        })
    };

    tokio::select! {
        _ = idx_started_recv => {
            tracing::info!("Indexer caught up..")
        },
    }

    // Start the proxy server
    let _server = Proxy::new(rome_evm_client)
        .start_rpc_server(config.proxy_host)
        .await?;

    tokio::select! {
        res = indexer_jh => {
            anyhow::bail!("Indexer exited unexpectedly {:?}", res)
        },
        res = solana_clock_indexer_jh => {
            anyhow::bail!("Solana Tower exit.. {:?}", res)
        },
        _ = signal::ctrl_c() => {
            anyhow::bail!("Shutdown..")
        }
    }
}
