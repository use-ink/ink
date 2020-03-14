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

use core::ops::{
    Add,
    Mul,
};
use typenum::{
    Integer,
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
    type Value: Integer;
}

/// Helper type alias for better readability.
pub type StorageFootprintOf<T> = <T as StorageFootprint>::Value;

/// Returns the `u64` representation of the storage footprint of `T`.
pub const fn storage_footprint_u64<T>() -> u64
where
    T: StorageFootprint,
{
    <StorageFootprintOf<T> as Integer>::I64 as u64
}

/// Returns the `u128` representation of the storage footprint of `T`.
pub const fn storage_footprint_u128<T>() -> u128
where
    T: StorageFootprint,
{
    <StorageFootprintOf<T> as Integer>::I128 as u128
}

/// Types implementing this trait are guaranteed to always use the same amount
/// of storage cells as described by the [`StorageFootprint`] trait.
///
/// It is a bug to implement this trait for a type that does not respect this
/// behaviour.
pub trait SaturatingStorage: StorageFootprint {}

/// Helper trait that should be implemented for types instead of implementing
/// [`SaturatingStorage`] trait directly since it decouples the trait bounds
/// of its super trait [`StorageFootprint`].
pub trait SaturatingStorageMarker {}
impl<T> SaturatingStorage for T where T: StorageFootprint + SaturatingStorageMarker {}

macro_rules! impl_storage_size_for_primitive {
    ( $($ty:ty),* ) => {
        $(
            impl StorageFootprint for $ty {
                type Value = P1;
            }
            impl SaturatingStorageMarker for $ty {}
        )*
    };
}
impl_storage_size_for_primitive!(Key, u8, u16, u32, u64, u128, i8, i16, i32, i64, i128);

macro_rules! impl_storage_size_for_array2 {
    ( $(($n:literal, $t:ty)),* $(,)? ) => {
        $(
            impl<T> StorageFootprint for [T; $n]
            where
                T: StorageFootprint,
                StorageFootprintOf<T>: Mul<$t>,
                Prod<StorageFootprintOf<T>, $t>: Integer,
            {
                type Value = Prod<StorageFootprintOf<T>, $t>;
            }
            impl<T> SaturatingStorageMarker for [T; $n]
            where
                T: SaturatingStorage,
            {}
        )*
    };
}
forward_supported_array_lens_ty!(impl_storage_size_for_array2);

macro_rules! impl_storage_size_tuple {
    ( $($frag:ident),* ) => {
        impl<T1 $(, $frag)*> SaturatingStorageMarker for (T1 $(, $frag)* ,)
        where
            T1: SaturatingStorage,
            $(
                $frag: SaturatingStorage,
            )*
        {}

        impl<T1 $(, $frag)*> StorageFootprint for (T1 $(, $frag)* ,)
        where
            T1: StorageFootprint,
            ($($frag ,)*): StorageFootprint,
            StorageFootprintOf<T1>: Add<Z0>, // Not sure why we need this trait bound for T1 ...
            StorageFootprintOf<T1>: Add<StorageFootprintOf<($($frag ,)*)>>,
            Sum<StorageFootprintOf<T1>, StorageFootprintOf<($($frag ,)*)>>: Integer,
        {
            type Value = Sum<StorageFootprintOf<T1>, StorageFootprintOf<($($frag ,)*)>>;
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
    type Value = Z0;
}
impl SaturatingStorage for () {}

impl<T> StorageFootprint for core::marker::PhantomData<T> {
    type Value = Z0;
}
impl<T> SaturatingStorage for core::marker::PhantomData<T> {}

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

impl<T, E> StorageFootprint for Result<T, E>
where
    T: StorageFootprint,
    E: StorageFootprint,
    StorageFootprintOf<T>: Max<StorageFootprintOf<E>>,
    Maximum<StorageFootprintOf<T>, StorageFootprintOf<E>>: Integer,
{
    type Value = Maximum<StorageFootprintOf<T>, StorageFootprintOf<E>>;
}

impl<T, E> SaturatingStorageMarker for Result<T, E>
where
    T: StorageFootprint + SaturatingStorage,
    E: StorageFootprint + SaturatingStorage,
    StorageFootprintOf<T>: IsEqual<StorageFootprintOf<E>, Output = True>,
{
}

impl<T> StorageFootprint for ink_prelude::boxed::Box<T>
where
    T: StorageFootprint,
{
    type Value = StorageFootprintOf<T>;
}

impl<T> SaturatingStorageMarker for Box<T> where T: SaturatingStorage {}

impl StorageFootprint for ink_prelude::string::String {
    type Value = P1;
}

impl SaturatingStorage for String {}

impl<T> StorageFootprint for ink_prelude::vec::Vec<T> {
    type Value = P1;
}

impl<T> SaturatingStorage for ink_prelude::vec::Vec<T> {}

impl<K, V> StorageFootprint for ink_prelude::collections::BTreeMap<K, V> {
    type Value = P1;
}

impl<K, V> SaturatingStorage for ink_prelude::collections::BTreeMap<K, V> {}

macro_rules! impl_storage_size_for_collection {
    ( $($name:ident),* $(,)? ) => {
        $(
            impl<T> StorageFootprint for ink_prelude::collections::$name<T> {
                type Value = P1;
            }

            impl<T> SaturatingStorage for ink_prelude::collections::$name<T> {}
        )*
    };
}
impl_storage_size_for_collection!(BinaryHeap, BTreeSet, VecDeque, LinkedList,);
