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

use crate::traits::{
    KeyPtr,
    PackedLayout,
    SpreadLayout,
};
use array_init::array_init;
use ink_primitives::Key;

impl<T, const N: usize> SpreadLayout for [T; N]
where
    T: SpreadLayout,
{
    const FOOTPRINT: u64 = N as u64 * <T as SpreadLayout>::FOOTPRINT;
    const REQUIRES_DEEP_CLEAN_UP: bool = <T as SpreadLayout>::REQUIRES_DEEP_CLEAN_UP;

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
        array_init::<_, T, N>(|_| <T as SpreadLayout>::pull_spread(ptr))
    }
}

impl<T, const N: usize> PackedLayout for [T; N]
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

#[cfg(test)]
mod tests {
    use crate::push_pull_works_for_primitive;

    type Array = [i32; 4];
    push_pull_works_for_primitive!(
        Array,
        [
            [0, 1, 2, 3],
            [i32::MAX, i32::MIN, i32::MAX, i32::MIN],
            [Default::default(), i32::MAX, Default::default(), i32::MIN]
        ]
    );

    type ArrayTuples = [(i32, i32); 2];
    push_pull_works_for_primitive!(
        ArrayTuples,
        [
            [(0, 1), (2, 3)],
            [(i32::MAX, i32::MIN), (i32::MIN, i32::MAX)],
            [
                (Default::default(), i32::MAX),
                (Default::default(), i32::MIN)
            ]
        ]
    );
}
