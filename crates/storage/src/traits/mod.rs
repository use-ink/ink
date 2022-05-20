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
//! `StorageType` trait. This defines certain constants and routines in order
//! to tell a smart contract how to load and store instances of this type
//! from and to the contract's storage.
//!
//! The `AtomicGuard<true>` shows that the type is atomic and can be stored
//! into single storage cell. Some collections works only with atomic structures.

mod impls;
mod storage;

#[cfg(feature = "std")]
mod layout;

#[macro_use]
#[doc(hidden)]
pub mod pull_or_init;

#[cfg(feature = "std")]
pub use self::layout::{
    LayoutCryptoHasher,
    StorageLayout,
};
pub use self::{
    impls::storage::{
        AutoKey,
        ManualKey,
        ResolverKey,
    },
    storage::{
        AtomicGuard,
        AutoStorageType,
        OnCallInitializer,
        StorageKeyHolder,
        StorageType,
    },
};
use ink_primitives::StorageKey;
pub use ink_storage_derive::{
    AtomicGuard,
    StorageKeyHolder,
    StorageLayout,
    StorageType,
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
    ink_env::get_contract_storage::<(), T>(key, None)
        .expect("could not properly decode storage entry")
        .expect("storage entry was empty")
}

/// Pushes the entity to the contract storage using encode and storage key.
pub fn push_storage<T>(entity: &T, key: &StorageKey) -> Option<u32>
where
    T: Encode,
{
    ink_env::set_contract_storage::<(), T>(key, None, entity)
}
