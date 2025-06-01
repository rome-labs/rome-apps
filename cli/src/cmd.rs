use {
    crate::program_option::Cmd,
    ethers::{
        prelude::{H256, U256},
        types::{
            transaction::{
                eip2718::TypedTransaction, optimism::DepositTransaction,
                request::TransactionRequest,
            },
            Signature,
        },
    },
    rome_sdk::rome_evm_client::RomeEVMClient as Client,
    solana_sdk::signature::{read_keypair_file, Signer},
    std::path::Path,
};

pub async fn execute(cmd: Cmd, client: &Client) -> anyhow::Result<()> {
    match cmd {
        Cmd::Deposit {
            address,
            balance,
            keypair,
        } => {
            let keypair =
                read_keypair_file(Path::new(&keypair)).expect("read rollup owner keypair error");

            let tx = DepositTransaction {
                tx: TransactionRequest {
                    from: Some(address),
                    to: Some(address.into()),
                    gas: Some(21000.into()),
                    gas_price: None,
                    value: Some(balance.into()),
                    data: None,
                    nonce: None,
                    chain_id: None,
                },
                source_hash: H256::zero(),
                mint: Some(balance.into()),
                is_system_tx: false,
            };

            let sig = Signature {
                r: U256::default(),
                s: U256::default(),
                v: 0,
            };

            let typed_tx: TypedTransaction = tx.clone().into();
            let rlp = typed_tx.rlp_signed(&sig);
            let pool_key = client.program_sol_wallet();

            let init_user = client
                .solana_balance(&keypair.pubkey())
                .await
                .unwrap_or_default();
            let init_pool = client.solana_balance(&pool_key).await.unwrap_or_default();

            client.deposit(rlp.as_ref(), &keypair).await?;

            let user = client.solana_balance(&keypair.pubkey()).await?;
            let pool = client.solana_balance(&pool_key).await?;
            let balance = client.get_balance(address)?;
            println!(
                "Funds have been deposited: chain_id {}, address {}, tokens: {}",
                client.chain_id(),
                address,
                tx.mint.unwrap(),
            );
            println!("user account:         {}", keypair.pubkey());
            println!("rome-evm account:     {}", pool_key);
            println!("user balance (Wei) :  {}", balance);
            println!("user (lamports):      {} -> {}", init_user, user);
            println!("rome-evm (lamports):  {} -> {}", init_pool, pool);
        }
        Cmd::RegRollup { upgrade_authority } => {
            let upgrade_authority = read_keypair_file(Path::new(&upgrade_authority))
                .expect("read upgrade_authority keypair error");

            client
                .reg_owner(client.chain_id(), &upgrade_authority)
                .await?;
            println!("chain_id {} has been registered", client.chain_id(),);
        }
        Cmd::GetBalance { address } => {
            let balance = client.get_balance(address)?;
            println!("balance: {}", balance);
        }
        Cmd::GetCode { address } => {
            let code = client.get_code(address)?;
            println!("code: {:#}", hex::encode(code.as_ref()));
        }
        Cmd::GetStorageAt { address, slot } => {
            let value = client.eth_get_storage_at(address, slot)?;
            println!("value: {}", value);
        }
        Cmd::GetTransactionCount { address } => {
            let nonce = client.transaction_count(address)?;
            println!("nonce: {}", nonce);
        }
        Cmd::GetRollups => {
            let rollups = client.get_rollups()?;
            rollups.iter().for_each(|a| println!("{:?}", a));
        }
    }

    Ok(())
}
