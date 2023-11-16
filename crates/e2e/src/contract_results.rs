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

use ink::codegen::ContractCallBuilder;
use ink_env::{
    call::FromAccountId,
    Environment,
};
use ink_primitives::{
    ConstructorResult,
    MessageResult,
};
use pallet_contracts_primitives::{
    CodeUploadResult,
    ContractExecResult,
    ContractInstantiateResult,
    ExecReturnValue,
    InstantiateReturnValue,
};
use std::{
    fmt,
    fmt::Debug,
    marker::PhantomData,
};

/// Result of a contract instantiation using bare call.
pub struct BareInstantiationResult<E: Environment, EventLog> {
    /// The account id at which the contract was instantiated.
    pub account_id: E::AccountId,
    /// Events that happened with the contract instantiation.
    pub events: EventLog,
}

impl<E: Environment, EventLog> BareInstantiationResult<E, EventLog> {
    /// Returns the account id at which the contract was instantiated.
    pub fn call<Contract>(&self) -> <Contract as ContractCallBuilder>::Type
    where
        Contract: ContractCallBuilder,
        Contract::Type: FromAccountId<E>,
    {
        <<Contract as ContractCallBuilder>::Type as FromAccountId<E>>::from_account_id(
            self.account_id.clone(),
        )
    }
}

/// We implement a custom `Debug` here, as to avoid requiring the trait bound `Debug` for
/// `E`.
impl<E: Environment, EventLog> Debug for BareInstantiationResult<E, EventLog>
where
    E::AccountId: Debug,
    E::Balance: Debug,
    EventLog: Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.debug_struct("InstantiationResult")
            .field("account_id", &self.account_id)
            .field("events", &self.events)
            .finish()
    }
}

/// Result of a contract instantiation.
pub struct InstantiationResult<E: Environment, EventLog> {
    /// The account id at which the contract was instantiated.
    pub account_id: E::AccountId,
    /// The result of the dry run, contains debug messages
    /// if there were any.
    pub dry_run: InstantiateDryRunResult<E>,
    /// Events that happened with the contract instantiation.
    pub events: EventLog,
}

impl<E: Environment, EventLog> InstantiationResult<E, EventLog> {
    /// Returns the account id at which the contract was instantiated.
    pub fn call<Contract>(&self) -> <Contract as ContractCallBuilder>::Type
    where
        Contract: ContractCallBuilder,
        Contract::Type: FromAccountId<E>,
    {
        <<Contract as ContractCallBuilder>::Type as FromAccountId<E>>::from_account_id(
            self.account_id.clone(),
        )
    }
}

/// We implement a custom `Debug` here, as to avoid requiring the trait bound `Debug` for
/// `E`.
impl<E: Environment, EventLog> Debug for InstantiationResult<E, EventLog>
where
    E::AccountId: Debug,
    E::Balance: Debug,
    EventLog: Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.debug_struct("InstantiationResult")
            .field("account_id", &self.account_id)
            .field("dry_run", &self.dry_run)
            .field("events", &self.events)
            .finish()
    }
}

/// Result of a contract upload.
pub struct UploadResult<E: Environment, EventLog> {
    /// The hash with which the contract can be instantiated.
    pub code_hash: E::Hash,
    /// The result of the dry run, contains debug messages if there were any.
    pub dry_run: CodeUploadResult<E::Hash, E::Balance>,
    /// Events that happened with the contract instantiation.
    pub events: EventLog,
}

/// We implement a custom `Debug` here, to avoid requiring the trait bound `Debug` for
/// `E`.
impl<E: Environment, EventLog> Debug for UploadResult<E, EventLog>
where
    E::Balance: Debug,
    E::Hash: Debug,
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
#[derive(Debug)]
pub struct CallDryRunResult<E: Environment, V> {
    /// The result of the dry run, contains debug messages if there were any.
    pub exec_result: ContractExecResult<E::Balance, ()>,
    pub _marker: PhantomData<V>,
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
    pub contract_result: ContractInstantiateResult<E::AccountId, E::Balance, ()>,
}

impl<E: Environment> From<ContractInstantiateResult<E::AccountId, E::Balance, ()>>
    for InstantiateDryRunResult<E>
{
    fn from(
        contract_result: ContractInstantiateResult<E::AccountId, E::Balance, ()>,
    ) -> Self {
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
    pub fn instantiate_return_value(&self) -> &InstantiateReturnValue<E::AccountId> {
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
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("InstantiateDryRunResult")
            .field("contract_result", &self.contract_result)
            .finish()
    }
}
