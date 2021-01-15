// Copyright 2018-2021 Parity Technologies (UK) Ltd.
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
    Constructor,
    DispatchError,
    FnOutput,
    FnState,
    MessageMut,
    MessageRef,
};
use core::{
    any::TypeId,
    mem::ManuallyDrop,
};
use ink_env::{
    Environment,
    ReturnFlags,
};
use ink_primitives::Key;
use ink_storage::{
    alloc,
    alloc::ContractPhase,
    traits::{
        pull_spread_root,
        push_spread_root,
    },
};

/// Results of message handling operations.
#[doc(hidden)]
pub type Result<T> = core::result::Result<T, DispatchError>;

/// Connector trait: Connects enum dispatcher for messages with the contract.
#[doc(hidden)]
pub trait MessageDispatcher {
    /// The contract's message dispatcher type.
    type Type;
}

/// Connector trait: Connects enum dispatcher for constructors with the contract.
#[doc(hidden)]
pub trait ConstructorDispatcher {
    /// The contract's constructors dispatcher type.
    type Type;
}

/// Connector trait used to start the execution of a smart contract.
///
/// The generated message and constructor dispatch enums implement this trait
/// in order to forward their already decoded state to the selected messages
/// or constructors.
#[doc(hidden)]
pub trait Execute {
    /// Starts the smart contract execution.
    fn execute(self) -> Result<()>;
}

/// Yields `true` if the message accepts payments.
#[derive(Copy, Clone)]
#[doc(hidden)]
pub struct AcceptsPayments(pub bool);

impl From<AcceptsPayments> for bool {
    #[inline]
    fn from(accepts_payments: AcceptsPayments) -> Self {
        accepts_payments.0
    }
}

/// Yields `true` if the dynamic storage allocator is enabled for the given call.
#[derive(Copy, Clone)]
#[doc(hidden)]
pub struct EnablesDynamicStorageAllocator(pub bool);

impl From<EnablesDynamicStorageAllocator> for bool {
    #[inline]
    fn from(enables_dynamic_storage_allocator: EnablesDynamicStorageAllocator) -> Self {
        enables_dynamic_storage_allocator.0
    }
}

/// Executes the given `&self` message closure.
///
/// # Note
///
/// The closure is supposed to already contain all the arguments that the real
/// message requires and forwards them.
#[inline]
#[doc(hidden)]
pub fn execute_message<E, M, F>(
    accepts_payments: AcceptsPayments,
    enables_dynamic_storage_allocator: EnablesDynamicStorageAllocator,
    f: F,
) -> Result<()>
where
    E: Environment,
    M: MessageRef,
    F: FnOnce(&<M as FnState>::State) -> <M as FnOutput>::Output,
{
    let accepts_payments: bool = accepts_payments.into();
    let enables_dynamic_storage_allocator: bool =
        enables_dynamic_storage_allocator.into();
    if !accepts_payments {
        deny_payment::<E>()?;
    }
    if enables_dynamic_storage_allocator {
        alloc::initialize(ContractPhase::Call);
    }
    let root_key = Key::from([0x00; 32]);
    let state = ManuallyDrop::new(pull_spread_root::<<M as FnState>::State>(&root_key));
    let result = f(&state);
    if enables_dynamic_storage_allocator {
        alloc::finalize();
    }
    if TypeId::of::<<M as FnOutput>::Output>() != TypeId::of::<()>() {
        ink_env::return_value::<<M as FnOutput>::Output>(ReturnFlags::default(), &result)
    }
    Ok(())
}

/// Returns `Ok` if the caller did not transfer additional value to the callee.
///
/// # Errors
///
/// If the caller did send some amount of transferred value to the callee.
#[inline]
#[doc(hidden)]
pub fn deny_payment<E>() -> Result<()>
where
    E: Environment,
{
    let transferred = ink_env::transferred_balance::<E>()
        .expect("encountered error while querying transferred balance");
    if transferred != <E as Environment>::Balance::from(0u32) {
        return Err(DispatchError::PaidUnpayableMessage)
    }
    Ok(())
}

/// Executes the given `&mut self` message closure.
///
/// # Note
///
/// The closure is supposed to already contain all the arguments that the real
/// message requires and forwards them.
#[inline]
#[doc(hidden)]
pub fn execute_message_mut<E, M, F>(
    accepts_payments: AcceptsPayments,
    enables_dynamic_storage_allocator: EnablesDynamicStorageAllocator,
    f: F,
) -> Result<()>
where
    E: Environment,
    M: MessageMut,
    F: FnOnce(&mut <M as FnState>::State) -> <M as FnOutput>::Output,
{
    let accepts_payments: bool = accepts_payments.into();
    let enables_dynamic_storage_allocator: bool =
        enables_dynamic_storage_allocator.into();
    if !accepts_payments {
        deny_payment::<E>()?;
    }
    if enables_dynamic_storage_allocator {
        alloc::initialize(ContractPhase::Call);
    }
    let root_key = Key::from([0x00; 32]);
    let mut state =
        ManuallyDrop::new(pull_spread_root::<<M as FnState>::State>(&root_key));
    let result = f(&mut state);
    push_spread_root::<<M as FnState>::State>(&state, &root_key);
    if enables_dynamic_storage_allocator {
        alloc::finalize();
    }
    if TypeId::of::<<M as FnOutput>::Output>() != TypeId::of::<()>() {
        ink_env::return_value::<<M as FnOutput>::Output>(ReturnFlags::default(), &result)
    }
    Ok(())
}

/// Executes the given constructor closure.
///
/// # Note
///
/// The closure is supposed to already contain all the arguments that the real
/// constructor message requires and forwards them.
#[inline]
#[doc(hidden)]
pub fn execute_constructor<C, F>(
    enables_dynamic_storage_allocator: EnablesDynamicStorageAllocator,
    f: F,
) -> Result<()>
where
    C: Constructor,
    F: FnOnce() -> <C as FnState>::State,
{
    let enables_dynamic_storage_allocator: bool =
        enables_dynamic_storage_allocator.into();
    if enables_dynamic_storage_allocator {
        alloc::initialize(ContractPhase::Deploy);
    }
    let state = ManuallyDrop::new(f());
    let root_key = Key::from([0x00; 32]);
    push_spread_root::<<C as FnState>::State>(&state, &root_key);
    if enables_dynamic_storage_allocator {
        alloc::finalize();
    }
    Ok(())
}
