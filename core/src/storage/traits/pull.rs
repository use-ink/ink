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
    ArrayLenLessEquals32,
    KeyPtr,
    StorageSize,
};
use crate::env;
use array_init::{
    array_init,
    IsArray,
};
use core::marker::PhantomData;
use ink_primitives::Key;

/// Types that implement this trait can pull their fields from the associated
/// contract storage in a distributive manner.
///
/// # Note
///
/// This tries to distribute all separate fields of an entity into distinct
/// storage cells.
pub trait PullForward {
    /// Pulls `self` distributedly from the associated contract storage.
    fn pull_forward(ptr: &mut KeyPtr) -> Self;
}

/// Types that implement this trait can pull their fields from an associated
/// contract storage cell.
///
/// # Note
///
/// This tries to compactly load the entire entity's fields from a single
/// contract storage cell.
pub trait PullAt {
    /// Pulls `self` packed from the contract storage cell determined by `at`.
    fn pull_at(at: Key) -> Self;
}

fn pull_single_cell<T>(at: Key) -> T
where
    T: scale::Decode,
{
    crate::env::get_contract_storage::<T>(at)
        .expect("storage entry was empty")
        .expect("could not properly decode storage entry")
}

macro_rules! impl_pull_for_primitive {
    ( $($ty:ty),* $(,)? ) => {
        $(
            impl PullForward for $ty {
                fn pull_forward(ptr: &mut KeyPtr) -> Self {
                    <Self as PullAt>::pull_at(ptr.next_for::<Self>())
                }
            }
            impl PullAt for $ty {
                fn pull_at(at: Key) -> Self {
                    pull_single_cell::<$ty>(at)
                }
            }
        )*
    };
}
impl_pull_for_primitive!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128);

macro_rules! impl_pull_for_array {
    ( $($len:literal),* $(,)? ) => {
        $(
            impl<T> PullForward for [T; $len]
            where
                Self: IsArray + ArrayLenLessEquals32,
                <Self as IsArray>::Item: PullForward,
            {
                fn pull_forward(ptr: &mut KeyPtr) -> Self {
                    array_init::<Self, _>(|_| PullForward::pull_forward(ptr))
                }
            }

            impl<T> PullAt for [T; $len]
            where
                [T; $len]: scale::Decode,
            {
                fn pull_at(at: Key) -> Self {
                    pull_single_cell::<[T; $len]>(at)
                }
            }
        )*
    }
}
forward_supported_array_lens!(impl_pull_for_array);

macro_rules! impl_pull_tuple {
    ( $($frag:ident),* $(,)? ) => {
        impl<$($frag),*> PullForward for ($($frag),* ,)
        where
            $(
                $frag: PullForward,
            )*
        {
            fn pull_forward(ptr: &mut KeyPtr) -> Self {
                (
                    $(
                        <$frag as PullForward>::pull_forward(ptr),
                    )*
                )
            }
        }

        impl<$($frag),*> PullAt for ($($frag),* ,)
        where
            Self: scale::Decode,
        {
            fn pull_at(at: Key) -> Self {
                pull_single_cell::<Self>(at)
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

impl PullForward for () {
    fn pull_forward(_ptr: &mut KeyPtr) -> Self {
        ()
    }
}

impl PullAt for () {
    fn pull_at(_at: Key) -> Self {
        ()
    }
}

impl<T> PullForward for PhantomData<T> {
    fn pull_forward(_ptr: &mut KeyPtr) -> Self {
        Default::default()
    }
}

impl<T> PullAt for PhantomData<T> {
    fn pull_at(_at: Key) -> Self {
        Default::default()
    }
}

impl<T> PullForward for Option<T>
where
    T: PullForward + StorageSize,
{
    fn pull_forward(ptr: &mut KeyPtr) -> Self {
        // We decode as `()` because at this point we are not interested
        // in the actual value, we just want to know if there exists a value
        // at all.
        match env::get_contract_storage::<()>(ptr.current()) {
            Some(_) => Some(<T as PullForward>::pull_forward(ptr)),
            None => None,
        }
    }
}

impl<T> PullAt for Option<T>
where
    Self: scale::Decode,
{
    fn pull_at(at: Key) -> Self {
        pull_single_cell::<Self>(at)
    }
}

impl<T, E> PullForward for Result<T, E>
where
    T: PullForward,
    E: PullForward,
{
    fn pull_forward(ptr: &mut KeyPtr) -> Self {
        match pull_single_cell::<u8>(ptr.next_for::<u8>()) {
            0 => Ok(<T as PullForward>::pull_forward(ptr)),
            1 => Err(<E as PullForward>::pull_forward(ptr)),
            _ => unreachable!("found invalid Result discriminator"),
        }
    }
}

impl<T, E> PullAt for Result<T, E>
where
    Self: scale::Decode,
{
    fn pull_at(at: Key) -> Self {
        pull_single_cell::<Self>(at)
    }
}

impl PullForward for ink_prelude::string::String {
    fn pull_forward(ptr: &mut KeyPtr) -> Self {
        <Self as PullAt>::pull_at(ptr.next_for::<Self>())
    }
}

impl PullAt for ink_prelude::string::String {
    fn pull_at(at: Key) -> Self {
        pull_single_cell::<Self>(at)
    }
}

impl<T> PullForward for ink_prelude::boxed::Box<T>
where
    T: PullForward,
{
    fn pull_forward(ptr: &mut KeyPtr) -> Self {
        Self::new(<T as PullForward>::pull_forward(ptr))
    }
}

impl<T> PullAt for ink_prelude::boxed::Box<T>
where
    T: PullAt,
{
    fn pull_at(at: Key) -> Self {
        Self::new(<T as PullAt>::pull_at(at))
    }
}

const _: () = {
    use ink_prelude::{
        collections::{
            BTreeSet,
            BinaryHeap,
            LinkedList,
            VecDeque,
        },
    };
    #[cfg(not(feature = "std"))]
    use ink_prelude::vec::Vec;

    macro_rules! impl_pull_at_for_collection {
        ( $($collection:ident),* $(,)? ) => {
            $(
                impl<T> PullAt for $collection<T>
                where
                    Self: scale::Decode,
                {
                    fn pull_at(at: Key) -> Self {
                        pull_single_cell::<Self>(at)
                    }
                }
            )*
        };
    }
    impl_pull_at_for_collection!(
        Vec,
        BTreeSet,
        BinaryHeap,
        LinkedList,
        VecDeque,
    );
};

impl<K, V> PullAt for ink_prelude::collections::BTreeMap<K, V>
where
    Self: scale::Decode,
{
    fn pull_at(at: Key) -> Self {
        pull_single_cell::<Self>(at)
    }
}
