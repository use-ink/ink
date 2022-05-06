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

// Collection works only with atomic structures
macro_rules! impl_always_storage_type {
    ( $name:ident < $($frag:ident),+ > ) => {
        impl<
            Salt: $crate::traits::StorageKeyHolder,
            $($frag),+> $crate::traits::StorageType<Salt> for $name < $($frag),+ >
        where
            $(
                $frag: $crate::traits::AtomicGuard< { true } >,
            )+
        {
            type Type = $name < $($frag),+ >;
        }
        impl<$($frag),+> $crate::traits::AtomicGuard< { true } >
            for $name < $($frag),+ >
            where
                $(
                    $frag: $crate::traits::AtomicGuard< { true } >,
                )+
            {}
    };
    ( $name:ty ) => {
        impl<
            Salt: $crate::traits::StorageKeyHolder,
            > $crate::traits::StorageType<Salt> for $name
        {
            type Type = $name;
        }
        impl $crate::traits::AtomicGuard< { true } > for $name {}
    };
}

mod arrays;
mod collections;
mod prims;
mod tuples;

use super::{
    allocate_packed_root,
    clear_packed_root,
    pull_packed_root,
    push_packed_root,
    PackedAllocate,
    PackedLayout,
};
use crate::traits::{
    ExtKeyPtr as _,
    KeyPtr,
};

/// Pulls an instance of type `T` in packed fashion from the contract storage.
///
/// Loads the instance from the storage location identified by `ptr`.
/// The storage entity is expected to be decodable in its packed form.
///
/// # Note
///
/// Use this utility function to use a packed pull operation for the type
/// instead of a spread storage layout pull operation.
#[inline]
pub fn forward_pull_packed<T>(ptr: &mut KeyPtr) -> T
where
    T: PackedLayout,
{
    pull_packed_root::<T>(ptr.next_for::<T>())
}

/// Allocates an instance of type `T` in packed fashion to the contract storage.
///
/// This default initializes the entity at the storage location identified
/// by `ptr`. The storage entity is expected to be decodable in its packed form.
///
/// # Note
///
/// Use this utility function to use a packed allocate operation for the type
/// instead of a spread storage layout allocation operation.
#[inline]
pub fn forward_allocate_packed<T>(ptr: &mut KeyPtr) -> T
where
    T: PackedAllocate + Default,
{
    allocate_packed_root::<T>(ptr.next_for::<T>())
}

/// Pushes an instance of type `T` in packed fashion to the contract storage.
///
/// Stores the instance to the storage location identified by `ptr`.
/// The storage entity is expected to be encodable in its packed form.
///
/// # Note
///
/// Use this utility function to use a packed push operation for the type
/// instead of a spread storage layout push operation.
#[inline]
pub fn forward_push_packed<T>(entity: &T, ptr: &mut KeyPtr)
where
    T: PackedLayout,
{
    push_packed_root::<T>(entity, ptr.next_for::<T>());
}

/// Clears an instance of type `T` in packed fashion from the contract storage.
///
/// Clears the instance from the storage location identified by `ptr`.
/// The cleared storage entity is expected to be encoded in its packed form.
///
/// # Note
///
/// Use this utility function to use a packed clear operation for the type
/// instead of a spread storage layout clear operation.
#[inline]
pub fn forward_clear_packed<T>(entity: &T, ptr: &mut KeyPtr)
where
    T: PackedLayout,
{
    clear_packed_root::<T>(entity, ptr.next_for::<T>())
}
