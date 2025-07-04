use crate::api::{B58Pubkey, EmulationAccountMeta};
use {
    super::EthServer,
    crate::{
        api::{ApiError, ApiResult},
        proxy::Proxy,
    },
    async_trait::async_trait,
    ethers::types::{
        Address, BlockId, Bytes, FeeHistory, Transaction, TransactionReceipt, TransactionRequest,
        TxHash, H256, U256, U64,
    },
    rome_sdk::rome_evm_client::indexer::BlockType,
};

#[async_trait]
impl EthServer for Proxy {
    #[tracing::instrument(name = "proxy::eth_chain_id", skip(self))]
    async fn eth_chain_id(&self) -> ApiResult<U64> {
        let result = self.rome_evm_client.chain_id();
        tracing::info!("eth_chain_id: {:?}", result);
        Ok(result.into())
    }

    #[tracing::instrument(name = "proxy::eth_block_number", skip(self))]
    async fn eth_block_number(&self) -> ApiResult<U64> {
        let result = self
            .rome_evm_client
            .block_number()
            .await
            .map_err(ApiError::RomeEvmError);
        tracing::info!("eth_block_number: {:?}", result);
        result
    }

    #[tracing::instrument(name = "proxy::eth_get_balance", skip(self), fields(address = ?address))]
    async fn eth_get_balance(&self, address: Address, _block: String) -> ApiResult<U256> {
        let result = self
            .rome_evm_client
            .get_balance(address)
            .map_err(ApiError::RomeEvmError)?;

        tracing::info!("eth_get_balance: {:?} {:?}", address, result);
        Ok(result)
    }

    #[tracing::instrument(name = "proxy::eth_gas_price", skip(self))]
    async fn eth_gas_price(&self) -> ApiResult<U256> {
        let result = self
            .rome_evm_client
            .gas_price()
            .map_err(ApiError::RomeEvmError);
        tracing::info!("eth_gas_price: {:?}", result);
        result
    }

    #[tracing::instrument(name = "proxy::eth_get_block_by_number", skip(self), fields(block_number = ?block_number))]
    async fn eth_get_block_by_number(
        &self,
        block_number: BlockId,
        full_transactions: bool,
    ) -> ApiResult<Option<BlockType>> {
        let result = self
            .rome_evm_client
            .get_block(block_number, full_transactions)
            .await
            .map_err(ApiError::RomeEvmError);
        tracing::info!("eth_get_block_by_number {:?}", result);
        result
    }

    #[tracing::instrument(name = "proxy::eth_get_block_by_hash", skip(self), fields(block_hash = %block_hash))]
    async fn eth_get_block_by_hash(
        &self,
        block_hash: H256,
        full_transactions: bool,
    ) -> ApiResult<Option<BlockType>> {
        let result = self
            .rome_evm_client
            .get_block(BlockId::Hash(block_hash), full_transactions)
            .await
            .map_err(ApiError::RomeEvmError);
        tracing::info!("eth_get_block_by_hash {:?}", block_hash);
        result
    }

    #[tracing::instrument(name = "proxy::eth_call", skip(self))]
    async fn eth_call(&self, call: TransactionRequest, _block: String) -> ApiResult<Bytes> {
        let result = self.rome_evm_client.call(&call).map_err(|e| e.into());
        tracing::info!("eth_call: {:?}", result);
        result
    }

    #[tracing::instrument(name = "proxy::eth_get_transaction_count", skip(self), fields(address = ?address))]
    async fn eth_get_transaction_count(&self, address: Address, _block: String) -> ApiResult<U64> {
        let result = self
            .rome_evm_client
            .transaction_count(address)
            .map_err(ApiError::RomeEvmError);
        tracing::info!("eth_get_transaction_count: {:?}, {:?}", address, result);
        result
    }

    #[tracing::instrument(name = "proxy::eth_estimate_gas", skip(self))]
    async fn eth_estimate_gas(&self, call: TransactionRequest) -> ApiResult<U256> {
        let result = self
            .rome_evm_client
            .estimate_gas(&call)
            .map_err(ApiError::RomeEvmError);
        tracing::info!("eth_estimate_gas: {:?}", result);
        result
    }

    #[tracing::instrument(name = "proxy::eth_get_code", skip(self), fields(address = ?address))]
    async fn eth_get_code(&self, address: Address, _block: String) -> ApiResult<Bytes> {
        let result = self
            .rome_evm_client
            .get_code(address)
            .map_err(ApiError::RomeEvmError);
        tracing::info!("eth_get_code: {:?} {:?}", address, result);
        result
    }

    #[tracing::instrument(name = "proxy::eth_send_raw_transaction", skip(self))]
    async fn eth_send_raw_transaction(&self, rlp: Bytes) -> ApiResult<TxHash> {
        let result = self
            .rome_evm_client
            .send_transaction(rlp)
            .await
            .map_err(|e| e.into());

        tracing::info!("eth_send_raw_transaction: {:?}", result);
        result
    }

    #[tracing::instrument(name = "proxy::emulate_with_payer", skip(self))]
    async fn emulate_with_payer(
        &self,
        rlp: Bytes,
        pkey: B58Pubkey,
    ) -> ApiResult<Vec<EmulationAccountMeta>> {
        let result = self
            .rome_evm_client
            .emulate_tx(rlp, pkey.0)
            .await
            .map_err(ApiError::from)?
            .into_iter()
            .map(EmulationAccountMeta::from)
            .collect::<Vec<_>>();

        tracing::info!("rome_emulate_tx: {:?}", result);
        Ok(result)
    }

    #[tracing::instrument(name = "proxy::net_version", skip(self))]
    async fn net_version(&self) -> ApiResult<U64> {
        let result = self.rome_evm_client.chain_id();
        tracing::info!("net_version: {result}");
        Ok(result.into())
    }

    #[tracing::instrument(name = "proxy::eth_get_transaction_receipt", skip(self), fields(tx_hash = ?tx_hash))]
    async fn eth_get_transaction_receipt(
        &self,
        tx_hash: H256,
    ) -> ApiResult<Option<TransactionReceipt>> {
        self.rome_evm_client
            .get_transaction_receipt(&tx_hash)
            .await
            .map_err(|err| err.into())
    }

    #[tracing::instrument(name = "proxy::eth_get_transaction_by_hash", skip(self), fields(tx_hash = ?tx_hash))]
    async fn eth_get_transaction_by_hash(&self, tx_hash: H256) -> ApiResult<Option<Transaction>> {
        self.rome_evm_client
            .get_transaction(&tx_hash)
            .await
            .map_err(|err| err.into())
    }

    #[tracing::instrument(name = "proxy::eth_fee_history", skip(self), fields(block_number = ?block_number))]
    async fn eth_fee_history(
        &self,
        count: u64,
        block_number: BlockId,
        reward_percentiles: Vec<f64>,
    ) -> ApiResult<FeeHistory> {
        let result = self
            .rome_evm_client
            .fee_history(count, block_number, reward_percentiles)
            .await
            .map_err(ApiError::from);

        tracing::info!("eth_fee_history({:?}): {:?}", block_number, result);
        result
    }

    #[tracing::instrument(name = "proxy::web3_client_version", skip(self))]
    async fn web3_client_version(&self) -> ApiResult<String> {
        Ok("proxy-version".to_string())
    }

    #[tracing::instrument(name = "proxy::eth_get_storage_at", skip(self, slot, _block), fields(address = ?address))]
    async fn eth_get_storage_at(
        &self,
        address: Address,
        slot: U256,
        _block: String,
    ) -> ApiResult<String> {
        let value = self
            .rome_evm_client
            .eth_get_storage_at(address, slot)
            .map_err(ApiError::from)?;
        let mut buf = [0_u8; 32];
        value.to_big_endian(&mut buf);
        let hex = format!("0x{}", hex::encode(buf));

        Ok(hex)
    }

    #[tracing::instrument(name = "proxy::eth_max_priority_fee_per_gas", skip(self))]
    async fn eth_max_priority_fee_per_gas(&self) -> ApiResult<U256> {
        Ok(U256::zero())
    }

    #[tracing::instrument(name = "proxy::emulate_tx", skip(self))]
    async fn emulate_tx(&self, rlp: Bytes) -> ApiResult<()> {
        let _ = self
            .rome_evm_client
            .prepare_transaction(rlp)
            .await
            .inspect_err(|err| tracing::warn!("emulate_tx error: {:?}", err))?;
        Ok(())
    }
}
