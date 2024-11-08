use clap::Parser;
use rome_da::celestia::RomeDaClient;

use rome_sdk::Rome;
use tokio::signal;

use self::cli::Cli;
use self::config::RheaConfig;
use self::service::RheaService;
use anyhow::bail;
use dotenv::dotenv;
use rome_sdk::rome_evm_client::{
    indexer::{
        indexer::Indexer, solana_block_inmemory_storage::SolanaBlockInMemoryStorage,
    },
};
use rome_sdk::rome_geth::engine::GethEngine;
use std::sync::Arc;
use rome_sdk::rome_evm_client::{
    indexer::transaction_inmemory_storage::TransactionInMemoryStorage,
};
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

mod cli;
mod config;
mod service;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    tracing_subscriber::fmt().init();

    // get the config
    let config: RheaConfig = Cli::parse().load_config().await?;

    let program_id = Pubkey::from_str(&config.program_id).unwrap();

    // Create the Rome instance
    let rome = Rome::new_with_config(rome_sdk::RomeConfig {
        solana_config: config.solana.clone(),
        rollups: vec![(config.chain_id, program_id.to_string())]
            .into_iter()
            .collect(),
        payers: config.payers,
    })
    .await?;

    // Create the Geth Engine
    let geth_engine =
        GethEngine::new(config.geth_engine).expect("geth_engine should be constructed");

    // Get the start slot
    let start_slot = config
        .start_slot
        .unwrap_or_else(|| rome.solana().clock().get_current_slot());

    let indexer_client = if let Some(solana_indexer) = config.solana_indexer {
        Arc::new(solana_indexer.into())
    } else {
        rome.solana().client_cloned()
    };

    // Create the indexer
    let indexer = Arc::new(Indexer::new(
        program_id,
        SolanaBlockInMemoryStorage::new(),
        TransactionInMemoryStorage::new(),
        indexer_client,
        config.solana.commitment,
        config.chain_id,
    ));

    // Geth indexer
    let geth_indexer = config.geth_indexer;

    let mut da_client: Option<RomeDaClient> = None;
    if config.celestia_url.is_some() && config.celestia_token.is_some() {
        let celestia_url =
            url::Url::parse(&config.celestia_url.unwrap()).expect("celestia_url should be set");
        let celestia_token = config.celestia_token.unwrap();

        da_client = Some(RomeDaClient::new(
            celestia_url,
            "".to_string(),
            celestia_token,
            config.chain_id,
        )?);
        tracing::info!("Initialized Da client");
    }

    // Run the Rhea Service
    let rhea_service_jh = RheaService::start(
        rome,
        geth_engine,
        indexer,
        start_slot,
        geth_indexer,
        da_client,
    );

    // Shutdown on ctrl-c
    tokio::select! {
        res = rhea_service_jh => {
            bail!("Rhea Service Exited: {:?}", res);
        },
        res = signal::ctrl_c() => {
            bail!("Shutting down Rhea: {:?}", res);
        },
    }
}
