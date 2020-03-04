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
pub trait StorageSize {
    /// The number of storage cells required by `Self` to be stored
    /// on the contract storage.
    const SIZE: u64;
}

use core::ops::{
    Add,
    Mul,
};
use typenum::{
    IsEqual,
    Max,
    Maximum,
    Prod,
    Sum,
    B1 as True,
    P1,
    Z0,
};

/// Implemented by types that can be stored on contract storage.
///
/// Tells the amount of storage cells the type requires to be stored.
pub trait StorageFootprint {
    /// The number of storage cells required by `Self` to be stored
    /// on the contract storage.
    ///
    /// # Note
    ///
    /// Using a type (`typenum`) here instead of an associated constant
    /// solves some problems for implementations of generics because Rust's
    /// handling of those associated constants is not mature while it can
    /// easily handle the `typenum` types solving the same underlying problem
    /// of representing a computable compile-time number.
    ///
    /// We should switch back to associated constants once the Rust compiler
    /// is more mature at handling them in generics.
    type Value;
}

/// Types implementing this trait are guaranteed to always use the same amount
/// of storage cells as described by the [`StorageSize`] trait.
///
/// It is a bug to implement this trait for a type that does not respect this
/// behaviour.
pub trait SaturatingStorage: StorageSize {}

macro_rules! impl_storage_size_for_primitive {
    ( $($ty:ty),* ) => {
        $(
            impl StorageSize for $ty {
                const SIZE: u64 = 1;
            }
            impl StorageFootprint for $ty {
                type Value = P1;
            }
            impl SaturatingStorage for $ty {}
        )*
    };
}
impl_storage_size_for_primitive!(Key, u8, u16, u32, u64, u128, i8, i16, i32, i64, i128);

macro_rules! impl_storage_size_for_array {
    ( $($n:literal),* $(,)? ) => {
        $(
            impl<T> StorageSize for [T; $n]
            where
                T: StorageSize,
            {
                const SIZE: u64 = <T as StorageSize>::SIZE * $n;
            }
            impl<T> SaturatingStorage for [T; $n]
            where
                T: SaturatingStorage,
            {}
        )*
    };
}
forward_supported_array_lens!(impl_storage_size_for_array);

macro_rules! impl_storage_size_for_array2 {
    ( $(($n:literal, $t:ty)),* $(,)? ) => {
        $(
            impl<T> StorageFootprint for [T; $n]
            where
                T: StorageFootprint,
                <T as StorageFootprint>::Value: Mul<$t>,
            {
                type Value = Prod<<T as StorageFootprint>::Value, $t>;
            }
        )*
    };
}
forward_supported_array_lens_ty!(impl_storage_size_for_array2);

macro_rules! impl_storage_size_tuple {
    ( $($frag:ident),* $(,)? ) => {
        #[allow(unused_parens)]
        impl<$($frag),*> StorageSize for ($($frag),* ,)
        where
            $(
                $frag: StorageSize,
            )*
        {
            const SIZE: u64 = 0
                $(
                    + <$frag as StorageSize>::SIZE
                )*
            ;
        }
        impl<$($frag),*> SaturatingStorage for ($($frag),* ,)
        where
            $(
                $frag: SaturatingStorage,
            )*
        {}
    }
}
impl_storage_size_tuple!(A);
impl_storage_size_tuple!(A, B);
impl_storage_size_tuple!(A, B, C);
impl_storage_size_tuple!(A, B, C, D);
impl_storage_size_tuple!(A, B, C, D, E);
impl_storage_size_tuple!(A, B, C, D, E, F);
impl_storage_size_tuple!(A, B, C, D, E, F, G);
impl_storage_size_tuple!(A, B, C, D, E, F, G, H);
impl_storage_size_tuple!(A, B, C, D, E, F, G, H, I);
impl_storage_size_tuple!(A, B, C, D, E, F, G, H, I, J);

macro_rules! impl_storage_size_tuple {
    ( $($frag:ident),* ) => {
        impl<T1 $(, $frag)*> StorageFootprint for (T1 $(, $frag)* ,)
        where
            T1: StorageFootprint,
            ($($frag ,)*): StorageFootprint,
            <T1 as StorageFootprint>::Value: Add<<($($frag ,)*) as StorageFootprint>::Value>,
        {
            type Value = Sum<<T1 as StorageFootprint>::Value, <($($frag ,)*) as StorageFootprint>::Value>;
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

impl StorageSize for () {
    const SIZE: u64 = 0;
}
impl StorageFootprint for () {
    type Value = Z0;
}
impl SaturatingStorage for () {}

impl<T> StorageSize for core::marker::PhantomData<T> {
    const SIZE: u64 = 0;
}
impl<T> StorageFootprint for core::marker::PhantomData<T> {
    type Value = Z0;
}
impl<T> SaturatingStorage for core::marker::PhantomData<T> {}

impl<T> StorageSize for Option<T>
where
    T: StorageSize,
{
    const SIZE: u64 = <T as StorageSize>::SIZE;
}
impl<T> StorageFootprint for Option<T>
where
    T: StorageFootprint,
{
    type Value = <T as StorageFootprint>::Value;
}
impl<T> SaturatingStorage for Option<T>
where
    T: SaturatingStorage,
{
    // Actually whether `SaturatingStorage` for `Option<T>` should be
    // implemented is an interesting question since it either takes up no
    // storage cells in the current implementation or it takes the same amount
    // of storage cells as `T`.
    // But since the amount of storage cells taken can always be derived from
    // the current state of the `Option` (`Some` or `None`) and compile-time
    // determined by `T` it should be okay to implement.
}

impl<T, E> StorageSize for Result<T, E>
where
    T: StorageSize,
    E: StorageSize,
{
    const SIZE: u64 = {
        // The following returns the maximum value from the storage
        // sizes of type `T` and `E` in a way that enables it to be used
        // at compile-time.
        [<T as StorageSize>::SIZE, <E as StorageSize>::SIZE]
            [(<T as StorageSize>::SIZE < <E as StorageSize>::SIZE) as usize]
    };
}

impl<T, E> StorageFootprint for Result<T, E>
where
    T: StorageFootprint,
    E: StorageFootprint,
    <T as StorageFootprint>::Value: Max<<E as StorageFootprint>::Value>,
{
    type Value = Maximum<<T as StorageFootprint>::Value, <E as StorageFootprint>::Value>;
}

impl<T, E> SaturatingStorage for Result<T, E>
where
    T: StorageFootprint + SaturatingStorage,
    E: StorageFootprint + SaturatingStorage,
    <T as StorageFootprint>::Value:
        IsEqual<<E as StorageFootprint>::Value, Output = True>,
{
}

impl<T> StorageSize for ink_prelude::boxed::Box<T>
where
    T: StorageSize,
{
    const SIZE: u64 = <T as StorageSize>::SIZE;
}

impl<T> StorageFootprint for ink_prelude::boxed::Box<T>
where
    T: StorageFootprint,
{
    type Value = <T as StorageFootprint>::Value;
}

impl<T> SaturatingStorage for Box<T> where T: SaturatingStorage {}

impl StorageSize for ink_prelude::string::String {
    const SIZE: u64 = 1;
}

impl StorageFootprint for ink_prelude::string::String {
    type Value = P1;
}

impl SaturatingStorage for String {}

impl<T> StorageSize for ink_prelude::vec::Vec<T> {
    const SIZE: u64 = 1;
}

impl<T> StorageFootprint for ink_prelude::vec::Vec<T> {
    type Value = P1;
}

impl<T> SaturatingStorage for ink_prelude::vec::Vec<T> {}

impl<K, V> StorageSize for ink_prelude::collections::BTreeMap<K, V> {
    const SIZE: u64 = 1;
}

impl<K, V> StorageFootprint for ink_prelude::collections::BTreeMap<K, V> {
    type Value = P1;
}

impl<K, V> SaturatingStorage for ink_prelude::collections::BTreeMap<K, V> {}

macro_rules! impl_storage_size_for_collection {
    ( $($name:ident),* $(,)? ) => {
        $(
            impl<T> StorageSize for ink_prelude::collections::$name<T> {
                const SIZE: u64 = 1;
            }

            impl<T> StorageFootprint for ink_prelude::collections::$name<T> {
                type Value = P1;
            }

            impl<T> SaturatingStorage for ink_prelude::collections::$name<T> {}
        )*
    };
}
impl_storage_size_for_collection!(BinaryHeap, BTreeSet, VecDeque, LinkedList,);
