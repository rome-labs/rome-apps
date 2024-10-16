use {
    crate::program_option::Cmd,
    rome_sdk::rome_evm_client::RomeEVMClient as Client,
    solana_sdk::signature::{read_keypair_file, Signer},
    std::path::Path,
};

pub async fn execute(cmd: Cmd, client: &Client) -> anyhow::Result<()> {
    match cmd {
        Cmd::CreateBalance {
            address,
            balance,
            keypair,
        } => {
            let keypair =
                read_keypair_file(Path::new(&keypair)).expect("read rollup owner keypair error");

            client
                .create_balance(address, balance.into(), &keypair)
                .await?;
            println!(
                "balance has been created: chain_id {}, address {}, balance {}, rollup owner {}",
                client.chain_id(),
                address,
                balance,
                keypair.pubkey()
            );
        }
        Cmd::RegRollup {
            rollup_owner,
            upgrade_authority,
        } => {
            let upgrade_authority = read_keypair_file(Path::new(&upgrade_authority))
                .expect("read upgrade_authority keypair error");

            client
                .reg_owner(&rollup_owner, client.chain_id(), &upgrade_authority)
                .await?;
            println!(
                "chain_id has been registered: chain_id {}, rollup owner {}",
                client.chain_id(),
                rollup_owner
            );
        }
        Cmd::RegGasRecipient { address, keypair } => {
            let keypair =
                read_keypair_file(Path::new(&keypair)).expect("read operator keypair error");

            client.reg_gas_recipient(address, &keypair).await?;
            println!(
                "gas recipient address has been registered: chain_id {}, address {}, operator {}",
                client.chain_id(),
                address,
                keypair.pubkey()
            );
        }
    }

    Ok(())
}
