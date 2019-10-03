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

use crate::{
    env2::{
        call::CallData,
        CallParams,
        CreateParams,
        EmitEventParams,
        EnvTypes,
    },
    memory::vec::Vec,
};
use derive_more::From;
use scale::Encode as _;

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
    pub callee: Vec<u8>,
    /// Recorded gas limit for the contract creation.
    pub gas_limit: u64,
    /// Recorded endowment.
    pub endowment: Vec<u8>,
    /// Recorded input data for contract creation.
    pub input_data: CallData,
}

impl CallContractRecord {
    /// Creates a new record for a contract call.
    pub fn new<'a, E, C>(call_params: &'a C) -> Self
    where
        E: EnvTypes,
        C: CallParams<E>,
    {
        Self {
            callee: call_params.callee().encode(),
            gas_limit: call_params.gas_limit(),
            endowment: call_params.endowment().encode(),
            input_data: call_params.input_data().clone(),
        }
    }
}

/// A contract instantitation record.
#[derive(Debug)]
pub struct CreateContractRecord {
    /// Recorded code hash for the created contract.
    pub code_hash: Vec<u8>,
    /// Recorded gas limit for the contract creation.
    pub gas_limit: u64,
    /// Recorded endowment.
    pub endowment: Vec<u8>,
    /// Recorded input data for contract creation.
    pub input_data: CallData,
}

impl CreateContractRecord {
    /// Creates a new record for a contract instantiation.
    pub fn new<'a, E, C>(create_params: &'a C) -> Self
    where
        E: EnvTypes,
        C: CreateParams<E>,
    {
        Self {
            code_hash: create_params.code_hash().encode(),
            gas_limit: create_params.gas_limit(),
            endowment: create_params.endowment().encode(),
            input_data: create_params.input_data().clone(),
        }
    }
}

/// Record for an emitted event.
#[derive(Debug)]
pub struct EmitEventRecord {
    /// Recorded topics of the emitted event.
    pub topics: Vec<u8>,
    /// Recorded encoding of the emitted event.
    pub data: Vec<u8>,
}

impl EmitEventRecord {
    /// Creates a new record for a contract instantiation.
    pub fn new<'a, E, R>(emit_event: &'a R) -> Self
    where
        E: EnvTypes,
        R: EmitEventParams<E>,
    {
        Self {
            topics: emit_event.topics().encode(),
            data: emit_event.data().to_vec(),
        }
    }
}
