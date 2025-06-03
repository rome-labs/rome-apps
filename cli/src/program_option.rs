use {
    clap::{Parser, Subcommand},
    ethers::types::{Address, U256},
    solana_sdk::pubkey::Pubkey,
};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
#[command(about = "cli application for the rome-evm program", long_about = None) ]
pub struct Cli {
    /// rome-evm program_id
    #[arg(short, long)]
    pub program_id: Pubkey,
    /// chain_id of rollup (optional, not required for get-rollup)
    #[arg(short, long)]
    pub chain_id: Option<u64>,
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
        /// path to registry_authority keypair of the rome-evm contract
        registry_authority: String,
    },
    /// Depositing funds to the rome-evm balance account.
    /// Special type 0x7E of rlp is used.
    /// Rome-evm mints the funds on the user account.
    /// SOLs are transferred from the solana user's wallet to rome-evm wallet.
    ///
    /// Rate: 1 SOL = 1 rome-evm token
    ///
    /// The amount in Wei is used as rlp.mint.
    /// This amount must be multiple of 10^9, because the precision of rome-evm token is 10^18,
    /// precision of native SOL token is 10^9.
    ///
    /// This solana transaction must be signed by solana user's wallet private key.
    Deposit {
        /// the user's address to mint a balance
        address: Address,
        /// balance in Wei to mint
        balance: u128,
        /// path to user's solana wallet keypair; the funds will be debited from this account (lamports = balance/10^9)
        keypair: String,
    },
    /// get balance
    GetBalance { address: Address },
    /// get contract code
    GetCode { address: Address },
    /// get storage slot
    GetStorageAt { address: Address, slot: U256 },
    /// get transaction count
    GetTransactionCount { address: Address },
    /// get list of registered rollups
    GetRollups,
}
