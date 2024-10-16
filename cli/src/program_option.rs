use {
    clap::{Parser, Subcommand},
    ethers::types::Address,
    solana_sdk::pubkey::Pubkey,
};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
#[command(about = "cli application for the rome-evm program", long_about = None) ]
pub struct Cli {
    /// rome-evm program_id
    #[arg(short, long)]
    pub program_id: Pubkey,
    /// chain_id of rollup
    #[arg(short, long)]
    pub chain_id: u64,
    /// URL for Solana's JSON RPC: http://localhost:8899
    #[arg(short, long)]
    pub url: String,
    /// rome-evm instruction
    #[command(subcommand)]
    pub cmd: Cmd,
}

#[derive(Subcommand, Debug)]
pub enum Cmd {
    /// registry a rollup in rome-evm contract
    RegRollup {
        /// rollup owner Pubkey
        rollup_owner: Pubkey,
        /// path to upgrade-authority keypair of the rome-evm contract
        upgrade_authority: String,
    },
    /// create balance on the address of the rollup owner; used to synchronize the initial state of rollup with the state of op-geth
    CreateBalance {
        /// the contract owner's address to mint a balance
        address: Address,
        /// balance to mint
        balance: u128,
        /// path to rollup owner keypair
        keypair: String,
    },
    /// registry the operator's gas recipient account
    RegGasRecipient {
        /// the gas recipient address of the operator
        address: Address,
        /// path to keypair of the operator
        keypair: String,
    },
}
