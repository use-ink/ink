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

/// Pushes the associated key values of `Self` to the contract storage.
pub trait Push {
    fn push(&self, key_ptr: &mut KeyPtr);
}

fn push_single_cell<T>(value: &T, key_ptr: &mut KeyPtr)
where
    T: StorageSize + scale::Encode,
{
    crate::env::set_contract_storage::<T>(key_ptr.next_for::<T>(), value)
}

macro_rules! impl_push_for_primitive {
    ( $($ty:ty),* $(,)? ) => {
        $(
            impl Push for $ty {
                fn push(&self, key_ptr: &mut KeyPtr) {
                    push_single_cell(self, key_ptr)
                }
            }
        )*
    };
}
impl_push_for_primitive!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128);

macro_rules! impl_push_for_array {
    ( $($len:literal),* $(,)? ) => {
        $(
            impl<T> Push for [T; $len]
            where
                [T; $len]: scale::Encode,
            {
                fn push(&self, key_ptr: &mut KeyPtr) {
                    push_single_cell(self, key_ptr)
                }
            }
        )*
    }
}
forward_supported_array_lens!(impl_push_for_array);

macro_rules! impl_push_tuple {
    ( $($frag:ident),* $(,)? ) => {
        impl<$($frag),*> Push for ($($frag),* ,)
        where
            ( $($frag),* ,): scale::Encode,
        {
            fn push(&self, key_ptr: &mut KeyPtr) {
                push_single_cell(self, key_ptr)
            }
        }
    }
}
impl_push_tuple!(A);
impl_push_tuple!(A, B);
impl_push_tuple!(A, B, C);
impl_push_tuple!(A, B, C, D);
impl_push_tuple!(A, B, C, D, E);
impl_push_tuple!(A, B, C, D, E, F);
impl_push_tuple!(A, B, C, D, E, F, G);
impl_push_tuple!(A, B, C, D, E, F, G, H);
impl_push_tuple!(A, B, C, D, E, F, G, H, I);
impl_push_tuple!(A, B, C, D, E, F, G, H, I, J);

impl Push for () {
    fn push(&self, _key_ptr: &mut KeyPtr) {}
}

impl<T> Push for core::marker::PhantomData<T> {
    fn push(&self, _key_ptr: &mut KeyPtr) {}
}

impl<T> Push for Option<T>
where
    Self: scale::Encode,
{
    fn push(&self, key_ptr: &mut KeyPtr) {
        push_single_cell(self, key_ptr)
    }
}

impl<T, E> Push for Result<T, E>
where
    Self: scale::Encode,
{
    fn push(&self, key_ptr: &mut KeyPtr) {
        push_single_cell(self, key_ptr)
    }
}

impl<T> Push for ink_prelude::vec::Vec<T>
where
    Self: scale::Encode,
{
    fn push(&self, key_ptr: &mut KeyPtr) {
        push_single_cell(self, key_ptr)
    }
}

impl Push for ink_prelude::string::String
where
    Self: scale::Encode,
{
    fn push(&self, key_ptr: &mut KeyPtr) {
        push_single_cell(self, key_ptr)
    }
}

impl<T> Push for ink_prelude::boxed::Box<T>
where
    Self: scale::Encode,
{
    fn push(&self, key_ptr: &mut KeyPtr) {
        push_single_cell(self, key_ptr)
    }
}

impl<T> Push for ink_prelude::collections::BTreeSet<T>
where
    Self: scale::Encode,
{
    fn push(&self, key_ptr: &mut KeyPtr) {
        push_single_cell(self, key_ptr)
    }
}

impl<T> Push for ink_prelude::collections::BinaryHeap<T>
where
    Self: scale::Encode,
{
    fn push(&self, key_ptr: &mut KeyPtr) {
        push_single_cell(self, key_ptr)
    }
}

impl<T> Push for ink_prelude::collections::LinkedList<T>
where
    Self: scale::Encode,
{
    fn push(&self, key_ptr: &mut KeyPtr) {
        push_single_cell(self, key_ptr)
    }
}

impl<T> Push for ink_prelude::collections::VecDeque<T>
where
    Self: scale::Encode,
{
    fn push(&self, key_ptr: &mut KeyPtr) {
        push_single_cell(self, key_ptr)
    }
}
