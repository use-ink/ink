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

use crate::traits::ExtKeyPtr as _;

macro_rules! impl_always_packed_layout {
    ( $name:ident < $($frag:ident),+ >, deep: $deep:expr ) => {
        const _: () = {
            use crate::traits::impls::{
                forward_clear_packed,
                forward_pull_packed,
                forward_push_packed,
            };
            impl<$($frag),+> SpreadLayout for $name < $($frag),+ >
            where
                $(
                    $frag: PackedLayout,
                )+
            {
                const FOOTPRINT: u64 = 1;

                const REQUIRES_DEEP_CLEAN_UP: bool = $deep;

                #[inline]
                fn pull_spread(ptr: &mut KeyPtr) -> Self {
                    forward_pull_packed::<Self>(ptr)
                }

                #[inline]
                fn push_spread(&self, ptr: &mut KeyPtr) {
                    forward_push_packed::<Self>(self, ptr)
                }

                #[inline]
                fn clear_spread(&self, ptr: &mut KeyPtr) {
                    forward_clear_packed::<Self>(self, ptr)
                }
            }
        };
    };
    ( $name:ty, deep: $deep:expr ) => {
        const _: () = {
            use crate::traits::impls::{
                forward_clear_packed,
                forward_pull_packed,
                forward_push_packed,
            };
            impl SpreadLayout for $name
            where
                Self: PackedLayout,
            {
                const FOOTPRINT: u64 = 1;

                const REQUIRES_DEEP_CLEAN_UP: bool = $deep;

                #[inline]
                fn pull_spread(ptr: &mut KeyPtr) -> Self {
                    forward_pull_packed::<Self>(ptr)
                }

                #[inline]
                fn push_spread(&self, ptr: &mut KeyPtr) {
                    forward_push_packed::<Self>(self, ptr)
                }

                #[inline]
                fn clear_spread(&self, ptr: &mut KeyPtr) {
                    forward_clear_packed::<Self>(self, ptr)
                }
            }
        };
    };
}

mod arrays;
mod collections;
mod prims;
mod tuples;

#[cfg(all(test, feature = "ink-fuzz-tests"))]
mod fuzz_tests;

use super::{
    clear_packed_root,
    pull_packed_root,
    push_packed_root,
    PackedLayout,
};
use crate::traits::KeyPtr;

/// Returns the greater of both values.
const fn max(a: u64, b: u64) -> u64 {
    [a, b][(a > b) as usize]
}

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
    push_packed_root::<T>(entity, ptr.next_for::<T>())
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
