// Copyright 2018-2020 Parity Technologies (UK) Ltd.
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

use crate::{
    dispatcher2::{
        Dispatch,
        DispatchList,
        EmptyDispatchList,
        MsgCon,
        MsgMut,
        MsgRef,
    },
    traits2::{
        Constructor,
        MessageMut,
        MessageRef,
    },
    DispatchError,
};
use core::marker::PhantomData;
use ink_core::env::call::CallData;

/// The contract dispatch mode.
///
/// Tells the [`Contract::dispatch_using_mode`] routine what to dispatch for.
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum DispatchMode {
    /// Mode for instantiating a contract.
    Instantiate,
    /// Mode for calling a contract.
    Call,
}

/// Trait implemented by contracts themselves in order to provide a clean
/// interface for the C-ABI specified `call` and `create` functions to forward
/// calls to.
pub trait DispatchUsingMode {
    fn dispatch_using_mode(mode: DispatchMode) -> Result<(), DispatchError>;
}

/// Placeholder for the given type.
#[derive(Debug)]
pub struct Placeholder<T>(PhantomData<fn() -> T>);

impl<T> Default for Placeholder<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<T> Clone for Placeholder<T> {
    fn clone(&self) -> Self {
        Default::default()
    }
}

impl<T> Copy for Placeholder<T> {}

/// The contract definition.
///
/// ## Dispatch
///
/// ### Constructor Dispatch
///
/// 1. Retrieve function selector `s`.
/// 1. Find selected constructor `C` given `s`:
///     - If `C` was found:
///         1. Decode input arguments `i` according to `C`'s requirements
///         1. Initialize dynamic storage allocator for instantiation
///         1. Call `C(i)`, returns storage instance `S`
///         1. Push state of `S` to contract storage
///         1. Finalize dynamic storage allocator
///         1. Exit
///     - Otherwise:
///         1. Panic that the provided constructor selector was invalid
///
/// ### Message Dispatch
///
/// 1. Retrieve function selector `s`.
/// 1. Find selected message `M` given `s`:
///     - If `M` was found:
///         1. Decode input arguments `i` according to `M`'s requirements
///         1. Initialize dynamic storage allocator for calls
///         1. Pull storage instance `S` from contract storage
///         1. Call `M(s, i)`, returns return value `R`
///         1. If `M` has mutable access to the contract storage:
///             1. push mutated state of `S` back to contract storage
///         1. Finalize dynamic storage allocator
///         1. If `R` is not of type `()`:
///             1. return `R`
///         1. Exit
///     - Otherwise:
///         1. Panic that the provided message selector was invalid
#[derive(Debug)]
pub struct Contract<Constrs, Msgs, Phase> {
    /// The current type state of the contract.
    state: Placeholder<(Constrs, Msgs, Phase)>,
}

impl<Constrs, Msgs, Phase> Default for Contract<Constrs, Msgs, Phase> {
    #[inline(always)]
    fn default() -> Self {
        Self {
            state: Default::default(),
        }
    }
}

impl<Constrs, Msgs, Phase> Clone for Contract<Constrs, Msgs, Phase> {
    fn clone(&self) -> Self {
        Default::default()
    }
}

impl<Constrs, Msgs, Phase> Copy for Contract<Constrs, Msgs, Phase> {}

/// Phase to start the building process of a contract.
#[derive(Debug)]
pub enum EmptyPhase {}
/// Building phase of a contract construction.
#[derive(Debug)]
pub enum BuildPhase {}
/// Final phase of a fully constructed contract.
#[derive(Debug)]
pub enum FinalPhase {}

impl Contract<(), (), EmptyPhase> {
    /// Creates a new contract builder.
    #[inline(always)]
    pub fn build() -> Contract<EmptyDispatchList, EmptyDispatchList, BuildPhase> {
        Default::default()
    }
}

impl<Constrs> Contract<Constrs, EmptyDispatchList, BuildPhase> {
    /// Registers a new constructor for the contract.
    #[inline(always)]
    pub fn register_constructor<C>(
        self,
    ) -> Contract<DispatchList<MsgCon<C>, Constrs>, EmptyDispatchList, BuildPhase>
    where
        C: Constructor,
    {
        Default::default()
    }
}

impl<Constrs, Messages> Contract<Constrs, Messages, BuildPhase> {
    /// Registers a new `&self` message for the contract.
    #[inline(always)]
    pub fn register_message<M>(
        self,
    ) -> Contract<Constrs, DispatchList<MsgRef<M>, Messages>, BuildPhase>
    where
        M: MessageRef,
    {
        Default::default()
    }

    /// Registers a new `&mut self` message for the contract.
    #[inline(always)]
    pub fn register_message_mut<M>(
        self,
    ) -> Contract<Constrs, DispatchList<MsgMut<M>, Messages>, BuildPhase>
    where
        M: MessageMut,
    {
        Default::default()
    }

    /// Finalizes the construction of the contract.
    #[inline(always)]
    pub fn finalize(self) -> Contract<Constrs, Messages, FinalPhase> {
        Default::default()
    }
}

impl<Constrs, Messages> Contract<Constrs, Messages, FinalPhase>
where
    Constrs: Dispatch,
{
    /// Instantiates the contract.
    #[inline(always)]
    pub fn on_instantiate(self, call_data: &CallData) -> Result<(), DispatchError> {
        <Constrs as Dispatch>::dispatch(call_data)
    }
}

impl<Constrs, Messages> Contract<Constrs, Messages, FinalPhase>
where
    Messages: Dispatch,
{
    /// Calls a contract message.
    #[inline(always)]
    pub fn on_call(self, call_data: &CallData) -> Result<(), DispatchError> {
        <Messages as Dispatch>::dispatch(call_data)
    }
}
