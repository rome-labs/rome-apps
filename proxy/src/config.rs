use crate::proxy::Proxy;
use jsonrpsee::server::ServerHandle;
use rome_sdk::rome_evm_client::indexer::config::EthereumStorageConfig;
use rome_sdk::rome_evm_client::resources::PayerConfig;
use rome_sdk::rome_evm_client::{Payer, RomeEVMClient};
use rome_sdk::rome_solana::config::SolanaConfig;
use rome_sdk::rome_solana::indexers::clock::SolanaClockIndexer;
use rome_sdk::rome_solana::tower::SolanaTower;
use solana_sdk::clock::Slot;
use solana_sdk::pubkey::Pubkey;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use tokio::task::JoinHandle;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ProxyConfig {
    pub start_slot: Option<u64>,
    pub solana: SolanaConfig,
    pub program_id: String,
    pub chain_id: u64,
    pub payers: Vec<PayerConfig>,
    pub proxy_host: SocketAddr,
    pub max_slot_history: Option<Slot>,
    pub ethereum_storage: EthereumStorageConfig,
    pub gas_price: u128,
}

impl ProxyConfig {
    pub async fn init(self) -> anyhow::Result<(ServerHandle, JoinHandle<anyhow::Result<()>>)> {
        let rpc_client = Arc::new(self.solana.clone().into_async_client());
        let payers = Payer::from_config_list(&self.payers).await?;
        let solana_clock_indexer = SolanaClockIndexer::new(rpc_client.clone()).await?;
        let solana = SolanaTower::new(rpc_client, solana_clock_indexer.get_current_clock());
        let ethereum_block_storage = self.ethereum_storage.init()?;

        // create rome evm client
        let rome_evm_client = Arc::new(RomeEVMClient::new(
            self.chain_id,
            Pubkey::from_str(&self.program_id)?,
            solana,
            self.solana.commitment,
            ethereum_block_storage,
            payers,
            self.gas_price.into(),
        ));

        // Start the proxy server
        let server = Proxy::new(rome_evm_client.clone())
            .start_rpc_server(self.proxy_host)
            .await?;

        let join_handle = tokio::spawn(solana_clock_indexer.clone().start());
        Ok((server, join_handle))
    }
}
