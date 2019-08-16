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
use core::marker::PhantomData;
use ink_core::{
    env::{
        self,
        CallError,
        Env,
    },
    storage::alloc::{
        Allocate,
        AllocateUsing,
        CellChunkAlloc,
        Initialize,
    },
};
use scale::{
    Decode,
    Encode as _,
};

/// Provides a safe interface to an environment given a contract state.
pub struct ExecutionEnv<State, Env> {
    /// The environment handler.
    env_handler: EnvHandler<Env>,
    /// The contract state.
    pub state: State,
}

impl<State, Env> AllocateUsing for ExecutionEnv<State, Env>
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

impl<State, Env> Initialize for ExecutionEnv<State, Env>
where
    State: ContractState,
{
    type Args = ();

    fn initialize(&mut self, _: Self::Args) {
        self.env_handler.initialize(());
        self.state.try_default_initialize();
    }
}

impl<State, Env> core::ops::Deref for ExecutionEnv<State, Env> {
    type Target = EnvHandler<Env>;

    fn deref(&self) -> &Self::Target {
        &self.env_handler
    }
}

impl<State, Env> core::ops::DerefMut for ExecutionEnv<State, Env> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.env_handler
    }
}

impl<State, Env> ExecutionEnv<State, Env> {
    /// Splits the execution environment into shared references
    /// to the environment handler and the state.
    ///
    /// # Note
    ///
    /// This can be useful if you want to implement a message as
    /// a method of the state to make it callable from other messages.
    pub fn split(&self) -> (&EnvHandler<Env>, &State) {
        (&self.env_handler, &self.state)
    }

    /// Splits the execution environment into mutable references
    /// to the environment handler and the state.
    ///
    /// # Note
    ///
    /// This can be useful if you want to implement a message as
    /// a method of the state to make it callable from other messages.
    pub fn split_mut(&mut self) -> (&mut EnvHandler<Env>, &mut State) {
        (&mut self.env_handler, &mut self.state)
    }
}

/// The actual handler for the environment and for dynamic
/// allocations and deallocations.
pub struct EnvHandler<T> {
    /// The dynamic allocator.
    pub dyn_alloc: CellChunkAlloc,
    env_marker: PhantomData<T>,
}

impl<T> AllocateUsing for EnvHandler<T> {
    unsafe fn allocate_using<A>(alloc: &mut A) -> Self
    where
        A: Allocate,
    {
        Self {
            dyn_alloc: AllocateUsing::allocate_using(alloc),
            env_marker: PhantomData,
        }
    }
}

impl<T> Initialize for EnvHandler<T> {
    type Args = ();

    fn initialize(&mut self, _: Self::Args) {
        self.dyn_alloc.initialize(())
    }
}

impl<T: Env> EnvHandler<T> {
    /// Returns the address of the current smart contract.
    pub fn address(&self) -> T::AccountId {
        T::address()
    }

    /// Returns the balance of the current smart contract.
    pub fn balance(&self) -> T::Balance {
        T::balance()
    }

    /// Returns the caller address of the current smart contract execution.
    pub fn caller(&self) -> T::AccountId {
        T::caller()
    }

    /// Returns the given data back to the caller.
    ///
    /// # Note
    ///
    /// This must be the last operation executed before returning execution back to the caller.
    pub fn return_data<V>(&self, data: V)
    where
        V: scale::Encode,
    {
        env::return_data::<V, T>(data)
    }

    /// Prints the given content.
    ///
    /// # Note
    ///
    /// Only usable in development (`--dev`) chains.
    pub fn println(&self, content: &str) {
        T::println(content)
    }

    /// Deposits raw event data through the Contracts module.
    pub fn deposit_raw_event(&self, topics: &[T::Hash], event: &[u8]) {
        T::deposit_raw_event(topics, event)
    }

    /// Returns the random seed from the latest block.
    pub fn random_seed(&self) -> T::Hash {
        T::random_seed()
    }

    /// Returns the timestamp of the latest block.
    pub fn now(&self) -> T::Moment {
        T::now()
    }

    /// Returns the latest block number.
    pub fn block_number(&self) -> T::BlockNumber {
        T::block_number()
    }

    /// Dispatches a call into the runtime.
    pub fn dispatch_call<C>(&self, call: C)
    where
        C: Into<T::Call>,
    {
        T::dispatch_raw_call(call.into().encode().as_slice())
    }

    /// Calls a remote smart contract without returning data.
    pub fn call_invoke(
        &mut self,
        callee: T::AccountId,
        gas: u64,
        value: T::Balance,
        input_data: &[u8],
    ) -> Result<(), CallError> {
        T::call_invoke(callee, gas, value, input_data)
    }

    /// Calls a remote smart contract with returning encoded data.
    pub fn call_evaluate<U: Decode>(
        &mut self,
        callee: T::AccountId,
        gas: u64,
        value: T::Balance,
        input_data: &[u8],
    ) -> Result<U, CallError> {
        T::call_evaluate(callee, gas, value, input_data)
    }
}
