use rome_sdk::Rome;
use std::{env, fs::File, io, path::Path, sync::Arc};
use tokio::signal;
use url::Url;

use self::service::RheaService;
use anyhow::bail;
use dotenv::dotenv;
use rome_sdk::rome_evm_client::indexer::indexer::Indexer;
use rome_sdk::rome_geth::engine::{config::GethEngineConfig, GethEngine};
use solana_client::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::{
    commitment_config::CommitmentLevel, signature::read_keypair_file, signer::Signer,
};

mod service;

#[derive(serde::Serialize, serde::Deserialize, Debug, Default, Clone)]
pub struct Config {
    pub chain_id: u64,
    pub solana_url: String,
    pub program_id_keypair: String,
    pub payer_keypair: String,
    pub number_holders: u64,
    pub geth_http_addr: String,
    pub geth_engine_addr: String,
    pub geth_engine_secret: String,
    pub geth_poll_interval_ms: u64,
    pub start_slot: Option<u64>,
}

pub fn load_config<T, P>(config_file: P) -> Result<T, io::Error>
where
    T: serde::de::DeserializeOwned,
    P: AsRef<Path>,
{
    let file = File::open(config_file).expect("config file not found");
    let config = serde_yaml::from_reader(file)
        .map_err(|err| io::Error::new(io::ErrorKind::Other, format!("{:?}", err)))?;
    Ok(config)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    tracing_subscriber::fmt().init();

    let config_path = env::var("RHEA_CONFIG").expect("RHEA_CONFIG is not set");
    let config: Config = load_config(config_path).expect("load config error");
    let rpc_url = Url::parse(&config.solana_url).expect("solana rpc url should be set");
    let commitment = CommitmentLevel::Confirmed;
    let program_id = read_keypair_file(Path::new(&config.program_id_keypair))
        .expect("read program_id keypair error")
        .pubkey();

    let geth_engine_config = GethEngineConfig {
        geth_engine_addr: Url::parse(&config.geth_engine_addr).expect("geth url should be set"),
        geth_engine_secret: config.geth_engine_secret,
    };
    let geth_engine =
        GethEngine::new(geth_engine_config).expect("geth_engine should be constructed");
    tracing::info!("Initialized Geth engine");

    let rome_config = rome_sdk::RomeConfig {
        solana_config: rome_sdk::rome_solana::config::SolanaConfig {
            rpc_url: rpc_url.clone(),
            commitment,
        },
        rollups: vec![(config.chain_id, program_id.to_string())]
            .into_iter()
            .collect(),
        payer_path: Path::new(&config.payer_keypair).to_path_buf(),
    };
    let rome = Rome::new_with_config(rome_config).await?;
    tracing::info!("Initialized Rome with config");

    let client = Arc::new(RpcClient::new_with_commitment(
        rpc_url,
        CommitmentConfig { commitment },
    ));

    let start_slot = config.start_slot.unwrap_or(
        client
            .get_slot()
            .expect("Failed to read slot number from solana"),
    );

    let indexer = Indexer::new(program_id, client, commitment);
    tracing::info!("Initialized Indexer");

    let rhea_service = RheaService::new(rome, geth_engine, indexer.clone())?;
    tracing::info!("Initialized RheaService");

    tokio::select! {
        res = rhea_service.start(
            start_slot,
            Url::parse(&config.geth_http_addr).expect("geth_http_addr should be set"),
            config.geth_poll_interval_ms,
        ) => {
            bail!("Rhea Service Exited: {:?}", res);
        },
        res = signal::ctrl_c() => {
            bail!("Shutting down Rhea: {:?}", res);
        },
    }
}
