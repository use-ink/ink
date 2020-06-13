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
use ink_core::{
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

/// Connector trait: Connects enum dispatcher for messages with the contract.
pub trait MessageDispatcher {
    /// The contract's message dispatcher type.
    type Type;
}

/// Connector trait: Connects enum dispatcher for constructors with the contract.
pub trait ConstructorDispatcher {
    /// The contract's constructors dispatcher type.
    type Type;
}

/// Connector trait used to start the execution of a smart contract.
///
/// The generated message and constructor dispatch enums implement this trait
/// in order to forward their already decoded state to the selected messages
/// or constructors.
pub trait Execute {
    /// Starts the smart contract execution.
    fn execute(self) -> Result<()>;
}

/// Executes the given `&self` message closure.
///
/// # Note
///
/// The closure is supposed to already contain all the arguments that the real
/// message requires and forwards them.
pub fn execute_message<M, F>(f: F) -> Result<()>
where
    M: MessageRef,
    F: FnOnce(&<M as FnState>::State) -> <M as FnOutput>::Output,
{
    // alloc::initialize(ContractPhase::Call);
    let root_key = Key::from([0x00; 32]);
    let state = ManuallyDrop::new(pull_spread_root::<<M as FnState>::State>(&root_key));
    let result = f(&state);
    // alloc::finalize();
    if TypeId::of::<<M as FnOutput>::Output>() != TypeId::of::<()>() {
        ink_core::env::output::<<M as FnOutput>::Output>(&result)
    }
    Ok(())
}

/// Executes the given `&mut self` message closure.
///
/// # Note
///
/// The closure is supposed to already contain all the arguments that the real
/// message requires and forwards them.
pub fn execute_message_mut<M, F>(f: F) -> Result<()>
where
    M: MessageMut,
    F: FnOnce(&mut <M as FnState>::State) -> <M as FnOutput>::Output,
{
    // alloc::initialize(ContractPhase::Call);
    let root_key = Key::from([0x00; 32]);
    let mut state =
        ManuallyDrop::new(pull_spread_root::<<M as FnState>::State>(&root_key));
    let result = f(&mut state);
    push_spread_root::<<M as FnState>::State>(&state, &root_key);
    // alloc::finalize();
    if TypeId::of::<<M as FnOutput>::Output>() != TypeId::of::<()>() {
        ink_core::env::output::<<M as FnOutput>::Output>(&result)
    }
    Ok(())
}

/// Executes the given constructor closure.
///
/// # Note
///
/// The closure is supposed to already contain all the arguments that the real
/// constructor message requires and forwards them.
pub fn execute_constructor<C, F>(f: F) -> Result<()>
where
    C: Constructor,
    F: FnOnce() -> <C as FnState>::State,
{
    // alloc::initialize(ContractPhase::Deploy);
    let state = ManuallyDrop::new(f());
    let root_key = Key::from([0x00; 32]);
    push_spread_root::<<C as FnState>::State>(&state, &root_key);
    // alloc::finalize();
    Ok(())
}
