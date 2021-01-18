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
mod keyptr;
mod optspec;
mod packed;
mod spread;

#[cfg(feature = "std")]
mod layout;

#[cfg(feature = "std")]
pub use self::layout::{
    LayoutCryptoHasher,
    StorageLayout,
};
pub(crate) use self::optspec::{
    clear_spread_root_opt,
    pull_packed_root_opt,
    pull_spread_root_opt,
    push_packed_root_opt,
    push_spread_root_opt,
};
pub use self::{
    impls::{
        forward_clear_packed,
        forward_pull_packed,
        forward_push_packed,
    },
    keyptr::{
        ExtKeyPtr,
        KeyPtr,
    },
    packed::PackedLayout,
    spread::{
        SpreadLayout,
        FOOTPRINT_CLEANUP_THRESHOLD,
    },
};
pub use ::ink_storage_derive::{
    PackedLayout,
    SpreadLayout,
    StorageLayout,
};
use ink_primitives::Key;

/// Pulls an instance of type `T` from the contract storage using spread layout.
///
/// The root key denotes the offset into the contract storage where the
/// instance of type `T` is being pulled from.
///
/// # Note
///
/// - The routine assumes that the instance has previously been stored to
///   the contract storage using spread layout.
/// - Users should prefer using this function directly instead of using the
///   trait methods on [`SpreadLayout`].
pub fn pull_spread_root<T>(root_key: &Key) -> T
where
    T: SpreadLayout,
{
    let mut ptr = KeyPtr::from(*root_key);
    <T as SpreadLayout>::pull_spread(&mut ptr)
}

/// Clears the entity from the contract storage using spread layout.
///
/// The root key denotes the offset into the contract storage where the
/// instance of type `T` is being cleared from.
///
/// # Note
///
/// - The routine assumes that the instance has previously been stored to
///   the contract storage using spread layout.
/// - Users should prefer using this function directly instead of using the
///   trait methods on [`SpreadLayout`].
pub fn clear_spread_root<T>(entity: &T, root_key: &Key)
where
    T: SpreadLayout,
{
    let mut ptr = KeyPtr::from(*root_key);
    <T as SpreadLayout>::clear_spread(entity, &mut ptr);
}

/// Pushes the entity to the contract storage using spread layout.
///
/// The root key denotes the offset into the contract storage where the
/// instance of type `T` is being pushed to.
///
/// # Note
///
/// - The routine will push the given entity to the contract storage using
///   spread layout.
/// - Users should prefer using this function directly instead of using the
///   trait methods on [`SpreadLayout`].
pub fn push_spread_root<T>(entity: &T, root_key: &Key)
where
    T: SpreadLayout,
{
    let mut ptr = KeyPtr::from(*root_key);
    <T as SpreadLayout>::push_spread(entity, &mut ptr);
}

/// Pulls an instance of type `T` from the contract storage using packed layout.
///
/// The root key denotes the offset into the contract storage where the
/// instance of type `T` is being pulled from.
///
/// # Note
///
/// - The routine assumes that the instance has previously been stored to
///   the contract storage using packed layout.
/// - Users should prefer using this function directly instead of using the
///   trait methods on [`PackedLayout`].
pub fn pull_packed_root<T>(root_key: &Key) -> T
where
    T: PackedLayout,
{
    let mut entity = ink_env::get_contract_storage::<T>(root_key)
        .expect("could not properly decode storage entry")
        .expect("storage entry was empty");
    <T as PackedLayout>::pull_packed(&mut entity, root_key);
    entity
}

/// Pushes the entity to the contract storage using packed layout.
///
/// The root key denotes the offset into the contract storage where the
/// instance of type `T` is being pushed to.
///
/// # Note
///
/// - The routine will push the given entity to the contract storage using
///   packed layout.
/// - Users should prefer using this function directly instead of using the
///   trait methods on [`PackedLayout`].
pub fn push_packed_root<T>(entity: &T, root_key: &Key)
where
    T: PackedLayout,
{
    <T as PackedLayout>::push_packed(entity, root_key);
    ink_env::set_contract_storage(root_key, entity);
}

/// Clears the entity from the contract storage using packed layout.
///
/// The root key denotes the offset into the contract storage where the
/// instance of type `T` is being cleared from.
///
/// # Note
///
/// - The routine assumes that the instance has previously been stored to
///   the contract storage using packed layout.
/// - Users should prefer using this function directly instead of using the
///   trait methods on [`PackedLayout`].
pub fn clear_packed_root<T>(entity: &T, root_key: &Key)
where
    T: PackedLayout,
{
    <T as PackedLayout>::clear_packed(entity, root_key);
    ink_env::clear_contract_storage(root_key);
}
