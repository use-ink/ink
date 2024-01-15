// Copyright (C) Parity Technologies (UK) Ltd.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use pallet_contracts_primitives::ContractExecResult;

use std::fmt;

/// An error occurred while interacting with the E2E backend.
///
/// We only convey errors here that are caused by the contract's testing logic. For
/// anything concerning the execution environment (like inability to communicate with node
/// or runtime, fetch the nonce, account info, etc.) we panic.
#[derive(Debug, thiserror::Error)]
pub enum Error<DispatchError: fmt::Debug + fmt::Display> {
    /// No contract with the given name found in scope.
    #[error("Contract not found: {0}")]
    ContractNotFound(String),
    /// The `instantiate_with_code` dry run failed.
    #[error("Instantiate dry-run error: {0}")]
    InstantiateDryRun(DryRunError<DispatchError>),
    /// The `instantiate_with_code` extrinsic failed.
    #[error("Instantiate extrinsic error: {0}")]
    InstantiateExtrinsic(DispatchError),
    /// The `upload` dry run failed.
    #[error("Upload dry-run error: {0}")]
    UploadDryRun(DispatchError),
    /// The `upload` extrinsic failed.
    #[error("Upload extrinsic error: {0}")]
    UploadExtrinsic(DispatchError),
    /// The `call` dry run failed.
    #[error("Call dry-run error: {0}")]
    CallDryRun(DryRunError<DispatchError>),
    /// The `call` extrinsic failed.
    #[error("Call extrinsic error: {0}")]
    CallExtrinsic(DispatchError),
    /// Error fetching account balance.
    #[error("Fetching account Balance error: {0}")]
    Balance(String),
    /// Decoding failed.
    #[error("Decoding failed: {0}")]
    Decoding(String),
}

/// Error during a dry run RPC invocation.
#[derive(Debug)]
pub struct DryRunError<DispatchError: fmt::Display + fmt::Debug> {
    pub debug_message: String,
    pub error: DispatchError,
}

impl<DispatchError> fmt::Display for DryRunError<DispatchError>
where
    DispatchError: fmt::Display + fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <Self as fmt::Debug>::fmt(self, f)
    }
}

/// Dummy error type for drink!
///
/// todo: https://github.com/Cardinal-Cryptography/drink/issues/32
#[derive(Debug, thiserror::Error)]
pub struct DrinkErr;

impl fmt::Display for DrinkErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DrinkErr")
    }
}

impl<Balance> From<ContractExecResult<Balance, ()>> for DrinkErr {
    fn from(_value: ContractExecResult<Balance, ()>) -> Self {
        Self {}
    }
}
