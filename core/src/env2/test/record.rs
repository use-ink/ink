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
    },
    memory::vec::Vec,
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
    pub account_id: Vec<u8>,
    /// Recorded gas limit for the contract creation.
    pub gas_limit: u64,
    /// Recorded endowment.
    pub endowment: Vec<u8>,
    /// Recorded input data for contract creation.
    pub input_data: CallData,
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

/// Record for an emitted event.
#[derive(Debug)]
pub struct EmitEventRecord {
    /// Recorded topics of the emitted event.
    pub topics: Vec<u8>,
    /// Recorded encoding of the emitted event.
    pub data: Vec<u8>,
}
