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

#[rustfmt::skip]
macro_rules! forward_supported_array_lens {
    ( $mac:ident ) => {
        $mac! {
                 1,  2,  3,  4,  5,  6,  7,  8,  9,
            10, 11, 12, 13, 14, 15, 16, 17, 18, 19,
            20, 21, 22, 23, 24, 25, 26, 27, 28, 29,
            30, 31, 32,
        }
    };
}

macro_rules! impl_always_packed_layout {
    ( $name:ident < $($frag:ident),+ > ) => {
        const _: () = {
            use crate::storage2::traits2::impls::{
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
    ( $name:ty ) => {
        const _: () = {
            use crate::storage2::traits2::impls::{
                forward_clear_packed,
                forward_pull_packed,
                forward_push_packed,
            };
            impl SpreadLayout for $name
            where
                Self: PackedLayout,
            {
                const FOOTPRINT: u64 = 1;

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

use super::{
    PackedLayout,
    pull_packed_root,
    push_packed_root,
    clear_packed_root
};
use crate::storage2::traits2::KeyPtr;

/// Returns the greater of both values.
const fn max(a: u64, b: u64) -> u64 {
    [a, b][(a > b) as usize]
}

#[inline]
pub fn forward_pull_packed<T>(ptr: &mut KeyPtr) -> T
where
    T: PackedLayout,
{
    pull_packed_root::<T>(&ptr.next_for::<T>())
}

#[inline]
pub fn forward_push_packed<T>(entity: &T, ptr: &mut KeyPtr)
where
    T: PackedLayout,
{
    push_packed_root::<T>(entity, &ptr.next_for::<T>())
}

#[inline]
pub fn forward_clear_packed<T>(entity: &T, ptr: &mut KeyPtr)
where
    T: PackedLayout,
{
    clear_packed_root::<T>(entity, &ptr.next_for::<T>())
}
