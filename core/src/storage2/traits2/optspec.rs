// Copyright 2019-2020 Parity Technologies (UK) Ltd.
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
//! These are mere optimizations compared to the non specialized root functions.
//! The specializations make use of the storage entry state (occupied or vacant)
//! in order to store the option's state thus using less storage in total.

use super::{
    KeyPtr,
    PackedLayout,
    SpreadLayout,
};
use crate::env;
use ink_primitives::Key;

pub fn pull_spread_root_opt<T>(root_key: &Key) -> Option<T>
where
    T: SpreadLayout,
{
    // In case the contract storage is occupied we handle
    // the Option<T> as if it was a T.
    env::get_contract_storage::<()>(*root_key)
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
            // Sadly this doesn't not work well with `Option<Option<T>>`.
            // For this we'd need specialization in Rust or similar.
            super::push_spread_root(value, root_key)
        }
        None => {
            // Clear all associated contract storage cells.
            //
            // Due to performance implications we do not allow this with
            // storage entities that have a footprint that is too big.
            let footprint = <T as SpreadLayout>::FOOTPRINT;
            assert!(
                footprint < 32,
                "footprint too large! try packing or boxing the storage entity."
            );
            let mut ptr = KeyPtr::from(*root_key);
            for _ in 0..footprint {
                env::clear_contract_storage(ptr.advance_by(1));
            }
        }
    }
}

pub fn pull_packed_root_opt<T>(root_key: &Key) -> Option<T>
where
    T: PackedLayout,
{
    match env::get_contract_storage::<T>(*root_key) {
        Some(value) => {
            // In case the contract storage is occupied we handle
            // the Option<T> as if it was a T.
            let mut value = value.expect("decoding does not match expected type");
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
            // Sadly this doesn't not work well with `Option<Option<T>>`.
            // For this we'd need specialization in Rust or similar.
            super::push_packed_root(value, root_key)
        }
        None => {
            // Clear the associated storage cell.
            env::clear_contract_storage(*root_key);
        }
    }
}
