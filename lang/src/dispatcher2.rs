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
    traits2::{
        Constructor,
        FnInput,
        FnOutput,
        FnSelector,
        FnState,
        MessageMut,
        MessageRef,
    },
    DispatchError,
    Placeholder,
};
use core::{
    any::TypeId,
    mem::ManuallyDrop,
};
use ink_core::{
    env::call::{
        CallData,
        Selector,
    },
    storage2::{
        alloc,
        alloc::ContractPhase,
        traits::{
            pull_spread_root,
            push_spread_root,
        },
    },
};
use ink_primitives::Key;

/// Results of message handling operations.
pub type Result<T> = core::result::Result<T, DispatchError>;

/// Types implementing this trait can handle contract calls.
pub trait Dispatch {
    /// Dispatches the call and returns the call result.
    fn dispatch(input: &CallData) -> Result<()>;
}

/// A dispatcher that shall never dispatch.
///
/// # Note
///
/// This always comes last in a chain of dispatchers
/// and is used to break out of the dispatch routine.
#[derive(Copy, Clone, Default)]
pub struct UnreachableDispatcher;

impl Dispatch for UnreachableDispatcher {
    #[inline(always)]
    fn dispatch(_data: &CallData) -> Result<()> {
        Err(DispatchError::UnknownSelector)
    }
}

/// Types able to push another dispatcher to themselves.
pub trait PushDispatcher: Sized {
    fn push<D>(self, dispatcher: D) -> DispatchList<D, Self>;
}

impl PushDispatcher for UnreachableDispatcher {
    /// Creates a dispatch list with `dispatcher` and `self` as elements.
    #[inline(always)]
    fn push<D>(self, _dispatcher: D) -> DispatchList<D, UnreachableDispatcher> {
        Default::default()
    }
}

/// A list of dispatchers.
#[derive(Debug)]
pub struct DispatchList<Head, Rest> {
    /// Placeholder for the current dispatcher head and rest.
    placeholder: Placeholder<(Head, Rest)>,
}

impl<Head, Rest> Default for DispatchList<Head, Rest> {
    #[inline(always)]
    fn default() -> Self {
        Self {
            placeholder: Default::default(),
        }
    }
}

/// A simple type definition to view the single unreachable dispatcher as list.
pub type EmptyDispatchList = UnreachableDispatcher;

impl EmptyDispatchList {
    /// Creates a new dispatch list.
    #[inline(always)]
    pub fn empty() -> EmptyDispatchList {
        UnreachableDispatcher
    }
}

impl<Head, Rest> PushDispatcher for DispatchList<Head, Rest> {
    /// Pushes another dispatcher onto the list.
    #[inline(always)]
    fn push<D>(self, _dispatcher: D) -> DispatchList<D, Self> {
        Default::default()
    }
}

impl<Head, Rest> Dispatch for DispatchList<Head, Rest>
where
    Head: Dispatch + FnSelector,
    Rest: Dispatch,
{
    #[inline(always)]
    fn dispatch(data: &CallData) -> Result<()> {
        if <Head as FnSelector>::SELECTOR == data.selector() {
            <Head as Dispatch>::dispatch(data)
        } else {
            <Rest as Dispatch>::dispatch(data)
        }
    }
}

/// A `&self` contract message wrapper.
#[derive(Debug, Default)]
pub struct MsgRef<M>
where
    M: MessageRef,
{
    message: Placeholder<M>,
}

impl<M> FnSelector for MsgRef<M>
where
    M: MessageRef,
{
    const SELECTOR: Selector = <M as FnSelector>::SELECTOR;
}

impl<M> Dispatch for MsgRef<M>
where
    M: MessageRef,
{
    #[inline(always)]
    fn dispatch(data: &CallData) -> Result<()> {
        use scale::Decode as _;
        let args = <M as FnInput>::Input::decode(&mut &data.params()[..])
            .map_err(|_| DispatchError::InvalidParameters)?;
        alloc::initialize(ContractPhase::Call);
        let root_key = Key([0x00; 32]);
        let state =
            ManuallyDrop::new(pull_spread_root::<<M as FnState>::State>(&root_key));
        let result = <M as MessageRef>::CALLABLE(&state, args);
        alloc::finalize();
        if TypeId::of::<<M as FnOutput>::Output>() != TypeId::of::<()>() {
            ink_core::env::output::<<M as FnOutput>::Output>(&result)
        }
        Ok(())
    }
}

/// A `&mut self` contract message wrapper.
#[derive(Debug, Default)]
pub struct MsgMut<M>
where
    M: MessageMut,
{
    message: Placeholder<M>,
}

impl<M> FnSelector for MsgMut<M>
where
    M: MessageMut,
{
    const SELECTOR: Selector = <M as FnSelector>::SELECTOR;
}

impl<M> Dispatch for MsgMut<M>
where
    M: MessageMut,
{
    #[inline(always)]
    fn dispatch(data: &CallData) -> Result<()> {
        use scale::Decode as _;
        let args = <M as FnInput>::Input::decode(&mut &data.params()[..])
            .map_err(|_| DispatchError::InvalidParameters)?;
        alloc::initialize(ContractPhase::Call);
        let root_key = Key([0x00; 32]);
        let mut state =
            ManuallyDrop::new(pull_spread_root::<<M as FnState>::State>(&root_key));
        let result = <M as MessageMut>::CALLABLE(&mut state, args);
        push_spread_root::<<M as FnState>::State>(&state, &root_key);
        alloc::finalize();
        if TypeId::of::<<M as FnOutput>::Output>() != TypeId::of::<()>() {
            ink_core::env::output::<<M as FnOutput>::Output>(&result)
        }
        Ok(())
    }
}

/// A constructor contract message wrapper.
#[derive(Debug, Default)]
pub struct MsgCon<M>
where
    M: Constructor,
{
    message: Placeholder<M>,
}

impl<M> FnSelector for MsgCon<M>
where
    M: Constructor,
{
    const SELECTOR: Selector = <M as FnSelector>::SELECTOR;
}

impl<M> Dispatch for MsgCon<M>
where
    M: Constructor,
{
    #[inline(always)]
    fn dispatch(data: &CallData) -> Result<()> {
        use scale::Decode as _;
        let args = <M as FnInput>::Input::decode(&mut &data.params()[..])
            .map_err(|_| DispatchError::InvalidParameters)?;
        alloc::initialize(ContractPhase::Deploy);
        let root_key = Key([0x00; 32]);
        let state = ManuallyDrop::new(<M as Constructor>::CALLABLE(args));
        push_spread_root::<<M as FnState>::State>(&state, &root_key);
        alloc::finalize();
        Ok(())
    }
}
