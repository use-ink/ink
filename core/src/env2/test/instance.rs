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
        call::{
            CallData,
            Selector,
        },
        test::{
            Storage,
            TypedEncoded,
            Accounts,
            types::*,
        },
    },
};

/// The instance of the test environment.
///
/// # Single Instance
///
/// This is basically the database of the actual test environment.
/// We need exactly one single instance of this type which the actual
/// `TestEnv` is going to access through `thread_local` storage.
/// Since `thread_local` storage doesn't allow for generics `TestEnvInstance`
/// needs to be `EnvTypes` agnostic.
///
/// # Type Safety
///
/// To counter the lost type safety of being `EnvTypes` agnostic
/// `TestEnvInstance` uses the `TypedEncoded` abstraction where possible
/// since it provides a small type-safe runtime-checked wrapper
/// arround the state.
///
/// # Default
///
/// The `thread_local` storage is using the `Default` implementation
/// of `TestEnvInstance` in order to initialize it thread locally.
/// However, since we are using `TypedEncoded` we need a separate initialization
/// routine to actually initialize those for their guarantees around type safe accesses.
/// To initialize `TestEnvInstance` type-safely `TestEnv` is using its `initialize_using`
/// routine which has certain constraints to the actual environmental types.
#[derive(Debug, Default)]
pub struct TestEnvInstance {
    /// The emulated contract storage.
    storage: Storage,
    /// The emulated chain state.
    state: ChainState,
    /// The most current block.
    block: Block,
    /// The current contract execution context.
    exec_context: ExecutionContext,
}

/// The emulated chain state.
///
/// This stores general information about the chain.
#[derive(Debug, Clone, Default)]
pub struct ChainState {
    /// The current gas price.
    gas_price: Balance,
}

/// A block within the emulated chain.
///
/// This stores information associated to blocks.
#[derive(Debug, Clone, Default)]
pub struct Block {
    /// The number of the block.
    number: BlockNumber,
    /// The blocktime in milliseconds.
    now_in_ms: Moment,
}

/// An execution context is opened whenever a contract is being called or instantiated.
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    /// The caller of the execution.
    caller: AccountId,
    /// The address of the called contract.
    callee: AccountId,
    /// The endowment for the call.
    endowment: Balance,
    /// The amount of gas left for further execution.
    gas_left: Balance,
    /// The raw call data for the contract execution.
    call_data: CallData,
    /// The limit of gas usage.
    ///
    /// There might be no limit thus `gas_left` is the actual limit then.
    gas_limit: Option<Balance>,
    /// The associated block for the execution.
    block: Block,
}

impl Default for ExecutionContext {
    fn default() -> Self {
        Self {
            caller: Default::default(),
            callee: Default::default(),
            endowment: Default::default(),
            gas_left: Default::default(),
            call_data: CallData::new(Selector::from([0x00; 4])),
            gas_limit: Default::default(),
            block: Default::default(),
        }
    }
}
