// Copyright (C) Use Ink (UK) Ltd.
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

use ink_env::{
    Environment,
    call::utils::DecodeMessageResult,
};
use ink_primitives::{
    DepositLimit,
    H160,
    abi::AbiEncodeWith,
};
use jsonrpsee::core::async_trait;
use pallet_revive::evm::CallTrace;
use sp_weights::Weight;
use subxt::dynamic::Value;

use super::{
    H256,
    InstantiateDryRunResult,
    Keypair,
};
use crate::{
    CallBuilder,
    CallBuilderFinal,
    CallDryRunResult,
    UploadResult,
    backend_calls::{
        InstantiateBuilder,
        RemoveCodeBuilder,
        UploadBuilder,
    },
    builders::CreateBuilderPartial,
    contract_results::BareInstantiationResult,
};

/// Full E2E testing backend: combines general chain API and contract-specific operations.
#[async_trait]
pub trait E2EBackend<E: Environment>: ChainBackend + BuilderClient<E> {}

/// General chain operations useful in contract testing.
#[async_trait]
pub trait ChainBackend {
    /// Account type.
    type AccountId;
    /// Balance type.
    type Balance: Send + From<u32> + std::fmt::Debug;
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

    /// Returns the free balance of `account`.
    async fn free_balance(
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

    /// Attempt to transfer the `value` from `origin` to `dest`.
    ///
    /// Returns `Ok` on success, and a [`subxt::Error`] if the extrinsic is
    /// invalid (e.g. out of date nonce)
    async fn transfer_allow_death(
        &mut self,
        origin: &Keypair,
        dest: Self::AccountId,
        value: Self::Balance,
    ) -> Result<(), Self::Error>;
}

/// Contract-specific operations.
#[async_trait]
pub trait ContractsBackend<E: Environment> {
    /// Error type.
    type Error;
    /// Event log type.
    type EventLog;

    /// Start building an instantiate call using a builder pattern.
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Constructor method
    /// let mut constructor = FlipperRef::new(false);
    /// let contract = client
    ///     .instantiate("flipper", &ink_e2e::alice(), &mut constructor)
    ///     // Optional arguments
    ///     // Send 100 units with the call.
    ///     .value(100)
    ///     // Add 10% margin to the gas limit
    ///     .extra_gas_portion(10)
    ///     .storage_deposit_limit(100)
    ///     // Submit the call for on-chain execution.
    ///     .submit()
    ///     .await
    ///     .expect("instantiate failed");
    /// ```
    fn instantiate<
        'a,
        Contract: Clone,
        Args: Send + Clone + AbiEncodeWith<Abi> + Sync,
        R,
        Abi: Send + Sync + Clone,
    >(
        &'a mut self,
        contract_name: &'a str,
        caller: &'a Keypair,
        constructor: &'a mut CreateBuilderPartial<E, Contract, Args, R, Abi>,
    ) -> InstantiateBuilder<'a, E, Contract, Args, R, Self, Abi>
    where
        Self: Sized + BuilderClient<E>,
    {
        InstantiateBuilder::new(self, caller, contract_name, constructor)
    }

    /// Start building an upload call.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let contract = client
    ///     .upload("flipper", &ink_e2e::alice())
    ///     // Optional arguments
    ///     .storage_deposit_limit(100)
    ///     // Submit the call for on-chain execution.
    ///     .submit()
    ///     .await
    ///     .expect("upload failed");
    /// ```
    fn upload<'a>(
        &'a mut self,
        contract_name: &'a str,
        caller: &'a Keypair,
    ) -> UploadBuilder<'a, E, Self>
    where
        Self: Sized + BuilderClient<E>,
    {
        UploadBuilder::new(self, contract_name, caller)
    }

    /// Start building a remove code call.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let contract = client
    ///     .remove_code(&ink_e2e::alice(), code_hash)
    ///     // Submit the call for on-chain execution.
    ///     .submit()
    ///     .await
    ///     .expect("remove failed");
    /// ```
    fn remove_code<'a>(
        &'a mut self,
        caller: &'a Keypair,
        code_hash: H256,
    ) -> RemoveCodeBuilder<'a, E, Self>
    where
        Self: Sized + BuilderClient<E>,
    {
        RemoveCodeBuilder::new(self, caller, code_hash)
    }

    /// Start building a call using a builder pattern.
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Message method
    /// let get = call_builder.get();
    /// let get_res = client
    ///    .call(&ink_e2e::bob(), &get)
    ///     // Optional arguments
    ///     // Send 100 units with the call.
    ///     .value(100)
    ///     // Add 10% margin to the gas limit
    ///     .extra_gas_portion(10)
    ///     .storage_deposit_limit(100)
    ///     // Submit the call for on-chain execution.
    ///     .submit()
    ///     .await
    ///     .expect("instantiate failed");
    /// ```
    fn call<
        'a,
        Args: Sync + AbiEncodeWith<Abi> + Clone,
        RetType: Send + DecodeMessageResult<Abi>,
        Abi: Sync + Clone,
    >(
        &'a mut self,
        caller: &'a Keypair,
        message: &'a CallBuilderFinal<E, Args, RetType, Abi>,
    ) -> CallBuilder<'a, E, Args, RetType, Self, Abi>
    where
        Self: Sized + BuilderClient<E>,
    {
        CallBuilder::new(self, caller, message)
    }
}

#[async_trait]
pub trait BuilderClient<E: Environment>: ContractsBackend<E> {
    /// Executes a bare `call` for the contract at `account_id`. This function does not
    /// perform a dry-run, and the user is expected to provide the gas limit.
    ///
    /// Use it when you want to have a more precise control over submitting extrinsic.
    ///
    /// Returns when the transaction is included in a block. The return value
    /// contains all events that are associated with this transaction.
    async fn bare_call<
        Args: Sync + AbiEncodeWith<Abi> + Clone,
        RetType: Send + DecodeMessageResult<Abi>,
        Abi: Sync + Clone,
    >(
        &mut self,
        caller: &Keypair,
        message: &CallBuilderFinal<E, Args, RetType, Abi>,
        value: E::Balance,
        gas_limit: Weight,
        storage_deposit_limit: DepositLimit<E::Balance>,
    ) -> Result<(Self::EventLog, Option<CallTrace>), Self::Error>
    where
        CallBuilderFinal<E, Args, RetType, Abi>: Clone;

    /// Executes a dry-run `call`.
    ///
    /// Returns the result of the dry run, together with the decoded return value of the
    /// invoked message.
    ///
    /// Important: For an uncomplicated UX of the E2E testing environment we
    /// decided to automatically map the account in `pallet-revive`, if not
    /// yet mapped. This is a side effect, as a transaction is then issued
    /// on-chain and the user incurs costs!
    async fn bare_call_dry_run<
        Args: Sync + AbiEncodeWith<Abi> + Clone,
        RetType: Send + DecodeMessageResult<Abi>,
        Abi: Sync + Clone,
    >(
        &mut self,
        caller: &Keypair,
        message: &CallBuilderFinal<E, Args, RetType, Abi>,
        value: E::Balance,
        storage_deposit_limit: DepositLimit<E::Balance>,
    ) -> Result<CallDryRunResult<E, RetType, Abi>, Self::Error>
    where
        CallBuilderFinal<E, Args, RetType, Abi>: Clone;

    /// Executes a dry-run `call`.
    ///
    /// Returns the result of the dry run, together with the decoded return value of the
    /// invoked message.
    ///
    /// Important: For an uncomplicated UX of the E2E testing environment we
    /// decided to automatically map the account in `pallet-revive`, if not
    /// yet mapped. This is a side effect, as a transaction is then issued
    /// on-chain and the user incurs costs!
    async fn raw_call_dry_run<
        RetType: Send + DecodeMessageResult<Abi>,
        Abi: Sync + Clone,
    >(
        &mut self,
        dest: H160,
        input_data: Vec<u8>,
        value: E::Balance,
        storage_deposit_limit: DepositLimit<E::Balance>,
        signer: &Keypair,
    ) -> Result<CallDryRunResult<E, RetType, Abi>, Self::Error>;

    /// Executes a dry-run `call`.
    ///
    /// Returns the result of the dry run, together with the decoded return value of the
    /// invoked message.
    async fn raw_call(
        &mut self,
        dest: H160,
        input_data: Vec<u8>,
        value: E::Balance,
        gas_limit: Weight,
        storage_deposit_limit: DepositLimit<E::Balance>,
        signer: &Keypair,
    ) -> Result<(Self::EventLog, Option<CallTrace>), Self::Error>;

    /// Uploads the contract call.
    ///
    /// This function extracts the binary of the contract for the specified contract.
    ///
    /// Calling this function multiple times should be idempotent, the contract is
    /// newly instantiated each time using a unique salt. No existing contract
    /// instance is reused!
    async fn bare_upload(
        &mut self,
        contract_name: &str,
        caller: &Keypair,
        storage_deposit_limit: Option<E::Balance>,
    ) -> Result<UploadResult<E, Self::EventLog>, Self::Error>;

    /// Removes the code of the contract at `code_hash`.
    async fn bare_remove_code(
        &mut self,
        caller: &Keypair,
        code_hash: crate::H256,
    ) -> Result<Self::EventLog, Self::Error>;

    /// Bare instantiate call. This function does not perform a dry-run,
    /// and user is expected to provide the gas limit.
    ///
    /// Use it when you want to have a more precise control over submitting extrinsic.
    ///
    /// The function subsequently uploads and instantiates an instance of the contract.
    ///
    /// This function extracts the metadata of the contract at the file path
    /// `target/ink/$contract_name.contract`.
    ///
    /// Calling this function multiple times should be idempotent, the contract is
    /// newly instantiated each time using a unique salt. No existing contract
    /// instance is reused!
    async fn bare_instantiate<
        Contract: Clone,
        Args: Send + Sync + AbiEncodeWith<Abi> + Clone,
        R,
        Abi: Send + Sync + Clone,
    >(
        &mut self,
        code: Vec<u8>,
        caller: &Keypair,
        constructor: &mut CreateBuilderPartial<E, Contract, Args, R, Abi>,
        value: E::Balance,
        gas_limit: Weight,
        storage_deposit_limit: DepositLimit<E::Balance>,
    ) -> Result<BareInstantiationResult<E, Self::EventLog>, Self::Error>;

    async fn raw_instantiate(
        &mut self,
        code: Vec<u8>,
        caller: &Keypair,
        constructor: Vec<u8>,
        value: E::Balance,
        gas_limit: Weight,
        storage_deposit_limit: DepositLimit<E::Balance>,
    ) -> Result<BareInstantiationResult<E, Self::EventLog>, Self::Error>;

    async fn raw_instantiate_dry_run<Abi: Sync + Clone>(
        &mut self,
        code: Vec<u8>,
        caller: &Keypair,
        constructor: Vec<u8>,
        value: E::Balance,
        storage_deposit_limit: DepositLimit<E::Balance>,
    ) -> Result<InstantiateDryRunResult<E, Abi>, Self::Error>;

    async fn exec_instantiate(
        &mut self,
        signer: &Keypair,
        contract_name: &str,
        data: Vec<u8>,
        value: E::Balance,
        gas_limit: Weight,
        storage_deposit_limit: E::Balance,
    ) -> Result<BareInstantiationResult<E, Self::EventLog>, Self::Error>;

    /// Dry run contract instantiation.
    ///
    /// Important: For an uncomplicated UX of the E2E testing environment we
    /// decided to automatically map the account in `pallet-revive`, if not
    /// yet mapped. This is a side effect, as a transaction is then issued
    /// on-chain and the user incurs costs!
    async fn bare_instantiate_dry_run<
        Contract: Clone,
        Args: Send + Sync + AbiEncodeWith<Abi> + Clone,
        R,
        Abi: Send + Sync + Clone,
    >(
        &mut self,
        contract_name: &str,
        caller: &Keypair,
        constructor: &mut CreateBuilderPartial<E, Contract, Args, R, Abi>,
        value: E::Balance,
        storage_deposit_limit: DepositLimit<E::Balance>,
    ) -> Result<InstantiateDryRunResult<E, Abi>, Self::Error>;

    /// Checks if `caller` was already mapped in `pallet-revive`. If not, it will do so
    /// and return the events associated with that transaction.
    async fn map_account(
        &mut self,
        caller: &Keypair,
    ) -> Result<Option<Self::EventLog>, Self::Error>;

    /// Returns the `Environment::AccountId` for an `H160` address.
    async fn to_account_id(&mut self, addr: &H160) -> Result<E::AccountId, Self::Error>;

    fn load_code(&self, contract_name: &str) -> Vec<u8>;
}
