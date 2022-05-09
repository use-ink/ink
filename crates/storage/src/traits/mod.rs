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

//! Traits and interfaces to operate with storage entities.
//!
//! Generally a type is said to be a storage entity if it implements the
//! `SpreadLayout` trait. This defines certain constants and routines in order
//! to tell a smart contract how to load and store instances of this type
//! from and to the contract's storage.
//!
//! The `PackedLayout` trait can then be implemented on top of the `SpreadLayout`
//! for types that further allow to be stored in the contract storage in a more
//! compressed format to a single storage cell.

mod impls;

#[cfg(feature = "std")]
mod layout;
mod storage;

#[cfg(feature = "std")]
pub use self::layout::{
    LayoutCryptoHasher,
    StorageLayout,
};
pub use self::storage::{
    AtomicGuard,
    AutoKey,
    AutomationStorageType,
    ManualKey,
    ResolverKey,
    StorageKeyHolder,
    StorageType,
    StorageType2,
};
use ink_primitives::StorageKey;
pub use ink_storage_derive::{
    AtomicGuard,
    StorageKeyHolder,
    StorageLayout,
    StorageType,
    StorageType2,
};
use scale::{
    Decode,
    Encode,
};

/// Pulls an instance of type `T` from the contract storage using decode and its storage key.
pub fn pull_storage<T>(key: &StorageKey) -> T
where
    T: Decode,
{
    ink_env::get_storage_value(key)
        .unwrap_or_else(|error| {
            panic!("failed to get storage value from key {}: {:?}", key, error)
        })
        .unwrap()
}

/// Pushes the entity to the contract storage using encode and storage key.
pub fn push_storage<T>(entity: &T, key: &StorageKey)
where
    T: Encode,
{
    ink_env::set_storage_value(key, entity)
}
