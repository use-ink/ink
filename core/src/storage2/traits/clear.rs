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

use super::{
    KeyPtr,
    StorageFootprint,
};
use crate::env;
use core::marker::PhantomData;
use ink_primitives::Key;

/// Types that implement this trait can clear their state at the associated
/// contract storage in a distributive manner.
///
/// # Note
///
/// This tries to distribute all clearing operations of an entity at distinct
/// storage cells.
pub trait ClearForward {
    /// Pushes `self` distributed to the associated contract storage.
    fn clear_forward(&self, ptr: &mut KeyPtr);
}

/// Types that implement this trait can clear their state at an associated
/// contract storage cell.
///
/// # Note
///
/// This tries to compactly clear the entire entity's fields at a single
/// contract storage cell.
pub trait ClearAt {
    /// Clears `self` packed at the contract storage cell determined by `at`.
    #[inline]
    fn clear_at(&self, at: Key) {
        env::clear_contract_storage(at)
    }
}

macro_rules! impl_clear_for_primitive {
    ( $($ty:ty),* $(,)? ) => {
        $(
            impl ClearForward for $ty {
                #[inline]
                fn clear_forward(&self, ptr: &mut KeyPtr) {
                    <$ty as ClearAt>::clear_at(self, ptr.next_for::<$ty>())
                }
            }

            impl ClearAt for $ty {}
        )*
    };
}
impl_clear_for_primitive!(Key, u8, u16, u32, u64, u128, i8, i16, i32, i64, i128);

macro_rules! impl_clear_for_array {
    ( $($len:literal),* $(,)? ) => {
        $(
            impl<T> ClearForward for [T; $len]
            where
                T: ClearForward,
            {
                fn clear_forward(&self, ptr: &mut KeyPtr) {
                    for elem in self.iter() {
                        <T as ClearForward>::clear_forward(elem, ptr)
                    }
                }
            }

            impl<T> ClearAt for [T; $len] {}
        )*
    }
}
forward_supported_array_lens!(impl_clear_for_array);

macro_rules! impl_clear_tuple {
    ( $($frag:ident),* $(,)? ) => {
        impl<$($frag),*> ClearForward for ($($frag),* ,)
        where
            $(
                $frag: ClearForward,
            )*
        {
            fn clear_forward(&self, ptr: &mut KeyPtr) {
                #[allow(non_snake_case)]
                let ($($frag),*,) = self;
                $(
                    <$frag as ClearForward>::clear_forward($frag, ptr);
                )*
            }
        }

        impl<$($frag),*> ClearAt for ($($frag),* ,) {}
    }
}
impl_clear_tuple!(A);
impl_clear_tuple!(A, B);
impl_clear_tuple!(A, B, C);
impl_clear_tuple!(A, B, C, D);
impl_clear_tuple!(A, B, C, D, E);
impl_clear_tuple!(A, B, C, D, E, F);
impl_clear_tuple!(A, B, C, D, E, F, G);
impl_clear_tuple!(A, B, C, D, E, F, G, H);
impl_clear_tuple!(A, B, C, D, E, F, G, H, I);
impl_clear_tuple!(A, B, C, D, E, F, G, H, I, J);

impl ClearForward for () {
    fn clear_forward(&self, _ptr: &mut KeyPtr) {}
}

impl ClearAt for () {
    fn clear_at(&self, _at: Key) { // literally do nothing here
    }
}

impl<T> ClearForward for PhantomData<T> {
    fn clear_forward(&self, _ptr: &mut KeyPtr) {}
}

impl<T> ClearAt for PhantomData<T> {
    fn clear_at(&self, _at: Key) { // literally do nothing here
    }
}

impl<T> ClearForward for Option<T>
where
    T: ClearForward + StorageFootprint,
{
    fn clear_forward(&self, ptr: &mut KeyPtr) {
        match self {
            Some(val) => <T as ClearForward>::clear_forward(val, ptr),
            None => (),
        }
    }
}

impl<T> ClearAt for Option<T>
where
    T: ClearAt,
{
    fn clear_at(&self, at: Key) {
        match self {
            Some(val) => <T as ClearAt>::clear_at(val, at),
            None => env::clear_contract_storage(at),
        }
    }
}

impl<T, E> ClearForward for Result<T, E>
where
    T: ClearForward,
    E: ClearForward,
{
    fn clear_forward(&self, ptr: &mut KeyPtr) {
        match self {
            Ok(val) => {
                ClearForward::clear_forward(&0u8, ptr);
                <T as ClearForward>::clear_forward(val, ptr);
            }
            Err(err) => {
                ClearForward::clear_forward(&1u8, ptr);
                <E as ClearForward>::clear_forward(err, ptr);
            }
        }
    }
}

impl<T, E> ClearAt for Result<T, E> {}

impl ClearForward for ink_prelude::string::String {
    fn clear_forward(&self, ptr: &mut KeyPtr) {
        <Self as ClearAt>::clear_at(self, ptr.next_for::<Self>())
    }
}

impl ClearAt for ink_prelude::string::String {}

impl<T> ClearForward for ink_prelude::boxed::Box<T>
where
    T: ClearForward,
{
    fn clear_forward(&self, ptr: &mut KeyPtr) {
        <T as ClearForward>::clear_forward(&*self, ptr)
    }
}

impl<T> ClearAt for ink_prelude::boxed::Box<T>
where
    T: ClearAt,
{
    fn clear_at(&self, at: Key) {
        <T as ClearAt>::clear_at(&*self, at)
    }
}

const _: () = {
    use ink_prelude::collections::{
        BTreeSet,
        BinaryHeap,
        LinkedList,
        VecDeque,
    };
    #[cfg(not(feature = "std"))]
    use ink_prelude::vec::Vec;

    macro_rules! impl_clear_at_for_collection {
        ( $($collection:ident),* $(,)? ) => {
            $(
                impl<T> ClearAt for $collection<T> {}
            )*
        };
    }
    impl_clear_at_for_collection!(Vec, BTreeSet, BinaryHeap, LinkedList, VecDeque,);
};

impl<K, V> ClearAt for ink_prelude::collections::BTreeMap<K, V> {}
