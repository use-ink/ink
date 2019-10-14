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
    Constructor,
    DispatchError,
    DispatchResult,
    DispatchRetCode,
    FnInput,
    FnOutput,
    Message,
};
use core::any::TypeId;
use ink_core::{
    env2::{
        call::CallData,
        Env,
        DynEnv,
        EnvAccess,
        EnvAccessMut,
    },
    storage::Flush,
};
use scale::Decode as _;

/// The mode for the dispatch routine.
#[derive(PartialEq, Eq)]
pub enum DispatchMode {
    /// Dispatch for contract instantiation.
    Instantiate,
    /// Dispatch for a contract message call.
    Call,
}

/// Dispatchable contracts implement this trait.
///
/// # Dev Note
///
/// Implemented on the storage struct of a contract.
/// Allows for dispatching on a modus for future proofing and
/// to allow automatic implementors to scope the entire implementation
/// and all associated trait implementations under a single function.
pub trait Dispatch {
    /// Dispatches for the given mode.
    ///
    /// # Note
    ///
    /// Available modes are `Instantiation` or `Call`:
    ///
    /// - `Instantiation`
    ///
    /// Executes a contract constructor
    /// that sets up the constract storage for later usage.
    /// Returns the account ID for the instantiated
    /// contract upon success.
    ///
    /// - `Call`
    ///
    /// Calls a constract message that may or may not mutate the
    /// contract storage. Also some calls may return a value back
    /// to the caller.
    fn dispatch(mode: DispatchMode) -> DispatchRetCode;
}

/// Allows to directly access the environment read-only.
///
/// # Note
///
/// This is generally implemented for storage structs that include
/// their environment in order to allow the different dispatch functions
/// to use it for returning the contract's output.
pub trait AccessEnv {
    /// The environment accessor.
    ///
    /// # Note
    ///
    /// This can be any of `ink_core::env::DynEnv` or `ink_core::env::EnvAccessMut`.
    /// The set of possible types may be extended in the future.
    type Target;

    /// Returns an immutable access to the environment.
    fn env(&self) -> &Self::Target;
}

/// Allows to directly access the environment mutably.
///
/// # Note
///
/// This is generally implemented for storage structs that include
/// their environment in order to allow the different dispatch functions
/// to use it for returning the contract's output.
pub trait AccessEnvMut: AccessEnv {
    /// Returns a mutable access to the environment.
    fn env_mut(&mut self) -> &mut Self::Target;
}

impl<E> AccessEnv for DynEnv<E> {
    type Target = E;

    fn env(&self) -> &Self::Target {
        DynEnv::env(self)
    }
}

impl<E> AccessEnvMut for DynEnv<E> {
    fn env_mut(&mut self) -> &mut Self::Target {
        DynEnv::env_mut(self)
    }
}

impl<E> AccessEnv for EnvAccess<E> {
    type Target = Self;

    fn env(&self) -> &Self::Target {
        self
    }
}

impl<E> AccessEnv for EnvAccessMut<E> {
    type Target = Self;

    fn env(&self) -> &Self::Target {
        self
    }
}

impl<E> AccessEnvMut for EnvAccessMut<E> {
    fn env_mut(&mut self) -> &mut Self::Target {
        self
    }
}

/// Executes a contract message for the given inputs.
///
/// # Note
///
/// The message may not mutate the storage.
#[inline]
pub fn dispatch_msg<E, S, M>(
    storage: &S,
    call_data: &CallData,
    impl_fn: fn(&S, <M as FnInput>::Input) -> <M as FnOutput>::Output,
) -> DispatchResult
where
    E: Env,
    S: AccessEnv,
    <S as AccessEnv>::Target:
        AccessEnv<Target = ink_core::env2::EnvAccess<E>>,
    // We need double indirection because of optional `DynEnv` usage.
    <<S as AccessEnv>::Target as AccessEnv>::Target:
        AccessEnv<Target = ink_core::env2::EnvAccess<E>>,
    M: Message,
{
    let params = <M as FnInput>::Input::decode(&mut call_data.params())
        .map_err(|_| DispatchError::InvalidInstantiateParameters)?;
    let ret = impl_fn(storage, params);
    if TypeId::of::<<M as FnOutput>::Output>() != TypeId::of::<()>() {
        storage.env().env().output(&ret);
    }
    Ok(())
}

/// Executes a contract message for the given inputs on the storage.
///
/// # Note
///
/// The message may mutate the storage.
#[inline]
pub fn dispatch_msg_mut<E, S, M>(
    storage: &mut S,
    call_data: &CallData,
    impl_fn: fn(&mut S, <M as FnInput>::Input) -> <M as FnOutput>::Output,
) -> DispatchResult
where
    E: Env,
    S: AccessEnvMut + Flush,
    <S as AccessEnv>::Target:
        AccessEnvMut<Target = ink_core::env2::EnvAccessMut<E>>,
    // We need double indirection because of optional `DynEnv` usage.
    <<S as AccessEnv>::Target as AccessEnv>::Target:
        AccessEnvMut<Target = ink_core::env2::EnvAccessMut<E>>,
    M: Message,
{
    let params = <M as FnInput>::Input::decode(&mut call_data.params())
        .map_err(|_| DispatchError::InvalidInstantiateParameters)?;
    let ret = impl_fn(storage, params);
    if TypeId::of::<<M as FnOutput>::Output>() != TypeId::of::<()>() {
        storage.env_mut().env_mut().output(&ret);
    }
    // Only flush in case the message is really defined as mutable.
    if <M as Message>::IS_MUT {
        storage.flush();
    }
    Ok(())
}

/// Executes a contract constructor.
#[inline]
pub fn dispatch_constr<E, S, M>(
    storage: &mut S,
    call_data: &CallData,
    impl_fn: fn(&mut S, <M as FnInput>::Input),
) -> DispatchResult
where
    E: Env,
    S: Flush,
    M: Constructor,
{
    let params = <M as FnInput>::Input::decode(&mut call_data.params())
        .map_err(|_| DispatchError::InvalidInstantiateParameters)?;
    impl_fn(storage, params);
    storage.flush();
    Ok(())
}
