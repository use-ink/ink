// Copyright 2018-2019 Parity Technologies (UK) Ltd.
// This file is part of ink!.
//
// ink! is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// ink! is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with ink!.  If not, see <http://www.gnu.org/licenses/>.

//! Records are log entries for important events that were happening
//! on the emulated block chain.
//!
//! Records are a critical component for the emulated test environment
//! since certain operations are not possible to be emulated in its
//! current implementation, e.g. contract execution or proper
//! runtime on-chain behaviour since it's off-chain.
//!
//! For this records are stored instead of performing certain operations
//! that the user can query after or during the emulated contract execution.

use crate::{
    env2::{
        call::{
            CallData,
            CallParams,
            CreateParams,
        },
        test::{
            types::*,
            TypedEncoded,
        },
        EnvTypes,
        Topics,
    },
    memory::vec::Vec,
    storage::Key,
};
use derive_more::From;

/// A record of an event happening on the off-chain test environment.
///
/// This is useful for inspection of a contract execution.
#[derive(Debug, From)]
pub enum Record {
    /// Calls (invoke or evaluate) of contracts.
    Call(CallContractRecord),
    /// Instantiations of a contracts.
    Create(CreateContractRecord),
    /// Emitted events.
    EmitEvent(EmitEventRecord),
    /// Invokation of the runtime.
    InvokeRuntime(InvokeRuntimeRecord),
    /// Restoration of a contract.
    RestoreContract(RestoreContractRecord),
}

impl Record {
    /// Returns the contract call record if `self` is one and otherwise `None`.
    pub fn contract_call(&self) -> Option<&CallContractRecord> {
        match self {
            Record::Call(call_record) => Some(call_record),
            _ => None,
        }
    }

    /// Returns the contract instantiation record if `self` is one and otherwise `None`.
    pub fn contract_instantiation(&self) -> Option<&CreateContractRecord> {
        match self {
            Record::Create(create_record) => Some(create_record),
            _ => None,
        }
    }

    /// Returns the emitted event record if `self` is one and otherwise `None`.
    pub fn emitted_event(&self) -> Option<&EmitEventRecord> {
        match self {
            Record::EmitEvent(emitted_event) => Some(emitted_event),
            _ => None,
        }
    }

    /// Returns the runtime invokation record if `self` is one and otherwise `None`.
    pub fn runtime_invokation(&self) -> Option<&InvokeRuntimeRecord> {
        match self {
            Record::InvokeRuntime(runtime_invokation) => Some(runtime_invokation),
            _ => None,
        }
    }
}

/// A contract call record.
///
/// # Note
///
/// This can be either an invokation (no return value) or an
/// evaluation (with return value) of a contract call.
#[derive(Debug)]
pub struct CallContractRecord {
    /// Recorded code hash for the created contract.
    pub callee: AccountId,
    /// Recorded gas limit for the contract creation.
    pub gas_limit: u64,
    /// Recorded endowment.
    pub endowment: Balance,
    /// Recorded input data for contract creation.
    pub input_data: CallData,
}

impl CallContractRecord {
    /// Creates a new record for a contract call.
    pub fn new<'a, E, R>(call_params: &'a CallParams<E, R>) -> Self
    where
        E: EnvTypes,
    {
        Self {
            callee: TypedEncoded::from_origin(call_params.callee()),
            gas_limit: call_params.gas_limit(),
            endowment: TypedEncoded::from_origin(call_params.endowment()),
            input_data: call_params.input_data().clone(),
        }
    }
}

/// A contract instantitation record.
#[derive(Debug)]
pub struct CreateContractRecord {
    /// Recorded code hash for the created contract.
    pub code_hash: Hash,
    /// Recorded gas limit for the contract creation.
    pub gas_limit: u64,
    /// Recorded endowment.
    pub endowment: Balance,
    /// Recorded input data for contract creation.
    pub input_data: CallData,
}

impl CreateContractRecord {
    /// Creates a new record for a contract instantiation.
    pub fn new<'a, E, C>(create_params: &'a CreateParams<E, C>) -> Self
    where
        E: EnvTypes,
    {
        Self {
            code_hash: TypedEncoded::from_origin(create_params.code_hash()),
            gas_limit: create_params.gas_limit(),
            endowment: TypedEncoded::from_origin(create_params.endowment()),
            input_data: create_params.input_data().clone(),
        }
    }
}

/// Record for an emitted event.
#[derive(Debug)]
pub struct EmitEventRecord {
    /// Recorded topics of the emitted event.
    pub topics: Vec<Hash>,
    /// Recorded encoding of the emitted event.
    pub data: Vec<u8>,
}

impl EmitEventRecord {
    /// Creates a new record for an emitted event.
    pub fn new<Env, Event>(event: Event) -> Self
    where
        Env: EnvTypes,
        Event: Topics<Env> + scale::Encode,
    {
        Self {
            topics: event
                .topics()
                .iter()
                .map(|topic| TypedEncoded::from_origin(topic))
                .collect::<Vec<_>>(),
            data: event.encode(),
        }
    }
}

/// Record of a runtime invokation.
#[derive(Debug)]
pub struct InvokeRuntimeRecord {
    /// Since we have to be agnostic over runtimes we cannot
    /// be more precise here than use the completely generic
    /// encoded raw bytes of the runtime call.
    pub encoded: Vec<u8>,
}

impl InvokeRuntimeRecord {
    /// Creates a new record for a runtime invokation.
    pub fn new<V>(data: V) -> Self
    where
        V: Into<Vec<u8>>,
    {
        Self {
            encoded: data.into(),
        }
    }
}

/// Record of a contract restoration.
#[derive(Debug)]
pub struct RestoreContractRecord {
    /// The destination account ID.
    pub dest: AccountId,
    /// The original code hash of the contract.
    pub code_hash: Hash,
    /// The initial rent allowance for the restored contract.
    pub rent_allowance: Balance,
    /// The filtered keys for the restoration process.
    pub filtered_keys: Vec<Key>,
}

impl RestoreContractRecord {
    /// Creates a new record for a contract restoration.
    pub fn new<E>(
        dest: &<E as EnvTypes>::AccountId,
        code_hash: &<E as EnvTypes>::Hash,
        rent_allowance: &<E as EnvTypes>::Balance,
        filtered_keys: &[Key],
    ) -> Self
    where
        E: EnvTypes,
    {
        Self {
            dest: TypedEncoded::from_origin(dest),
            code_hash: TypedEncoded::from_origin(code_hash),
            rent_allowance: TypedEncoded::from_origin(rent_allowance),
            filtered_keys: filtered_keys.to_vec(),
        }
    }
}
