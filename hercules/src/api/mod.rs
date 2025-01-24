pub mod admin;

use jsonrpsee::proc_macros::rpc;
use jsonrpsee::types::error::CALL_EXECUTION_FAILED_CODE;
use jsonrpsee::types::ErrorObjectOwned;
use rome_sdk::rome_evm_client::error::RomeEvmError;
use solana_sdk::clock::Slot;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Response failed: {0}")]
    ResponseFailed(ErrorObjectOwned),

    #[error("Rome-EVM SDK error: {0}")]
    RomeEvmError(RomeEvmError),

    #[error("Hercules error: {0}")]
    Hercules(String),
}

impl From<ApiError> for ErrorObjectOwned {
    fn from(e: ApiError) -> ErrorObjectOwned {
        match e {
            ApiError::ResponseFailed(e) => e,
            ApiError::RomeEvmError(RomeEvmError::EmulationRevert(mes, data)) => {
                ErrorObjectOwned::owned(3, mes, Some(data))
            }
            ApiError::RomeEvmError(RomeEvmError::EmulationError(err)) => {
                ErrorObjectOwned::owned(3, err, None::<String>)
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

pub type ApiResult<T> = Result<T, ApiError>;

#[rpc(server)]
pub trait Admin {
    #[method(name = "inSync")]
    async fn in_sync(&self) -> ApiResult<bool>;

    #[method(name = "lastSolanaStorageSlot")]
    async fn last_solana_storage_slot(&self) -> ApiResult<Option<Slot>>;

    #[method(name = "lastEthereumStorageSlot")]
    async fn last_ethereum_storage_slot(&self) -> ApiResult<Option<Slot>>;
}
