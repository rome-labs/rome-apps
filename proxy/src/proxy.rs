use jsonrpsee::tracing;
use {
    crate::config::Config,
    log::info,
    rome_sdk::rome_evm_client::RomeEVMClient,
    solana_client::rpc_client::RpcClient,
    solana_sdk::{
        commitment_config::{CommitmentConfig, CommitmentLevel},
        signature::{read_keypair_file, Signer},
    },
    std::{path::Path, sync::Arc},
    tokio_util::sync::CancellationToken,
};

#[derive(Clone)]
pub struct Proxy {
    pub rome_evm_client: Arc<RomeEVMClient>,
}

impl Proxy {
    pub async fn new(config: Config, token: CancellationToken) -> Self {
        let program_id = read_keypair_file(Path::new(&config.program_id_keypair))
            .expect("read program_id keypair error")
            .pubkey();
        let payer = Arc::new(
            read_keypair_file(Path::new(&config.payer_keypair)).expect("read payer keypair error"),
        );
        let client = Arc::new(RpcClient::new_with_commitment(
            &config.solana_url,
            CommitmentConfig {
                commitment: CommitmentLevel::Confirmed,
            },
        ));

        info!("Rome-EVM Program id: {:?}", program_id);
        info!("Proxy operator wallet: {:?}", payer.pubkey());

        Self {
            rome_evm_client: Arc::new(RomeEVMClient::new(
                config.chain_id,
                program_id,
                payer,
                client,
                config.number_holders,
                config.commitment_level,
                token,
            )),
        }
    }

    pub async fn start(self, start_slot: u64) {
        self.rome_evm_client
            .start(start_slot, || tracing::info!("Proxy started"))
            .await;
    }
}
