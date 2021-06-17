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

//! Implement specialized routines for managing Option<T> storage entities.
//!
//! These are mere optimizations compared to the non-specialized root functions.
//! The specializations make use of the storage entry state (occupied or vacant)
//! in order to store the option's state thus using less storage in total.

use super::{
    KeyPtr,
    PackedLayout,
    SpreadLayout,
};
use ink_primitives::Key;

pub fn pull_spread_root_opt<T>(root_key: &Key) -> Option<T>
where
    T: SpreadLayout,
{
    // In case the contract storage is occupied we handle
    // the Option<T> as if it was a T.
    ink_env::get_contract_storage::<()>(root_key)
        .ok()
        .flatten()
        .map(|_| super::pull_spread_root::<T>(root_key))
}

pub fn push_spread_root_opt<T>(entity: Option<&T>, root_key: &Key)
where
    T: SpreadLayout,
{
    match entity {
        Some(value) => {
            // Handle the Option<T> as if it was a T.
            //
            // Sadly this does not not work well with `Option<Option<T>>`.
            // For this we'd need specialization in Rust or similar.
            super::push_spread_root(value, root_key)
        }
        None => clear_spread_root_opt::<T, _>(root_key, || entity),
    }
}

pub fn clear_spread_root_opt<'a, T: 'a, F>(root_key: &Key, f: F)
where
    T: SpreadLayout,
    F: FnOnce() -> Option<&'a T>,
{
    // We can clean up some storage entity using its `SpreadLayout::clear_spread`
    // implementation or its defined storage footprint.
    //
    // While using its `SpreadLayout::clear_spread` implementation is more precise
    // and will only clean-up what is necessary it requires an actual instance.
    // Loading such an instance if it is not already in the memory cache of some
    // lazy abstraction will incur significant overhead.
    // Using its defined storage footprint this procedure can eagerly clean-up
    // the associated contract storage region, however, this might clean-up more
    // cells than needed.
    //
    // There are types that need a so-called "deep" clean-up. An example for this
    // is `storage::Box<storage::Box<T>>` where the outer storage box definitely
    // needs to propagate clearing signals onto its inner `storage::Box` in order
    // to properly clean-up the whole associate contract storage region.
    // This is when we cannot avoid loading the entity for the clean-up procedure.
    //
    // If the entity that shall be cleaned-up does not require deep clean-up we
    // check if its storage footprint exceeds a certain threshold and only then
    // we will still load it first in order to not clean-up too many unneeded
    // storage cells.
    let footprint = <T as SpreadLayout>::FOOTPRINT;
    if footprint >= super::FOOTPRINT_CLEANUP_THRESHOLD
        || <T as SpreadLayout>::REQUIRES_DEEP_CLEAN_UP
    {
        // We need to load the entity before we remove its associated contract storage
        // because it requires a deep clean-up which propagates clearing to its fields,
        // for example in the case of `T` being a `storage::Box`.
        if let Some(value) = f() {
            super::clear_spread_root(value, root_key);
            return
        }
    }
    // Clean-up eagerly without potentially loading the entity from storage:
    let mut ptr = KeyPtr::from(*root_key);
    for _ in 0..footprint {
        ink_env::clear_contract_storage(ptr.advance_by(1));
    }
}

pub fn pull_packed_root_opt<T>(root_key: &Key) -> Option<T>
where
    T: PackedLayout,
{
    match ink_env::get_contract_storage::<T>(root_key)
        .expect("decoding does not match expected type")
    {
        Some(mut value) => {
            // In case the contract storage is occupied we handle
            // the Option<T> as if it was a T.
            <T as PackedLayout>::pull_packed(&mut value, root_key);
            Some(value)
        }
        None => None,
    }
}

pub fn push_packed_root_opt<T>(entity: Option<&T>, root_key: &Key)
where
    T: PackedLayout,
{
    match entity {
        Some(value) => {
            // Handle the Option<T> as if it was a T.
            //
            // Sadly this does not work well with `Option<Option<T>>`.
            // For this we'd need specialization in Rust or similar.
            super::push_packed_root(value, root_key)
        }
        None => {
            // Clear the associated storage cell.
            ink_env::clear_contract_storage(root_key);
        }
    }
}
