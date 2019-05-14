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

use crate::ContractState;
use ink_core::{
    env,
    storage::alloc::{
        Allocate,
        AllocateUsing,
        CellChunkAlloc,
        Initialize,
    },
};

/// Provides a safe interface to an environment given a contract state.
pub struct ExecutionEnv<State> {
    /// The environment handler.
    env_handler: EnvHandler,
    /// The contract state.
    pub state: State,
}

impl<State> AllocateUsing for ExecutionEnv<State>
where
    State: ContractState,
{
    unsafe fn allocate_using<A>(alloc: &mut A) -> Self
    where
        A: Allocate,
    {
        let env_handler = AllocateUsing::allocate_using(alloc);
        let state = AllocateUsing::allocate_using(alloc);
        Self { env_handler, state }
    }
}

impl<State> Initialize for ExecutionEnv<State>
where
    State: ContractState,
{
    type Args = ();

    fn initialize(&mut self, _: Self::Args) {
        self.env_handler.initialize(());
        self.state.try_default_initialize();
    }
}

impl<State> core::ops::Deref for ExecutionEnv<State> {
    type Target = EnvHandler;

    fn deref(&self) -> &Self::Target {
        &self.env_handler
    }
}

impl<State> core::ops::DerefMut for ExecutionEnv<State> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.env_handler
    }
}

impl<State> ExecutionEnv<State> {
    /// Splits the execution environment into shared references
    /// to the environment handler and the state.
    ///
    /// # Note
    ///
    /// This can be useful if you want to implement a message as
    /// a method of the state to make it callable from other messages.
    pub fn split(&self) -> (&EnvHandler, &State) {
        (&self.env_handler, &self.state)
    }

    /// Splits the execution environment into mutable references
    /// to the environment handler and the state.
    ///
    /// # Note
    ///
    /// This can be useful if you want to implement a message as
    /// a method of the state to make it callable from other messages.
    pub fn split_mut(&mut self) -> (&mut EnvHandler, &mut State) {
        (&mut self.env_handler, &mut self.state)
    }
}

/// The actual handler for the environment and for dynamic
/// allocations and deallocations.
pub struct EnvHandler {
    /// The dynamic allocator.
    pub dyn_alloc: CellChunkAlloc,
}

impl AllocateUsing for EnvHandler {
    unsafe fn allocate_using<A>(alloc: &mut A) -> Self
    where
        A: Allocate,
    {
        Self {
            dyn_alloc: AllocateUsing::allocate_using(alloc),
        }
    }
}

impl Initialize for EnvHandler {
    type Args = ();

    fn initialize(&mut self, _: Self::Args) {
        self.dyn_alloc.initialize(())
    }
}

impl EnvHandler {
    /// Returns the address of the current smart contract.
    pub fn address(&self) -> env::AccountId {
        env::address()
    }

    /// Returns the balance of the current smart contract.
    pub fn balance(&self) -> env::Balance {
        env::balance()
    }

    /// Returns the caller address of the current smart contract execution.
    pub fn caller(&self) -> env::AccountId {
        env::caller()
    }

    /// Returns from the current smart contract execution with the given value.
    pub unsafe fn r#return<T>(&self, val: T) -> !
    where
        T: parity_codec::Encode,
    {
        env::r#return(val)
    }

    /// Prints the given content.
    ///
    /// # Note
    ///
    /// Only usable in development (`--dev`) chains.
    pub fn println(&self, content: &str) {
        env::println(content)
    }

    /// Deposits raw event data through the Contracts module.
    pub fn deposit_raw_event(&self, event: &[u8]) {
        env::deposit_raw_event(event)
    }

    /// Returns the random seed from the latest block.
    pub fn random_seed(&self) -> env::Hash {
        env::random_seed()
    }

    /// Returns the timestamp of the latest block.
    pub fn now(&self) -> env::Moment {
        env::now()
    }
}
