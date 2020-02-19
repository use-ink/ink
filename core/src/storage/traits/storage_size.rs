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

/// Implemented by types that can be stored on contract storage.
///
/// Tells the amount of storage cells the type requires to be stored.
pub trait StorageSize {
    /// The number of storage cells required by `Self` to be stored
    /// on the contract storage.
    const SIZE: u64;
}

macro_rules! impl_storage_size_for_primitive {
    ( $($ty:ty),* ) => {
        $(
            impl StorageSize for $ty {
                const SIZE: u64 = 1;
            }
        )*
    };
}
impl_storage_size_for_primitive!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128);

macro_rules! impl_storage_size_for_array {
    ( $($n:literal),* $(,)? ) => {
        $(
            impl<T> StorageSize for [T; $n]
            where
                T: StorageSize,
            {
                const SIZE: u64 = <T as StorageSize>::SIZE * $n;
            }
        )*
    };
}
forward_supported_array_lens!(impl_storage_size_for_array);

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

impl StorageSize for () {
    const SIZE: u64 = 0;
}

impl<T> StorageSize for core::marker::PhantomData<T> {
    const SIZE: u64 = 0;
}

impl<T> StorageSize for Option<T>
where
    T: StorageSize,
{
    const SIZE: u64 = <T as StorageSize>::SIZE;
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

impl<T> StorageSize for ink_prelude::boxed::Box<T>
where
    T: StorageSize,
{
    const SIZE: u64 = <T as StorageSize>::SIZE;
}

impl StorageSize for ink_prelude::string::String {
    const SIZE: u64 = 1;
}
