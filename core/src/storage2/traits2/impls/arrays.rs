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

use crate::storage2::traits2::{
    KeyPtr,
    PackedLayout,
    SpreadLayout,
};
use array_init::array_init;
use ink_primitives::Key;

macro_rules! impl_layout_for_array {
    ( $($len:literal),* $(,)? ) => {
        $(
            impl<T> SpreadLayout for [T; $len]
            where
                T: SpreadLayout,
            {
                const FOOTPRINT: u64 = $len * <T as SpreadLayout>::FOOTPRINT;

                fn push_spread(&self, ptr: &mut KeyPtr) {
                    for elem in self {
                        <T as SpreadLayout>::push_spread(elem, ptr)
                    }
                }

                fn clear_spread(&self, ptr: &mut KeyPtr) {
                    for elem in self {
                        <T as SpreadLayout>::clear_spread(elem, ptr)
                    }
                }

                fn pull_spread(ptr: &mut KeyPtr) -> Self {
                    array_init::<Self, _>(|_| <T as SpreadLayout>::pull_spread(ptr))
                }
            }

            impl<T> PackedLayout for [T; $len]
            where
                T: PackedLayout,
            {
                #[inline]
                fn push_packed(&self, at: &Key) {
                    for elem in self {
                        <T as PackedLayout>::push_packed(elem, at)
                    }
                }

                #[inline]
                fn clear_packed(&self, at: &Key) {
                    for elem in self {
                        <T as PackedLayout>::clear_packed(elem, at)
                    }
                }

                #[inline]
                fn pull_packed(&mut self, at: &Key) {
                    for elem in self {
                        <T as PackedLayout>::pull_packed(elem, at)
                    }
                }
            }
        )*
    }
}
forward_supported_array_lens!(impl_layout_for_array);
