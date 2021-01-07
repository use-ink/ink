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
use ink_primitives::Key;

macro_rules! impl_layout_for_tuple {
    ( $($frag:ident),* $(,)? ) => {
        impl<$($frag),*> SpreadLayout for ($($frag),* ,)
        where
            $(
                $frag: SpreadLayout,
            )*
        {
            const FOOTPRINT: u64 = 0 $(+ <$frag as SpreadLayout>::FOOTPRINT)*;
            const REQUIRES_DEEP_CLEAN_UP: bool = false $(|| <$frag as SpreadLayout>::REQUIRES_DEEP_CLEAN_UP)*;

            fn push_spread(&self, ptr: &mut KeyPtr) {
                #[allow(non_snake_case)]
                let ($($frag),*,) = self;
                $(
                    <$frag as SpreadLayout>::push_spread($frag, ptr);
                )*
            }

            fn clear_spread(&self, ptr: &mut KeyPtr) {
                #[allow(non_snake_case)]
                let ($($frag),*,) = self;
                $(
                    <$frag as SpreadLayout>::clear_spread($frag, ptr);
                )*
            }

            fn pull_spread(ptr: &mut KeyPtr) -> Self {
                (
                    $(
                        <$frag as SpreadLayout>::pull_spread(ptr),
                    )*
                )
            }
        }

        impl<$($frag),*> PackedLayout for ($($frag),* ,)
        where
            $(
                $frag: PackedLayout,
            )*
        {
            #[inline]
            fn push_packed(&self, at: &Key) {
                #[allow(non_snake_case)]
                let ($($frag),*,) = self;
                $(
                    <$frag as PackedLayout>::push_packed($frag, at);
                )*
            }

            #[inline]
            fn clear_packed(&self, at: &Key) {
                #[allow(non_snake_case)]
                let ($($frag),*,) = self;
                $(
                    <$frag as PackedLayout>::clear_packed($frag, at);
                )*
            }

            #[inline]
            fn pull_packed(&mut self, at: &Key) {
                #[allow(non_snake_case)]
                let ($($frag),*,) = self;
                $(
                    <$frag as PackedLayout>::pull_packed($frag, at);
                )*
            }
        }
    }
}
impl_layout_for_tuple!(A);
impl_layout_for_tuple!(A, B);
impl_layout_for_tuple!(A, B, C);
impl_layout_for_tuple!(A, B, C, D);
impl_layout_for_tuple!(A, B, C, D, E);
impl_layout_for_tuple!(A, B, C, D, E, F);
impl_layout_for_tuple!(A, B, C, D, E, F, G);
impl_layout_for_tuple!(A, B, C, D, E, F, G, H);
impl_layout_for_tuple!(A, B, C, D, E, F, G, H, I);
impl_layout_for_tuple!(A, B, C, D, E, F, G, H, I, J);

#[cfg(test)]
mod tests {
    use crate::push_pull_works_for_primitive;

    type TupleSix = (i32, u32, String, u8, bool, Box<Option<i32>>);
    push_pull_works_for_primitive!(
        TupleSix,
        [
            (
                -1,
                1,
                String::from("foobar"),
                13,
                true,
                Box::new(Some(i32::MIN))
            ),
            (
                i32::MIN,
                u32::MAX,
                String::from("❤ ♡ ❤ ♡ ❤"),
                Default::default(),
                false,
                Box::new(Some(i32::MAX))
            ),
            (
                Default::default(),
                Default::default(),
                Default::default(),
                Default::default(),
                Default::default(),
                Default::default()
            )
        ]
    );
}
