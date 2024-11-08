use crate::api::EthServer;
use anyhow::Context;
use jsonrpsee::server::{ServerBuilder, ServerHandle};
use jsonrpsee::RpcModule;

use rome_sdk::rome_evm_client::RomeEVMClient;
use std::net::SocketAddr;
use std::sync::Arc;
use rome_sdk::rome_evm_client::indexer::solana_block_storage::SolanaBlockStorage;
use rome_sdk::rome_evm_client::indexer::transaction_storage::TransactionStorage;

#[derive(Clone)]
pub struct Proxy<
    S: SolanaBlockStorage + 'static,
    T: TransactionStorage + 'static,
> {
    pub rome_evm_client: Arc<RomeEVMClient<S, T>>,
}

impl<
    S: SolanaBlockStorage + 'static,
    T: TransactionStorage + 'static
> Proxy<S, T> {
    /// Create a new instance of the [Proxy]
    pub fn new(rome_evm_client: Arc<RomeEVMClient<S, T>>) -> Self {
        Self {
            rome_evm_client,
        }
    }

    /// Start the RPC server
    pub async fn start_rpc_server(self, host: SocketAddr) -> anyhow::Result<ServerHandle> {
        tracing::info!("Starting the RPC server at {host}");

        let rpc = ServerBuilder::default()
            .build(host)
            .await
            .context("Unable to start the RPC server")?;

        let mut module = RpcModule::new(());

        // merge the rpc
        module.merge(EthServer::into_rpc(self)).unwrap();

        // start and return the rpc handle
        Ok(rpc.start(module))
    }
}
