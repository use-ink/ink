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

use crate::reflect::{
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

/// Returns `Ok` if the caller did not transfer additional value to the callee.
///
/// # Errors
///
/// If the caller did send some amount of transferred value to the callee.
#[inline]
pub fn deny_payment<E>() -> Result<(), DispatchError>
where
    E: Environment,
{
    let transferred = ink_env::transferred_balance::<E>();
    if transferred != <E as Environment>::Balance::from(0_u32) {
        return Err(DispatchError::PaidUnpayableMessage)
    }
    Ok(())
}

/// Configuration for execution of ink! constructor.
#[derive(Debug, Copy, Clone)]
pub struct ExecuteConstructorConfig {
    /// Yields `true` if the dynamic storage allocator has been enabled.
    ///
    /// # Note
    ///
    /// Authors can enable it via `#[ink::contract(dynamic_storage_allocator = true)]`.
    pub dynamic_storage_alloc: bool,
}

/// Executes the given ink! constructor.
///
/// # Note
///
/// The closure is supposed to already contain all the arguments that the real
/// constructor message requires and forwards them.
#[inline]
pub fn execute_constructor<S, F>(
    config: ExecuteConstructorConfig,
    f: F,
) -> Result<(), DispatchError>
where
    S: ink_storage::traits::SpreadLayout,
    F: FnOnce() -> S,
{
    if config.dynamic_storage_alloc {
        alloc::initialize(ContractPhase::Deploy);
    }
    let storage = ManuallyDrop::new(f());
    let root_key = Key::from([0x00; 32]);
    push_spread_root::<S>(&storage, &root_key);
    if config.dynamic_storage_alloc {
        alloc::finalize();
    }
    Ok(())
}

/// Configuration for execution of ink! messages.
#[derive(Debug, Copy, Clone)]
pub struct ExecuteMessageConfig {
    /// Yields `true` if the ink! message accepts payment.
    ///
    /// # Note
    ///
    /// If no ink! message within the same ink! smart contract
    /// is payable then this flag will be `true` since the check
    /// then is moved before the message dispatch as an optimization.
    pub payable: bool,
    /// Yields `true` if the ink! message might mutate contract storage.
    ///
    /// # Note
    ///
    /// This is usually true for `&mut self` ink! messages.
    pub mutates: bool,
    /// Yields `true` if the ink! message execution might revert execution.
    ///
    /// # Note
    ///
    /// In ink! this is usually `true` for root ink! smart contracts and
    /// `false` for dependency ink! smart contracts.
    pub may_revert: bool,
    /// Yields `true` if the dynamic storage allocator has been enabled.
    ///
    /// # Note
    ///
    /// Authors can enable it via `#[ink::contract(dynamic_storage_allocator = true)]`.
    pub dynamic_storage_alloc: bool,
}

/// Executes the given `&mut self` message closure.
///
/// # Note
///
/// The closure is supposed to already contain all the arguments that the real
/// message requires and forwards them.
#[inline]
pub fn execute_message<Storage, Output, F>(
    config: ExecuteMessageConfig,
    f: F,
) -> Result<(), DispatchError>
where
    Storage: SpreadLayout + ContractEnv,
    Output: scale::Encode + 'static,
    F: FnOnce(&mut Storage) -> Output,
{
    if !config.payable {
        deny_payment::<<Storage as ContractEnv>::Env>()?;
    }
    if config.dynamic_storage_alloc {
        alloc::initialize(ContractPhase::Call);
    }
    let root_key = Key::from([0x00; 32]);
    let mut storage = ManuallyDrop::new(pull_spread_root::<Storage>(&root_key));
    let result = f(&mut storage);
    if config.mutates {
        push_spread_root::<Storage>(&storage, &root_key);
    }
    if config.dynamic_storage_alloc {
        alloc::finalize();
    }
    if TypeId::of::<Output>() != TypeId::of::<()>() {
        // We include a check for `is_result_type!(Output)` despite the fact that this
        // is indirectly covered by `is_result_err!(&result)` because the Rust compiler
        // will have more opportunities to optimize the whole conditional away. This is
        // due to the fact that `is_result_type!` relies on constant information whereas
        // is_result_err!` requires `&self`.
        let revert_state =
            config.may_revert && is_result_type!(Output) && is_result_err!(&result);
        ink_env::return_value::<Output>(
            ReturnFlags::default().set_reverted(revert_state),
            &result,
        )
    }
    Ok(())
}
