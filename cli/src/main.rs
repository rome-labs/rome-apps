mod cmd;
mod program_option;

use {
    clap::Parser,
    program_option::Cli,
    rome_sdk::rome_evm_client::{indexer::inmemory, RomeEVMClient as Client},
    rome_sdk::rome_solana::{indexers::clock::SolanaClockIndexer, tower::SolanaTower},
    solana_client::nonblocking::rpc_client::RpcClient,
    solana_sdk::commitment_config::{CommitmentConfig, CommitmentLevel::Confirmed},
    std::sync::Arc,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let rpc_client = Arc::new(RpcClient::new_with_commitment(
        cli.url,
        CommitmentConfig {
            commitment: Confirmed,
        },
    ));
    let solana_clock_indexer = SolanaClockIndexer::new(rpc_client.clone())
        .await
        .expect("create solana clock indexer error");
    let clock = solana_clock_indexer.get_current_clock();
    let tower = SolanaTower::new(rpc_client, clock);

    let client = Client::new(
        cli.chain_id.unwrap(),
        cli.program_id,
        tower,
        Confirmed,
        Arc::new(inmemory::EthereumBlockStorage),
        vec![],
    );

    cmd::execute(cli.cmd, &client).await
}
