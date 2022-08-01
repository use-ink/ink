// Copyright 2018-2022 Parity Technologies (UK) Ltd.
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

use super::PackedLayout;
use ink_primitives::Key;

pub fn pull_packed_root_opt<T>(root_key: &Key) -> Option<T>
where
    T: PackedLayout,
{
    ink_env::get_contract_storage::<T>(root_key)
        .unwrap_or_else(|error| {
            panic!(
                "failed to pull packed from root key {}: {:?}",
                root_key, error
            )
        })
        .map(|mut value| {
            // In case the contract storage is occupied at the root key
            // we handle the Option<T> as if it was a T.
            <T as PackedLayout>::pull_packed(&mut value, root_key);
            value
        })
}
