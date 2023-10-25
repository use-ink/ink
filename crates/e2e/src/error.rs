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

use pallet_contracts_primitives::{
    CodeUploadResult,
    ContractExecResult,
    ContractInstantiateResult,
};

/// An error occurred while interacting with the E2E backend.
///
/// We only convey errors here that are caused by the contract's testing logic. For
/// anything concerning the execution environment (like inability to communicate with node
/// or runtime, fetch the nonce, account info, etc.) we panic.
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

impl<AccountId, Balance, CodeHash, DispatchError>
    From<ContractInstantiateResult<AccountId, Balance, ()>>
    for Error<AccountId, Balance, CodeHash, DispatchError>
{
    fn from(value: ContractInstantiateResult<AccountId, Balance, ()>) -> Self {
        Self::InstantiateDryRun(value)
    }
}

impl<AccountId, Balance, CodeHash, DispatchError> From<ContractExecResult<Balance, ()>>
    for Error<AccountId, Balance, CodeHash, DispatchError>
{
    fn from(value: ContractExecResult<Balance, ()>) -> Self {
        Self::CallDryRun(value)
    }
}

/// Dummy error type for drink!
///
/// todo: https://github.com/Cardinal-Cryptography/drink/issues/32
#[derive(Debug)]
pub struct DrinkErr;

impl<AccountId, Balance> From<ContractInstantiateResult<AccountId, Balance, ()>>
    for DrinkErr
{
    fn from(_value: ContractInstantiateResult<AccountId, Balance, ()>) -> Self {
        Self {}
    }
}

impl<Balance> From<ContractExecResult<Balance, ()>> for DrinkErr {
    fn from(_value: ContractExecResult<Balance, ()>) -> Self {
        Self {}
    }
}
