use clap::Parser;
use rome_da::celestia::RomeDaClient;
use rome_sdk::rome_solana::payer::SolanaKeyPayer;

use rome_sdk::Rome;
use tokio::signal;

use self::cli::Cli;
use self::config::RheaConfig;
use self::service::RheaService;
use anyhow::bail;
use dotenv::dotenv;
use rome_sdk::rome_evm_client::{indexer::indexer::Indexer, RomeEVMClient as Client};
use rome_sdk::rome_geth::engine::GethEngine;
use rome_sdk::rome_solana::indexers::clock::SolanaClockIndexer;
use rome_sdk::rome_solana::tower::SolanaTower;
use solana_sdk::signer::Signer;
use std::sync::Arc;

mod cli;
mod config;
mod service;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    tracing_subscriber::fmt().init();

    // get the config
    let config: RheaConfig = Cli::parse().load_config().await?;

    // get the program id
    let program_id = SolanaKeyPayer::read_from_file(&config.program_keypair)
        .await?
        .pubkey();

    // Create the Rome instance
    let rome = Rome::new_with_config(rome_sdk::RomeConfig {
        solana_config: config.solana.clone(),
        rollups: vec![(config.chain_id, program_id.to_string())]
            .into_iter()
            .collect(),
        payer_path: config.payer_keypair.clone(),
        holder_count: config.number_holders,
    })
    .await?;

    // Create the Geth Engine
    let geth_engine =
        GethEngine::new(config.geth_engine).expect("geth_engine should be constructed");

    // Get the start slot
    let start_slot = config
        .start_slot
        .unwrap_or_else(|| rome.solana().clock().get_current_slot());

    // Create the indexer
    let indexer = Arc::new(Indexer::new(
        program_id,
        rome.solana().client_cloned(),
        config.solana.commitment,
        config.chain_id,
    ));

    // Register the gas recipient address
    if let Some(fee_recipient) = config.fee_recipient {
        // Crete solana rpc client
        let rpc_client = Arc::new(config.solana.clone().into_async_client());
        // solana clock indexer
        let solana_clock_indexer = SolanaClockIndexer::new(rpc_client.clone()).await?;
        // Parse the sync rpc client
        let solana = SolanaTower::new(rpc_client, solana_clock_indexer.get_current_clock());
        // Parse the payer keypair
        let payer = SolanaKeyPayer::read_from_file(&config.payer_keypair)
            .await?
            .into_keypair()
            .into();
        // create rome evm client
        let client = Arc::new(Client::new(
            config.chain_id,
            program_id,
            solana,
            config.solana.commitment,
        ));
        client
            .reg_gas_recipient(fee_recipient, &payer)
            .await
            .map_err(|e| tracing::error!("Failed to register gas recipient address: {}", e))
            .unwrap();
    }

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
