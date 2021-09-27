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
    ContractEnv,
    DispatchError,
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
        SpreadLayout,
    },
};

/// Results of message handling operations.
#[doc(hidden)]
pub type Result<T> = core::result::Result<T, DispatchError>;

/// Yields `true` if the message accepts payments.
#[derive(Copy, Clone)]
#[doc(hidden)]
pub struct AcceptsPayments(pub bool);

/// Yields `true` if the associated ink! message may mutate contract storage.
#[derive(Copy, Clone)]
#[doc(hidden)]
pub struct MutatesStorage(pub bool);

/// Yields `true` if the associated ink! message may revert execution.
#[derive(Copy, Clone)]
#[doc(hidden)]
pub struct MayRevert(pub bool);

/// Yields `true` if the dynamic storage allocator is enabled for the given call.
#[derive(Copy, Clone)]
#[doc(hidden)]
pub struct EnablesDynamicStorageAllocator(pub bool);

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

/// Executes the given ink! constructor.
///
/// # Note
///
/// The closure is supposed to already contain all the arguments that the real
/// constructor message requires and forwards them.
#[inline]
#[doc(hidden)]
pub fn execute_constructor<S, F>(
    EnablesDynamicStorageAllocator(uses_dynamic_storage_allocator): EnablesDynamicStorageAllocator,
    f: F,
) -> Result<()>
where
    S: ink_storage::traits::SpreadLayout,
    F: FnOnce() -> S,
{
    if uses_dynamic_storage_allocator {
        alloc::initialize(ContractPhase::Deploy);
    }
    let storage = ManuallyDrop::new(f());
    let root_key = Key::from([0x00; 32]);
    push_spread_root::<S>(&storage, &root_key);
    if uses_dynamic_storage_allocator {
        alloc::finalize();
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
pub fn execute_message<Storage, Output, F>(
    AcceptsPayments(accepts_payments): AcceptsPayments,
    MutatesStorage(mutates_storage): MutatesStorage,
    MayRevert(may_revert): MayRevert,
    EnablesDynamicStorageAllocator(enables_dynamic_storage_allocator): EnablesDynamicStorageAllocator,
    f: F,
) -> Result<()>
where
    Storage: SpreadLayout + ContractEnv,
    Output: scale::Encode + 'static,
    F: FnOnce(&mut Storage) -> Output,
{
    if !accepts_payments {
        deny_payment::<<Storage as ContractEnv>::Env>()?;
    }
    if enables_dynamic_storage_allocator {
        alloc::initialize(ContractPhase::Call);
    }
    let root_key = Key::from([0x00; 32]);
    let mut storage = ManuallyDrop::new(pull_spread_root::<Storage>(&root_key));
    let result = f(&mut storage);
    if mutates_storage {
        push_spread_root::<Storage>(&storage, &root_key);
    }
    if enables_dynamic_storage_allocator {
        alloc::finalize();
    }
    if TypeId::of::<Output>() != TypeId::of::<()>() {
        let revert_state =
            may_revert && is_result_type!(Output) && is_result_err!(&result);
        ink_env::return_value::<Output>(
            ReturnFlags::default().set_reverted(revert_state),
            &result,
        )
    }
    Ok(())
}
