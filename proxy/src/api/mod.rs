pub mod eth;

use {
    ethers::types::{
        Address, BlockId, Bytes, FeeHistory, Transaction, TransactionReceipt, TransactionRequest,
        TxHash, H256, U256, U64,
    },
    jsonrpsee::proc_macros::rpc,
    jsonrpsee::types::{error::CALL_EXECUTION_FAILED_CODE, ErrorObjectOwned},
    rome_sdk::rome_evm_client::{
        error::RomeEvmError, indexer::ethereum_block_storage::BlockType,
        rome_evm::error::RomeProgramError,
    },
    solana_client::client_error::ClientError,
    thiserror::Error,
};

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Response failed: {0}")]
    ResponseFailed(ErrorObjectOwned),

    #[error("Rome Program Error {0}")]
    RomeProgramError(RomeProgramError),

    #[error("Rome-EVM SDK error: {0}")]
    RomeEvmError(RomeEvmError),

    #[error("Solana client error: {0}")]
    SolanaClientError(ClientError),
}

impl From<ApiError> for ErrorObjectOwned {
    fn from(e: ApiError) -> ErrorObjectOwned {
        match e {
            ApiError::ResponseFailed(e) => e,
            ApiError::RomeEvmError(RomeEvmError::Revert(mes, data)) => {
                let data_hex = format!("0x{}", hex::encode(data));
                if mes.is_empty() {
                    ErrorObjectOwned::owned(3, "execution reverted", Some(data_hex))
                } else {
                    let str = format!("execution reverted: {}", mes);
                    ErrorObjectOwned::owned(3, str, Some(data_hex))
                }
            }
            _ => ErrorObjectOwned::borrowed(CALL_EXECUTION_FAILED_CODE, "", None),
        }
    }
}

impl From<RomeEvmError> for ApiError {
    fn from(value: RomeEvmError) -> Self {
        Self::RomeEvmError(value)
    }
}

impl From<ClientError> for ApiError {
    fn from(value: ClientError) -> Self {
        Self::SolanaClientError(value)
    }
}

impl From<RomeProgramError> for ApiError {
    fn from(value: RomeProgramError) -> Self {
        Self::RomeProgramError(value)
    }
}

pub type ApiResult<T> = Result<T, ApiError>;

#[rpc(server)]
pub trait Eth {
    #[method(name = "eth_getBalance")]
    async fn eth_get_balance(&self, address: Address, block: String) -> ApiResult<U256>;
    #[method(name = "eth_chainId")]
    async fn eth_chain_id(&self) -> ApiResult<U64>;
    #[method(name = "eth_blockNumber")]
    async fn eth_block_number(&self) -> ApiResult<U64>;
    #[method(name = "eth_gasPrice")]
    async fn eth_gas_price(&self) -> ApiResult<U256>;
    #[method(name = "eth_getBlockByNumber")]
    async fn eth_get_block_by_number(
        &self,
        block_number: BlockId,
        flag: bool,
    ) -> ApiResult<Option<BlockType>>;
    #[method(name = "eth_call")]
    async fn eth_call(&self, call: TransactionRequest, block: String) -> ApiResult<Bytes>;
    #[method(name = "eth_getTransactionCount")]
    async fn eth_get_transaction_count(&self, address: Address, block: String) -> ApiResult<U64>;
    #[method(name = "eth_estimateGas")]
    async fn eth_estimate_gas(&self, call: TransactionRequest) -> ApiResult<U256>;
    #[method(name = "eth_getCode")]
    async fn eth_get_code(&self, address: Address, block: String) -> ApiResult<Bytes>;
    #[method(name = "eth_sendRawTransaction")]
    async fn eth_send_raw_transaction(&self, rlp: Bytes) -> ApiResult<TxHash>;
    #[method(name = "net_version")]
    async fn net_version(&self) -> ApiResult<U64>;
    #[method(name = "eth_getTransactionReceipt")]
    async fn eth_get_transaction_receipt(
        &self,
        hash: H256,
    ) -> ApiResult<Option<TransactionReceipt>>;
    #[method(name = "eth_getTransactionByHash")]
    async fn eth_get_transaction_by_hash(&self, hash: H256) -> ApiResult<Option<Transaction>>;
    #[method(name = "eth_feeHistory")]
    async fn eth_fee_history(
        &self,
        count: u64,
        block_number: BlockId,
        reward_percentiles: Vec<f64>,
    ) -> ApiResult<FeeHistory>;
}