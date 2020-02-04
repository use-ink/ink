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
    StorageSize,
};

/// Pulls the associated key values from the contract storage and forms `Self`.
pub trait Pull {
    fn pull(key_ptr: &mut KeyPtr) -> Self;
}

fn pull_single_cell<T>(key_ptr: &mut KeyPtr) -> T
where
    T: StorageSize + scale::Decode,
{
    crate::env::get_contract_storage::<T>(key_ptr.next_for::<T>())
        .expect("storage entry was empty")
        .expect("could not properly decode storage entry")
}

macro_rules! impl_pull_for_primitive {
    ( $($ty:ty),* $(,)? ) => {
        $(
            impl Pull for $ty {
                fn pull(key_ptr: &mut KeyPtr) -> Self {
                    pull_single_cell::<$ty>(key_ptr)
                }
            }
        )*
    };
}
impl_pull_for_primitive!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128);

macro_rules! impl_pull_for_array {
    ( $($len:literal),* $(,)? ) => {
        $(
            impl<T> Pull for [T; $len]
            where
                [T; $len]: scale::Decode,
            {
                fn pull(key_ptr: &mut KeyPtr) -> Self {
                    pull_single_cell::<[T; $len]>(key_ptr)
                }
            }
        )*
    }
}
forward_supported_array_lens!(impl_pull_for_array);

macro_rules! impl_pull_tuple {
    ( $($frag:ident),* $(,)? ) => {
        impl<$($frag),*> Pull for ($($frag),* ,)
        where
            ( $($frag),* ,): scale::Decode,
        {
            fn pull(key_ptr: &mut KeyPtr) -> Self {
                pull_single_cell::<($($frag),* ,)>(key_ptr)
            }
        }
    }
}
impl_pull_tuple!(A);
impl_pull_tuple!(A, B);
impl_pull_tuple!(A, B, C);
impl_pull_tuple!(A, B, C, D);
impl_pull_tuple!(A, B, C, D, E);
impl_pull_tuple!(A, B, C, D, E, F);
impl_pull_tuple!(A, B, C, D, E, F, G);
impl_pull_tuple!(A, B, C, D, E, F, G, H);
impl_pull_tuple!(A, B, C, D, E, F, G, H, I);
impl_pull_tuple!(A, B, C, D, E, F, G, H, I, J);

impl Pull for () {
    fn pull(_key_ptr: &mut KeyPtr) -> Self {
        ()
    }
}

impl<T> Pull for core::marker::PhantomData<T> {
    fn pull(_key_ptr: &mut KeyPtr) -> Self {
        Default::default()
    }
}

impl<T> Pull for Option<T>
where
    Self: scale::Decode,
{
    fn pull(key_ptr: &mut KeyPtr) -> Self {
        pull_single_cell::<Self>(key_ptr)
    }
}

impl<T, E> Pull for Result<T, E>
where
    Self: scale::Decode,
{
    fn pull(key_ptr: &mut KeyPtr) -> Self {
        pull_single_cell::<Self>(key_ptr)
    }
}

impl<T> Pull for ink_prelude::vec::Vec<T>
where
    Self: scale::Decode,
{
    fn pull(key_ptr: &mut KeyPtr) -> Self {
        pull_single_cell::<Self>(key_ptr)
    }
}

impl Pull for ink_prelude::string::String
where
    Self: scale::Decode,
{
    fn pull(key_ptr: &mut KeyPtr) -> Self {
        pull_single_cell::<Self>(key_ptr)
    }
}

impl<T> Pull for ink_prelude::boxed::Box<T>
where
    Self: scale::Decode,
{
    fn pull(key_ptr: &mut KeyPtr) -> Self {
        pull_single_cell::<Self>(key_ptr)
    }
}

impl<T> Pull for ink_prelude::collections::BTreeSet<T>
where
    Self: scale::Decode,
{
    fn pull(key_ptr: &mut KeyPtr) -> Self {
        pull_single_cell::<Self>(key_ptr)
    }
}

impl<T> Pull for ink_prelude::collections::BinaryHeap<T>
where
    Self: scale::Decode,
{
    fn pull(key_ptr: &mut KeyPtr) -> Self {
        pull_single_cell::<Self>(key_ptr)
    }
}

impl<T> Pull for ink_prelude::collections::LinkedList<T>
where
    Self: scale::Decode,
{
    fn pull(key_ptr: &mut KeyPtr) -> Self {
        pull_single_cell::<Self>(key_ptr)
    }
}

impl<T> Pull for ink_prelude::collections::VecDeque<T>
where
    Self: scale::Decode,
{
    fn pull(key_ptr: &mut KeyPtr) -> Self {
        pull_single_cell::<Self>(key_ptr)
    }
}
