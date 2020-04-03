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

use ink_primitives::Key;

/// Implemented by types that can be stored on contract storage.
///
/// Tells the amount of storage cells the type requires to be stored.
pub trait StorageFootprint {
    /// The number of consecutive storage cells required by `Self` to be stored
    /// on the contract storage.
    const VALUE: u64;
}

macro_rules! impl_storage_size_for_primitive {
    ( $($ty:ty),* ) => {
        $(
            impl StorageFootprint for $ty {
                const VALUE: u64 = 1;
            }
        )*
    };
}
impl_storage_size_for_primitive!(
    // We do not include `f32` and `f64` since Wasm contracts currently
    // do not support them. We might add them to this list once we add
    // support for those primitives.
    Key, u8, u16, u32, u64, u128, i8, i16, i32, i64, i128
);

macro_rules! impl_storage_size_for_array2 {
    ( $($n:literal),* $(,)? ) => {
        $(
            impl<T> StorageFootprint for [T; $n]
            where
                T: StorageFootprint,
            {
                const VALUE: u64 = <T as StorageFootprint>::VALUE * $n;
            }
        )*
    };
}
forward_supported_array_lens!(impl_storage_size_for_array2);

macro_rules! impl_storage_size_tuple {
    ( $($frag:ident),* ) => {
        impl<T1 $(, $frag)*> StorageFootprint for (T1 $(, $frag)* ,)
        where
            T1: StorageFootprint,
            $(
                $frag: StorageFootprint,
            )*
        {
            const VALUE: u64 = (0 $( + <$frag as StorageFootprint>::VALUE )* );
        }
    }
}
impl_storage_size_tuple!();
impl_storage_size_tuple!(T2);
impl_storage_size_tuple!(T2, T3);
impl_storage_size_tuple!(T2, T3, T4);
impl_storage_size_tuple!(T2, T3, T4, T5);
impl_storage_size_tuple!(T2, T3, T4, T5, T6);
impl_storage_size_tuple!(T2, T3, T4, T5, T6, T7);
impl_storage_size_tuple!(T2, T3, T4, T5, T6, T7, T8);
impl_storage_size_tuple!(T2, T3, T4, T5, T6, T7, T8, T9);
impl_storage_size_tuple!(T2, T3, T4, T5, T6, T7, T8, T9, T10);

impl StorageFootprint for () {
    const VALUE: u64 = 0;
}

impl<T> StorageFootprint for core::marker::PhantomData<T> {
    const VALUE: u64 = 0;
}

impl<T> StorageFootprint for Option<T>
where
    T: StorageFootprint,
{
    const VALUE: u64 = <T as StorageFootprint>::VALUE;
}

/// Returns the greater element between `a` and `b`.
const fn max(a: u64, b: u64) -> u64 {
    [a, b][(a < b) as usize]
}

impl<T, E> StorageFootprint for Result<T, E>
where
    T: StorageFootprint,
    E: StorageFootprint,
{
    const VALUE: u64 = max(
        <T as StorageFootprint>::VALUE,
        <E as StorageFootprint>::VALUE,
    );
}

impl<T> StorageFootprint for ink_prelude::boxed::Box<T>
where
    T: StorageFootprint,
{
    const VALUE: u64 = <T as StorageFootprint>::VALUE;
}

impl StorageFootprint for ink_prelude::string::String {
    const VALUE: u64 = 1;
}

impl<T> StorageFootprint for ink_prelude::vec::Vec<T> {
    const VALUE: u64 = 1;
}

impl<K, V> StorageFootprint for ink_prelude::collections::BTreeMap<K, V> {
    const VALUE: u64 = 1;
}

macro_rules! impl_storage_size_for_collection {
    ( $($name:ident),* $(,)? ) => {
        $(
            impl<T> StorageFootprint for ink_prelude::collections::$name<T> {
                const VALUE: u64 = 1;
            }
        )*
    };
}
impl_storage_size_for_collection!(BinaryHeap, BTreeSet, VecDeque, LinkedList,);
