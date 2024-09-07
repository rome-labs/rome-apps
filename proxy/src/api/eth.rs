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
    log::info,
    rome_sdk::rome_evm_client::indexer::{
        ethereum_block_storage::BlockType, transaction_data::TransactionData,
    },
};

#[async_trait]
impl EthServer for Proxy {
    async fn eth_get_balance(&self, address: Address, _block: String) -> ApiResult<U256> {
        let result = self
            .rome_evm_client
            .get_balance(address)
            .map_err(|err| ApiError::RomeEvmError(err))?;
        info!("eth_get_balance: {:?} {:?}", address, result);
        Ok(result)
    }

    async fn eth_chain_id(&self) -> ApiResult<U64> {
        let result = self.rome_evm_client.chain_id();
        info!("eth_chain_id: {:?}", result);
        Ok(result.into())
    }

    async fn eth_block_number(&self) -> ApiResult<U64> {
        let result = self
            .rome_evm_client
            .block_number()
            .map_err(|err| ApiError::RomeEvmError(err));
        info!("eth_block_number: {:?}", result);
        result
    }

    async fn eth_gas_price(&self) -> ApiResult<U256> {
        let result = self
            .rome_evm_client
            .gas_price()
            .map_err(|err| ApiError::RomeEvmError(err));
        info!("eth_gas_price: {:?}", result);
        result
    }

    async fn eth_get_block_by_number(
        &self,
        block_number: BlockId,
        full_transactions: bool,
    ) -> ApiResult<Option<BlockType>> {
        let result = self
            .rome_evm_client
            .get_block(block_number, full_transactions)
            .map_err(|err| ApiError::RomeEvmError(err));
        info!("eth_get_block_by_number {:?}", block_number);
        result
    }

    async fn eth_call(&self, call: TransactionRequest, _block: String) -> ApiResult<Bytes> {
        let result = self
            .rome_evm_client
            .call(call)
            .map_err(|err| ApiError::RomeEvmError(err));
        info!("eth_call: {:?}", result);
        result
    }

    async fn eth_get_transaction_count(&self, address: Address, _block: String) -> ApiResult<U64> {
        let result = self
            .rome_evm_client
            .transaction_count(address)
            .map_err(|err| ApiError::RomeEvmError(err));
        info!("eth_get_transaction_count: {:?}, {:?}", address, result);
        result
    }

    async fn eth_estimate_gas(&self, _call: TransactionRequest) -> ApiResult<U256> {
        let result = self
            .rome_evm_client
            .estimate_gas(_call)
            .map_err(|err| ApiError::RomeEvmError(err));
        info!("eth_estimate_gas: {:?}", result);
        result
    }

    async fn eth_get_code(&self, address: Address, _block: String) -> ApiResult<Bytes> {
        let result = self
            .rome_evm_client
            .get_code(address)
            .map_err(|err| ApiError::RomeEvmError(err));
        info!("eth_get_code: {:?} {:?}", address, result);
        result
    }

    async fn eth_send_raw_transaction(&self, rlp: Bytes) -> ApiResult<TxHash> {
        let result = self
            .rome_evm_client
            .send_transaction(rlp)
            .await
            .map_err(|e| e.into());
        info!("eth_send_raw_transaction: {:?}", result);
        result
    }

    async fn net_version(&self) -> ApiResult<U64> {
        let result = self.rome_evm_client.chain_id();
        info!("net_version: {result}");
        Ok(result.into())
    }

    async fn eth_get_transaction_receipt(
        &self,
        tx_hash: H256,
    ) -> ApiResult<Option<TransactionReceipt>> {
        let func = |tx_data: &TransactionData| tx_data.get_receipt().cloned();
        let result = self
            .rome_evm_client
            .process_transaction(tx_hash, func)
            .map_err(|e| e.into());

        info!("eth_get_transaction_receipt({:?}): {:?}", tx_hash, result);
        result
    }

    async fn eth_get_transaction_by_hash(&self, tx_hash: H256) -> ApiResult<Option<Transaction>> {
        let func = |tx_data: &TransactionData| tx_data.get_transaction().cloned();
        let result = self
            .rome_evm_client
            .process_transaction(tx_hash, func)
            .map_err(|e| e.into());

        info!("eth_get_transaction_by_hash({:?}): {:?}", tx_hash, result);
        result
    }

    async fn eth_fee_history(
        &self,
        count: u64,
        block_number: BlockId,
        reward_percentiles: Vec<f64>,
    ) -> ApiResult<FeeHistory> {
        let result = self
            .rome_evm_client
            .fee_history(count, block_number, reward_percentiles)
            .map_err(|e| ApiError::from(e));

        info!("eth_fee_history({:?}): {:?}", block_number, result);
        result
    }
}
