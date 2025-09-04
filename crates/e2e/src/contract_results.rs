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

use std::{
    fmt,
    fmt::Debug,
    marker::PhantomData,
};

use frame_support::pallet_prelude::{
    Decode,
    Encode,
};
use ink::codegen::ContractCallBuilder;
use ink_env::{
    call::{
        utils::DecodeMessageResult,
        FromAddr,
    },
    Environment,
};
use ink_primitives::{
    Address,
    ConstructorResult,
    MessageResult,
    H256,
};
use pallet_revive::{
    evm::CallTrace,
    CodeUploadResult,
    ExecReturnValue,
    InstantiateReturnValue,
    StorageDeposit,
};
use sp_runtime::{
    DispatchError,
    Weight,
};

/// Alias for the contract instantiate result.
pub type ContractInstantiateResultFor<E> =
    ContractResult<InstantiateReturnValue, <E as Environment>::Balance>;

// todo use the obj one from `pallet-revive` instead
/// Result type of a `bare_call`, `bare_instantiate`, `ReviveApi::call`, and
/// `ReviveApi::instantiate`.
///
/// It contains the execution result together with some auxiliary information.
///
/// # Note
///
/// It has been extended to include `events` at the end of the struct while not bumping
/// the `ReviveApi` version. Therefore when SCALE decoding a `ContractResult` its
/// trailing data should be ignored to avoid any potential compatibility issues.
#[derive(Debug, Clone, Eq, PartialEq, Encode, Decode)]
pub struct ContractResult<R, Balance> {
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
    pub storage_deposit: StorageDeposit<Balance>,
    /// The execution result of the code.
    pub result: Result<R, DispatchError>,
}

/// Alias for the contract exec result.
pub type ContractExecResultFor<E> =
    ContractResult<ExecReturnValue, <E as Environment>::Balance>;

/// Result of a contract instantiation using bare call.
pub struct BareInstantiationResult<EventLog> {
    /// The address at which the contract was instantiated.
    pub addr: Address,
    /// Events that happened with the contract instantiation.
    pub events: EventLog,
    /// todo
    pub trace: Option<CallTrace>,
    /// todo
    pub code_hash: H256,
}

impl<EventLog> BareInstantiationResult<EventLog> {
    /// Returns the address at which the contract was instantiated.
    /// todo why this strange name? shouldn't it be `fn addr()`?
    pub fn call(&self) -> Address {
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
        // todo add missing fields
        f.debug_struct("BareInstantiationResult")
            .field("addr", &self.addr)
            .field("events", &self.events)
            .field("trace", &self.trace)
            .finish()
    }
}

/// Result of a contract instantiation.
pub struct InstantiationResult<E: Environment, EventLog, Abi> {
    /// The account id at which the contract was instantiated.
    pub addr: Address,
    /// The result of the dry run, contains debug messages
    /// if there were any.
    pub dry_run: InstantiateDryRunResult<E, Abi>,
    /// Events that happened with the contract instantiation.
    pub events: EventLog,
    /// todo
    pub trace: Option<CallTrace>,
    /// todo
    pub code_hash: H256,
}

impl<E: Environment, EventLog, Abi> InstantiationResult<E, EventLog, Abi> {
    /// Returns a call builder for the contract which was instantiated.
    ///
    /// # Note
    ///
    /// This uses the ABI used for the contract instantiation call.
    pub fn call_builder<Contract>(&self) -> <Contract as ContractCallBuilder>::Type<Abi>
    where
        Contract: ContractCallBuilder,
        <Contract as ContractCallBuilder>::Type<Abi>: FromAddr,
    {
        <<Contract as ContractCallBuilder>::Type<Abi> as FromAddr>::from_addr(self.addr)
    }

    /// Returns a call builder for the specified ABI for the contract which was
    /// instantiated.
    ///
    /// # Note
    ///
    /// This is useful for contracts that support multiple ABIs.
    pub fn call_builder_abi<Contract, CallAbi>(
        &self,
    ) -> <Contract as ContractCallBuilder>::Type<CallAbi>
    where
        Contract: ContractCallBuilder,
        <Contract as ContractCallBuilder>::Type<CallAbi>: FromAddr,
    {
        <<Contract as ContractCallBuilder>::Type<CallAbi> as FromAddr>::from_addr(
            self.addr,
        )
    }
}

/// We implement a custom `Debug` here, as to avoid requiring the trait bound `Debug` for
/// `E`.
impl<E: Environment, EventLog, Abi> Debug for InstantiationResult<E, EventLog, Abi>
where
    E::AccountId: Debug,
    E::Balance: Debug,
    E::EventRecord: Debug,
    EventLog: Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        // todo add missing fields
        f.debug_struct("InstantiationResult")
            .field("addr", &self.addr)
            .field("dry_run", &self.dry_run)
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
pub struct CallResult<E: Environment, V, EventLog, Abi> {
    /// The result of the dry run, contains debug messages if there were any.
    pub dry_run: CallDryRunResult<E, V, Abi>,
    /// Events that happened with the contract instantiation.
    pub events: EventLog,
    /// todo
    pub trace: Option<CallTrace>,
}

impl<E: Environment, V: DecodeMessageResult<Abi>, EventLog, Abi>
    CallResult<E, V, EventLog, Abi>
{
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
}

impl<E: Environment, V, EventLog, Abi> CallResult<E, V, EventLog, Abi> {
    /// Returns the return value of the message dry-run as raw bytes.
    ///
    /// Panics if the dry-run message call failed to execute.
    pub fn return_data(&self) -> &[u8] {
        &self.dry_run.exec_return_value().data
    }
}

// TODO(#xxx) Improve the `Debug` implementation.
impl<E: Environment, V, EventLog, Abi> Debug for CallResult<E, V, EventLog, Abi>
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
            .field("trace", &self.trace)
            .finish()
    }
}

/// Result of the dry run of a contract call.
pub struct CallDryRunResult<E: Environment, V, Abi> {
    /// The result of the dry run, contains debug messages if there were any.
    pub exec_result: ContractExecResultFor<E>,
    /// The execution trace (if any).
    pub trace: Option<CallTrace>,
    /// Phantom data for return type and its ABI encoding.
    pub _marker: PhantomData<(V, Abi)>,
}

/// We implement a custom `Debug` here, as to avoid requiring the trait bound `Debug` for
/// `E`.
impl<E: Environment, V, Abi> Debug for CallDryRunResult<E, V, Abi>
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

impl<E: Environment, V, Abi> CallDryRunResult<E, V, Abi> {
    /// Returns true if the dry-run execution resulted in an error.
    pub fn is_err(&self) -> bool {
        self.exec_result.result.is_err() || self.did_revert()
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

    /// Returns true if the message call reverted.
    pub fn did_revert(&self) -> bool {
        let res = self.exec_result.result.clone().expect("no result found");
        res.did_revert()
    }

    /// Returns the return value as raw bytes of the message from the dry-run.
    ///
    /// Panics if the dry-run message call failed to execute.
    pub fn return_data(&self) -> &[u8] {
        &self.exec_return_value().data
    }
}

impl<E: Environment, V: DecodeMessageResult<Abi>, Abi> CallDryRunResult<E, V, Abi> {
    /// Returns the [`MessageResult`] from the execution of the dry-run message call.
    ///
    /// # Panics
    /// - if the dry-run message call failed to execute.
    /// - if message result cannot be decoded into the expected return value type.
    pub fn message_result(&self) -> MessageResult<V> {
        let data = &self.exec_return_value().data;
        DecodeMessageResult::decode_output(data.as_ref(), self.did_revert()).unwrap_or_else(|env_err| {
            panic!(
                "Decoding dry run result to ink! message return type failed: {env_err:?} {:?}\n\nAttempt to stringify returned data: {:?}",
                self.exec_return_value(),
                String::from_utf8_lossy(&self.exec_return_value().data[..])
            )
        })
    }

    /// Returns the decoded return value of the message from the dry-run.
    ///
    /// Panics if the value could not be decoded. The raw bytes can be accessed via
    /// [`CallResult::return_data`].
    pub fn return_value(&self) -> V {
        self.message_result()
            .unwrap_or_else(|lang_err| {
                panic!(
                    "Encountered a `LangError` while decoding dry run result to ink! message: {lang_err:?}"
                )
            })
    }
}

/// Result of the dry run of a contract call.
pub struct InstantiateDryRunResult<E: Environment, Abi> {
    /// The result of the dry run, contains debug messages if there were any.
    pub contract_result: ContractInstantiateResultFor<E>,
    /// Phantom data for return type and its ABI encoding.
    pub _marker: PhantomData<Abi>,
}

impl<E: Environment, Abi> From<ContractInstantiateResultFor<E>>
    for InstantiateDryRunResult<E, Abi>
{
    fn from(contract_result: ContractInstantiateResultFor<E>) -> Self {
        Self {
            contract_result,
            _marker: PhantomData,
        }
    }
}

impl<E: Environment, Abi> InstantiateDryRunResult<E, Abi> {
    /// Returns true if the dry-run execution resulted in an error.
    pub fn is_err(&self) -> bool {
        self.contract_result.result.is_err() || self.did_revert()
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
    pub fn constructor_result<V: DecodeMessageResult<Abi>>(
        &self,
    ) -> ConstructorResult<V> {
        let data = &self.instantiate_return_value().result.data;
        DecodeMessageResult::decode_output(data.as_ref(), self.did_revert()).unwrap_or_else(|env_err| {
            panic!("Decoding dry run result to constructor return type failed: {env_err:?}")
        })
    }

    /// Returns the return value of the instantiation dry-run as raw bytes.
    ///
    /// Panics if the dry-run message call failed to execute.
    pub fn return_data(&self) -> &[u8] {
        &self.instantiate_return_value().result.data
    }

    /// Returns true if the instantiation dry-run reverted.
    pub fn did_revert(&self) -> bool {
        let res = self.instantiate_return_value().clone().result;
        res.did_revert()
    }
}

impl<E, Abi> Debug for InstantiateDryRunResult<E, Abi>
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
