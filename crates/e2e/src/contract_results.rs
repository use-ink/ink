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

use crate::H256;
use ink::codegen::ContractCallBuilder;
use ink_env::Environment;
use ink_primitives::{
    ConstructorResult,
    MessageResult,
};
use pallet_revive::{
    evm::H160,
    CodeUploadResult,
    ContractResult,
    ExecReturnValue,
    InstantiateReturnValue,
    StorageDeposit,
};
use sp_runtime::{
    DispatchError,
    Weight,
};
use std::{
    fmt,
    fmt::Debug,
    marker::PhantomData,
};
use frame_support::pallet_prelude::{Decode, Encode};
use ink_env::call::FromAddr;

/// Alias for the contract instantiate result.
pub type ContractInstantiateResultFor<E> = ContractResult<
    InstantiateReturnValue,
    <E as Environment>::Balance,
    <E as Environment>::EventRecord,
>;

/// Alias for the contract instantiate result.
pub type ContractInstantiateResultForBar<E> = ContractResultBar<
    InstantiateReturnValue,
    <E as Environment>::Balance
>;

/// Result type of a `bare_call` or `bare_instantiate` call as well as `ContractsApi::call` and
/// `ContractsApi::instantiate`.
///
/// It contains the execution result together with some auxiliary information.
///
/// #Note
///
/// It has been extended to include `events` at the end of the struct while not bumping the
/// `ContractsApi` version. Therefore when SCALE decoding a `ContractResult` its trailing data
/// should be ignored to avoid any potential compatibility issues.
#[derive(Debug, Clone, Eq, PartialEq, Encode, Decode)]
pub struct ContractResultBar<R, Balance> {
    /// How much weight was consumed during execution.
    pub gas_consumed: Weight,
    /// How much weight is required as gas limit in order to execute this call.
    ///
    /// This value should be used to determine the weight limit for on-chain execution.
    ///
    /// # Note
    ///
    /// This can only different from [`Self::gas_consumed`] when weight pre-charging
    /// is used. Currently, only `seal_call_runtime` makes use of pre-charging.
    /// Additionally, any `seal_call` or `seal_instantiate` makes use of pre-charging
    /// when a non-zero `gas_limit` argument is supplied.
    pub gas_required: Weight,
    /// How much balance was paid by the origin into the contract's deposit account in order to
    /// pay for storage.
    ///
    /// The storage deposit is never actually charged from the origin in case of [`Self::result`]
    /// is `Err`. This is because on error all storage changes are rolled back including the
    /// payment of the deposit.
    pub storage_deposit: StorageDeposit<Balance>,
    /// An optional debug message. This message is only filled when explicitly requested
    /// by the code that calls into the contract. Otherwise it is empty.
    ///
    /// The contained bytes are valid UTF-8. This is not declared as `String` because
    /// this type is not allowed within the runtime.
    ///
    /// Clients should not make any assumptions about the format of the buffer.
    /// They should just display it as-is. It is **not** only a collection of log lines
    /// provided by a contract but a formatted buffer with different sections.
    ///
    /// # Note
    ///
    /// The debug message is never generated during on-chain execution. It is reserved for
    /// RPC calls.
    pub debug_message: Vec<u8>,
    /// The execution result of the Wasm code.
    pub result: Result<R, DispatchError>,
}

/// Alias for the contract exec result.
pub type ContractExecResultFor<E> = ContractResultBar<
    ExecReturnValue,
    <E as Environment>::Balance,
>;

/*
/// Alias for the contract exec result.
pub type ContractInstantiateResultFor<E> = ContractResultBar<
    InstantiateReturnValue,
    <E as Environment>::Balance,
>;
 */


// todo can be removed
/// Copied from `pallet-revive`.
#[derive(Debug, Encode, Decode)]
pub struct BareInstantiationDryRunResult<E: Environment> {
    /// How much weight was consumed during execution.
    pub gas_consumed: Weight,
    /// How much weight is required as gas limit in order to execute this call.
    ///
    /// This value should be used to determine the weight limit for on-chain execution.
    ///
    /// # Note
    ///
    /// This can only different from [`Self::gas_consumed`] when weight pre-charging
    /// is used. Currently, only `seal_call_runtime` makes use of pre-charging.
    /// Additionally, any `seal_call` or `seal_instantiate` makes use of pre-charging
    /// when a non-zero `gas_limit` argument is supplied.
    pub gas_required: Weight,
    /// How much balance was paid by the origin into the contract's deposit account in
    /// order to pay for storage.
    ///
    /// The storage deposit is never actually charged from the origin in case of
    /// [`Self::result`] is `Err`. This is because on error all storage changes are
    /// rolled back including the payment of the deposit.
    pub storage_deposit: StorageDeposit<E::Balance>,
    /// An optional debug message. This message is only filled when explicitly requested
    /// by the code that calls into the contract. Otherwise it is empty.
    ///
    /// The contained bytes are valid UTF-8. This is not declared as `String` because
    /// this type is not allowed within the runtime.
    ///
    /// Clients should not make any assumptions about the format of the buffer.
    /// They should just display it as-is. It is **not** only a collection of log lines
    /// provided by a contract but a formatted buffer with different sections.
    ///
    /// # Note
    ///
    /// The debug message is never generated during on-chain execution. It is reserved
    /// for RPC calls.
    pub debug_message: Vec<u8>,
    /// The execution result of the Wasm code.
    pub result: Result<InstantiateReturnValue, DispatchError>,
}

/// Result of a contract instantiation using bare call.
pub struct BareInstantiationResult<EventLog> {
    /// The address at which the contract was instantiated.
    pub addr: H160,
    /// Events that happened with the contract instantiation.
    pub events: EventLog,
}

impl<EventLog> BareInstantiationResult<EventLog> {
    /// Returns the account id at which the contract was instantiated.
    pub fn call(&self) -> H160 {
        self.addr
    }
}

/// We implement a custom `Debug` here, as to avoid requiring the trait bound `Debug` for
/// `E`.
impl<EventLog> Debug for BareInstantiationResult<EventLog>
where
    EventLog: Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.debug_struct("InstantiationResult")
            .field("account_id", &self.addr)
            .field("events", &self.events)
            // todo
            .finish()
    }
}

/// Result of a contract instantiation.
pub struct InstantiationResult<E: Environment, EventLog> {
    /// The account id at which the contract was instantiated.
    pub addr: H160,
    /// The result of the dry run, contains debug messages
    /// if there were any.
    //pub dry_run: BareInstantiationDryRunResult<E>,
    pub dry_run: InstantiateDryRunResult<E>,
    /// Events that happened with the contract instantiation.
    pub events: EventLog,
}

impl<E: Environment, EventLog> InstantiationResult<E, EventLog> {
    /// Returns the account id at which the contract was instantiated.
    pub fn call_builder<Contract>(&self) -> <Contract as ContractCallBuilder>::Type
    where
        Contract: ContractCallBuilder,
        Contract::Type: FromAddr,
    {
        <<Contract as ContractCallBuilder>::Type as FromAddr>::from_addr(
            self.addr
        )
    }
}

/// We implement a custom `Debug` here, as to avoid requiring the trait bound `Debug` for
/// `E`.
impl<E: Environment, EventLog> Debug for InstantiationResult<E, EventLog>
where
    E::AccountId: Debug,
    E::Balance: Debug,
    E::EventRecord: Debug,
    EventLog: Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.debug_struct("InstantiationResult")
            .field("account_id", &self.addr)
            // .field("dry_run", &self.dry_run) // todo
            .field("events", &self.events)
            .finish()
    }
}

/// Result of a contract upload.
pub struct UploadResult<E: Environment, EventLog> {
    /// The hash with which the contract can be instantiated.
    pub code_hash: H256,
    /// The result of the dry run, contains debug messages if there were any.
    pub dry_run: CodeUploadResult<E::Balance>,
    /// Events that happened with the contract instantiation.
    pub events: EventLog,
}

/// We implement a custom `Debug` here, to avoid requiring the trait bound `Debug` for
/// `E`.
impl<E: Environment, EventLog> Debug for UploadResult<E, EventLog>
where
    E::Balance: Debug,
    H256: Debug,
    EventLog: Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.debug_struct("UploadResult")
            .field("code_hash", &self.code_hash)
            .field("dry_run", &self.dry_run)
            .field("events", &self.events)
            .finish()
    }
}

/// Result of a contract call.
pub struct CallResult<E: Environment, V, EventLog> {
    /// The result of the dry run, contains debug messages if there were any.
    pub dry_run: CallDryRunResult<E, V>,
    /// Events that happened with the contract instantiation.
    pub events: EventLog,
}

impl<E: Environment, V: scale::Decode, EventLog> CallResult<E, V, EventLog> {
    /// Returns the [`MessageResult`] from the execution of the dry-run message
    /// call.
    ///
    /// # Panics
    /// - if the dry-run message call failed to execute.
    /// - if message result cannot be decoded into the expected return value type.
    pub fn message_result(&self) -> MessageResult<V> {
        self.dry_run.message_result()
    }

    /// Returns the decoded return value of the message from the dry-run.
    ///
    /// Panics if the value could not be decoded. The raw bytes can be accessed
    /// via [`CallResult::return_data`].
    pub fn return_value(self) -> V {
        self.dry_run.return_value()
    }

    /// Returns the return value as raw bytes of the message from the dry-run.
    ///
    /// Panics if the dry-run message call failed to execute.
    pub fn return_data(&self) -> &[u8] {
        &self.dry_run.exec_return_value().data
    }

    /// Returns any debug message output by the contract decoded as UTF-8.
    pub fn debug_message(&self) -> String {
        self.dry_run.debug_message()
    }
}

// TODO(#xxx) Improve the `Debug` implementation.
impl<E: Environment, V, EventLog> Debug for CallResult<E, V, EventLog>
where
    E: Debug,
    E::Balance: Debug,
    E::EventRecord: Debug,
    V: Debug,
    EventLog: Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.debug_struct("CallResult")
            .field("dry_run", &self.dry_run)
            .field("events", &self.events)
            .finish()
    }
}

/// Result of the dry run of a contract call.
pub struct CallDryRunResult<E: Environment, V> {
    /// The result of the dry run, contains debug messages if there were any.
    pub exec_result: ContractExecResultFor<E>,
    pub _marker: PhantomData<V>,
}

/// We implement a custom `Debug` here, as to avoid requiring the trait bound `Debug` for
/// `E`.
impl<E: Environment, V> Debug for CallDryRunResult<E, V>
where
    E::Balance: Debug,
    E::EventRecord: Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.debug_struct("CallDryRunResult")
            .field("exec_result", &self.exec_result)
            .finish()
    }
}

impl<E: Environment, V: scale::Decode> CallDryRunResult<E, V> {
    /// Returns true if the dry-run execution resulted in an error.
    pub fn is_err(&self) -> bool {
        self.exec_result.result.is_err()
    }

    /// Returns the [`ExecReturnValue`] resulting from the dry-run message call.
    ///
    /// Panics if the dry-run message call failed to execute.
    pub fn exec_return_value(&self) -> &ExecReturnValue {
        self.exec_result
            .result
            .as_ref()
            .unwrap_or_else(|call_err| panic!("Call dry-run failed: {call_err:?}"))
    }

    /// Returns the [`MessageResult`] from the execution of the dry-run message call.
    ///
    /// # Panics
    /// - if the dry-run message call failed to execute.
    /// - if message result cannot be decoded into the expected return value type.
    pub fn message_result(&self) -> MessageResult<V> {
        let data = &self.exec_return_value().data;
        scale::Decode::decode(&mut data.as_ref()).unwrap_or_else(|env_err| {
            panic!(
                "Decoding dry run result to ink! message return type failed: {env_err}"
            )
        })
    }

    /// Returns the decoded return value of the message from the dry-run.
    ///
    /// Panics if the value could not be decoded. The raw bytes can be accessed via
    /// [`CallResult::return_data`].
    pub fn return_value(self) -> V {
        self.message_result()
            .unwrap_or_else(|lang_err| {
                panic!(
                    "Encountered a `LangError` while decoding dry run result to ink! message: {lang_err:?}"
                )
            })
    }

    /// Returns the return value as raw bytes of the message from the dry-run.
    ///
    /// Panics if the dry-run message call failed to execute.
    pub fn return_data(&self) -> &[u8] {
        &self.exec_return_value().data
    }

    /// Returns any debug message output by the contract decoded as UTF-8.
    pub fn debug_message(&self) -> String {
        String::from_utf8_lossy(&self.exec_result.debug_message).into()
    }
}

/// Result of the dry run of a contract call.
pub struct InstantiateDryRunResult<E: Environment> {
    /// The result of the dry run, contains debug messages if there were any.
    pub contract_result: ContractInstantiateResultForBar<E>,
}

impl<E: Environment> From<ContractInstantiateResultForBar<E>>
    for InstantiateDryRunResult<E>
{
    fn from(contract_result: ContractInstantiateResultForBar<E>) -> Self {
        Self { contract_result }
    }
}

impl<E: Environment> InstantiateDryRunResult<E> {
    /// Returns true if the dry-run execution resulted in an error.
    pub fn is_err(&self) -> bool {
        self.contract_result.result.is_err()
    }

    /// Returns the [`InstantiateReturnValue`] resulting from the dry-run message call.
    ///
    /// Panics if the dry-run message call failed to execute.
    pub fn instantiate_return_value(&self) -> &InstantiateReturnValue {
        self.contract_result
            .result
            .as_ref()
            .unwrap_or_else(|call_err| panic!("Instantiate dry-run failed: {call_err:?}"))
    }

    /// Returns the encoded return value from the constructor.
    ///
    /// # Panics
    /// - if the dry-run message instantiate failed to execute.
    /// - if message result cannot be decoded into the expected return value type.
    pub fn constructor_result<V: scale::Decode>(&self) -> ConstructorResult<V> {
        let data = &self.instantiate_return_value().result.data;
        scale::Decode::decode(&mut data.as_ref()).unwrap_or_else(|env_err| {
            panic!("Decoding dry run result to constructor return type failed: {env_err}")
        })
    }

    /// Returns any debug message output by the contract decoded as UTF-8.
    pub fn debug_message(&self) -> String {
        String::from_utf8_lossy(&self.contract_result.debug_message).into()
    }
}

impl<E> Debug for InstantiateDryRunResult<E>
where
    E: Environment,
    E::AccountId: Debug,
    E::Balance: Debug,
    E::EventRecord: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("InstantiateDryRunResult")
            .field("contract_result", &self.contract_result)
            .finish()
    }
}
