use pallet_contracts_primitives::{
    CodeUploadResult, ContractExecResult, ContractInstantiateResult,
};

/// An error occurred while interacting with the Substrate node.
///
/// We only convey errors here that are caused by the contract's
/// testing logic. For anything concerning the node (like inability
/// to communicate with it, fetch the nonce, account info, etc.) we
/// panic.
#[derive(Debug)]
pub enum Error<AccountId, Balance, CodeHash, DispatchError> {
    /// No contract with the given name found in scope.
    ContractNotFound(String),
    /// The `instantiate_with_code` dry run failed.
    InstantiateDryRun(ContractInstantiateResult<AccountId, Balance, ()>),
    /// The `instantiate_with_code` extrinsic failed.
    InstantiateExtrinsic(DispatchError),
    /// The `upload` dry run failed.
    UploadDryRun(CodeUploadResult<CodeHash, Balance>),
    /// The `upload` extrinsic failed.
    UploadExtrinsic(DispatchError),
    /// The `call` dry run failed.
    CallDryRun(ContractExecResult<Balance, ()>),
    /// The `call` extrinsic failed.
    CallExtrinsic(DispatchError),
    /// Error fetching account balance.
    Balance(String),
    /// Decoding failed.
    Decoding(String),
}
