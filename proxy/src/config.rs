use {
    ethers::types::Address,
    solana_sdk::commitment_config::CommitmentLevel,
    std::{fs::File, io, path::Path},
};

#[derive(serde::Serialize, serde::Deserialize, Debug, Default, Clone)]
pub struct Config {
    pub chain_id: u64,
    pub solana_url: String,
    pub commitment_level: CommitmentLevel,
    pub program_id_keypair: String,
    pub payer_keypair: String,
    pub log: String,
    pub host: String,
    pub number_holders: u64,
    pub start_slot: Option<u64>,
    pub fee_recipient: Option<Address>,
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
