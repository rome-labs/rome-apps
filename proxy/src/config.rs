use std::net::SocketAddr;
use ethers::types::Address;
use std::path::PathBuf;

use rome_sdk::rome_solana::config::SolanaConfig;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ProxyConfig {
    pub chain_id: u64,
    pub start_slot: Option<u64>,
    pub solana: SolanaConfig,
    pub program_keypair: PathBuf,
    pub payer_keypair: PathBuf,
    pub number_holders: u64,
    pub proxy_host: SocketAddr,
    pub fee_recipient: Option<Address>,
}
