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

use super::Keypair;
use crate::{
    backend_calls::InstantiateBuilder,
    builders::CreateBuilderPartial,
    contract_results::BareInstantiationResult,
    CallBuilder,
    CallBuilderFinal,
    CallDryRunResult,
    InstantiationResult,
    UploadResult,
};
use ink_env::{
    DefaultEnvironment,
    Environment,
};
use jsonrpsee::core::async_trait;
use pallet_contracts_primitives::ContractInstantiateResult;
use sp_weights::Weight;
use subxt::dynamic::Value;

/// Full E2E testing backend: combines general chain API and contract-specific operations.
#[async_trait]
pub trait E2EBackend<E: Environment = DefaultEnvironment>:
    ChainBackend + ContractsBackend<E>
{
}

/// General chain operations useful in contract testing.
#[async_trait]
pub trait ChainBackend {
    /// Account type.
    type AccountId;
    /// Balance type.
    type Balance: Send + From<u32>;
    /// Error type.
    type Error;
    /// Event log type.
    type EventLog;

    /// Generate a new account and fund it with the given `amount` of tokens from the
    /// `origin`.
    async fn create_and_fund_account(
        &mut self,
        origin: &Keypair,
        amount: Self::Balance,
    ) -> Keypair;

    /// Returns the balance of `actor`.
    async fn balance(
        &mut self,
        account: Self::AccountId,
    ) -> Result<Self::Balance, Self::Error>;

    /// Executes a runtime call `call_name` for the `pallet_name`.
    /// The `call_data` is a `Vec<Value>`.
    ///
    /// Note:
    /// - `pallet_name` must be in camel case, for example `Balances`.
    /// - `call_name` must be snake case, for example `force_transfer`.
    /// - `call_data` is a `Vec<subxt::dynamic::Value>` that holds a representation of
    ///   some value.
    ///
    /// Returns when the transaction is included in a block. The return value contains all
    /// events that are associated with this transaction.
    ///
    /// Since we might run node with an arbitrary runtime, this method inherently must
    /// support dynamic calls.
    async fn runtime_call<'a>(
        &mut self,
        origin: &Keypair,
        pallet_name: &'a str,
        call_name: &'a str,
        call_data: Vec<Value>,
    ) -> Result<Self::EventLog, Self::Error>;
}

/// Contract-specific operations.
#[async_trait]
pub trait ContractsBackend<E: Environment> {
    /// Error type.
    type Error;
    /// Event log type.
    type EventLog;

    fn instantiate<'a, Contract, Args: Send + Clone + scale::Encode + Sync, R>(
        &'a mut self,
        contract_name: &'a str,
        caller: &'a Keypair,
        constructor: CreateBuilderPartial<E, Contract, Args, R>,
    ) -> InstantiateBuilder<'a, E, Contract, Args, R, Self>
    where
        Self: std::marker::Sized,
    {
        InstantiateBuilder::new(self, caller, contract_name, constructor)
    }

    async fn instantiate_with_gas_margin<
        Contract,
        Args: Send + scale::Encode + Clone,
        R,
    >(
        &mut self,
        contract_name: &str,
        caller: &Keypair,
        constructor: CreateBuilderPartial<E, Contract, Args, R>,
        value: E::Balance,
        margin: Option<u64>,
        storage_deposit_limit: Option<E::Balance>,
    ) -> Result<InstantiationResult<E, Self::EventLog>, Self::Error>;

    /// The function subsequently uploads and instantiates an instance of the contract.
    ///
    /// This function extracts the metadata of the contract at the file path
    /// `target/ink/$contract_name.contract`.
    ///
    /// Calling this function multiple times should be idempotent, the contract is
    /// newly instantiated each time using a unique salt. No existing contract
    /// instance is reused!
    async fn bare_instantiate<Contract, Args: Send + scale::Encode + Clone, R>(
        &mut self,
        contract_name: &str,
        caller: &Keypair,
        constructor: CreateBuilderPartial<E, Contract, Args, R>,
        value: E::Balance,
        gas_limit: Weight,
        storage_deposit_limit: Option<E::Balance>,
    ) -> Result<BareInstantiationResult<E, Self::EventLog>, Self::Error>;

    /// Dry run contract instantiation.
    async fn bare_instantiate_dry_run<
        Contract,
        Args: Send + Sync + scale::Encode + Clone,
        R,
    >(
        &mut self,
        contract_name: &str,
        caller: &Keypair,
        constructor: CreateBuilderPartial<E, Contract, Args, R>,
        value: E::Balance,
        storage_deposit_limit: Option<E::Balance>,
    ) -> ContractInstantiateResult<E::AccountId, E::Balance, ()>;

    /// The function subsequently uploads and instantiates an instance of the contract.
    ///
    /// This function extracts the Wasm of the contract for the specified contract.
    ///
    /// Calling this function multiple times should be idempotent, the contract is
    /// newly instantiated each time using a unique salt. No existing contract
    /// instance is reused!
    async fn upload(
        &mut self,
        contract_name: &str,
        caller: &Keypair,
        storage_deposit_limit: Option<E::Balance>,
    ) -> Result<UploadResult<E, Self::EventLog>, Self::Error>;

    /// Creates a call builder then can later be submitted.
    fn call<'a, Args: Sync + scale::Encode + Clone, RetType: Send + scale::Decode>(
        &'a mut self,
        caller: &'a Keypair,
        message: &'a CallBuilderFinal<E, Args, RetType>,
    ) -> CallBuilder<'a, E, Args, RetType, Self>
    where
        Self: std::marker::Sized,
    {
        CallBuilder::new(self, caller, message)
    }

    /// Executes a `call` for the contract at `account_id`.
    ///
    /// Returns when the transaction is included in a block. The return value
    /// contains all events that are associated with this transaction.
    async fn bare_call<
        Args: Sync + scale::Encode + Clone,
        RetType: Send + scale::Decode,
    >(
        &mut self,
        caller: &Keypair,
        message: &CallBuilderFinal<E, Args, RetType>,
        value: E::Balance,
        gas_limit: Weight,
        storage_deposit_limit: Option<E::Balance>,
    ) -> Result<Self::EventLog, Self::Error>
    where
        CallBuilderFinal<E, Args, RetType>: Clone;

    /// Executes a dry-run `call`.
    ///
    /// Returns the result of the dry run, together with the decoded return value of the
    /// invoked message.
    async fn bare_call_dry_run<
        Args: Sync + scale::Encode + Clone,
        RetType: Send + scale::Decode,
    >(
        &mut self,
        caller: &Keypair,
        message: &CallBuilderFinal<E, Args, RetType>,
        value: E::Balance,
        storage_deposit_limit: Option<E::Balance>,
    ) -> CallDryRunResult<E, RetType>
    where
        CallBuilderFinal<E, Args, RetType>: Clone;
}
