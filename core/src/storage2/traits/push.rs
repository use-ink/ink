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

/// Types that implement this trait can push their fields to the associated
/// contract storage in a distributive manner.
///
/// # Note
///
/// This tries to distribute all separate fields of an entity into distinct
/// storage cells.
pub trait PushForward {
    /// Pushes `self` distributed to the associated contract storage.
    fn push_forward(&self, ptr: &mut KeyPtr);
}

/// Types that implement this trait can push their fields to an associated
/// contract storage cell.
///
/// # Note
///
/// This tries to compactly store the entire entity's fields into a single
/// contract storage cell.
pub trait PushAt {
    /// Pushes `self` packed to the contract storage cell determined by `at`.
    fn push_at(&self, at: Key);
}

macro_rules! impl_push_for_primitive {
    ( $($ty:ty),* $(,)? ) => {
        $(
            impl PushForward for $ty {
                fn push_forward(&self, ptr: &mut KeyPtr) {
                    <$ty as PushAt>::push_at(self, ptr.next_for::<$ty>())
                }
            }

            impl PushAt for $ty {
                fn push_at(&self, at: Key) {
                    env::set_contract_storage::<$ty>(at, self)
                }
            }
        )*
    };
}
impl_push_for_primitive!(Key, u8, u16, u32, u64, u128, i8, i16, i32, i64, i128);

macro_rules! impl_push_for_array {
    ( $($len:literal),* $(,)? ) => {
        $(
            impl<T> PushForward for [T; $len]
            where
                T: PushForward,
            {
                fn push_forward(&self, ptr: &mut KeyPtr) {
                    // TODO: Insert check to assert that the array length is
                    //       32 or smaller since otherwise this is a rather
                    //       expensive operation that it shouldn't be.
                    //       Arrays should generally be used in packed form.
                    for elem in self.iter() {
                        <T as PushForward>::push_forward(elem, ptr)
                    }
                }
            }

            impl<T> PushAt for [T; $len]
            where
                [T; $len]: scale::Encode,
            {
                fn push_at(&self, at: Key) {
                    env::set_contract_storage::<[T; $len]>(at, self)
                }
            }
        )*
    }
}
forward_supported_array_lens!(impl_push_for_array);

macro_rules! impl_push_tuple {
    ( $($frag:ident),* $(,)? ) => {
        impl<$($frag),*> PushForward for ($($frag),* ,)
        where
            $(
                $frag: PushForward,
            )*
        {
            fn push_forward(&self, ptr: &mut KeyPtr) {
                #[allow(non_snake_case)]
                let ($($frag),*,) = self;
                $(
                    <$frag as PushForward>::push_forward($frag, ptr);
                )*
            }
        }

        impl<$($frag),*> PushAt for ($($frag),* ,)
        where
            Self: scale::Encode,
        {
            fn push_at(&self, at: Key) {
                env::set_contract_storage::<Self>(at, self)
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

impl PushForward for () {
    fn push_forward(&self, _ptr: &mut KeyPtr) {}
}

impl PushAt for () {
    fn push_at(&self, _at: Key) {}
}

impl<T> PushForward for PhantomData<T> {
    fn push_forward(&self, _ptr: &mut KeyPtr) {}
}

impl<T> PushAt for PhantomData<T> {
    fn push_at(&self, _at: Key) {}
}

impl<T> PushForward for Option<T>
where
    T: PushForward + StorageFootprint,
{
    /// We implement `PushForward` for `Option<T>` in an optimized fashion
    /// leaving behind a cleared contract storage cell area in case of `None`.
    ///
    /// This can be tricky and a performance hazard for types that occupy
    /// a large amount of storage cells (i.e. their `StorageSize::SIZE` is
    /// big.). This implementation needs protection against this sort of hazard.
    fn push_forward(&self, ptr: &mut KeyPtr) {
        match self {
            Some(val) => <T as PushForward>::push_forward(val, ptr),
            None => {
                // We still need to advance the key pointer.
                let pos0 = ptr.next_for::<T>();
                // Bail out early if `StorageSize` is too big and the method
                // is used even though we have tried to prevent this at compile
                // time.
                if <T as StorageFootprint>::VALUE > 32 {
                    return
                }
                // # ToDo
                //
                // Create a trait bound onto something like
                // `ClearForward` and `ClearAt` that have a sole purpose of
                // clearing the underlying storage of a storage entity.
                for n in 0..<T as StorageFootprint>::VALUE {
                    env::clear_contract_storage(pos0 + n);
                }
            }
        }
    }
}

impl<T> PushAt for Option<T>
where
    T: PushAt,
{
    fn push_at(&self, at: Key) {
        match self {
            Some(val) => <T as PushAt>::push_at(val, at),
            None => env::clear_contract_storage(at),
        }
    }
}

impl<T, E> PushForward for Result<T, E>
where
    T: PushForward,
    E: PushForward,
{
    fn push_forward(&self, ptr: &mut KeyPtr) {
        match self {
            Ok(val) => {
                PushForward::push_forward(&0u8, ptr);
                <T as PushForward>::push_forward(val, ptr);
            }
            Err(err) => {
                PushForward::push_forward(&1u8, ptr);
                <E as PushForward>::push_forward(err, ptr);
            }
        }
    }
}

impl<T, E> PushAt for Result<T, E>
where
    Self: scale::Encode,
{
    fn push_at(&self, at: Key) {
        env::set_contract_storage::<Self>(at, self)
    }
}

// The `PushForward` and `PullForward` traits are not implemented for
//
// - `ink_prelude::vec::Vec<T>`
// - `ink_prelude::collections::BTreeSet<T>`
// - `ink_prelude::collections::BinaryHeap<T>`
// - `ink_prelude::collections::LinkedList<T>`
// - `ink_prelude::collections::VecDeque<T>`
//
// since their storage sizes cannot be determined during compilation time as
// every element in them would potentially occupy its own storage cell and there
// can be arbitrary many elements at runtime.

impl PushForward for ink_prelude::string::String {
    fn push_forward(&self, ptr: &mut KeyPtr) {
        <Self as PushAt>::push_at(self, ptr.next_for::<Self>())
    }
}

impl PushAt for ink_prelude::string::String {
    fn push_at(&self, at: Key) {
        env::set_contract_storage::<Self>(at, self)
    }
}

impl<T> PushForward for ink_prelude::boxed::Box<T>
where
    T: PushForward,
{
    fn push_forward(&self, ptr: &mut KeyPtr) {
        <T as PushForward>::push_forward(&*self, ptr)
    }
}

impl<T> PushAt for ink_prelude::boxed::Box<T>
where
    T: PushAt,
{
    fn push_at(&self, at: Key) {
        <T as PushAt>::push_at(&*self, at)
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

    macro_rules! impl_push_at_for_collection {
        ( $($collection:ident),* $(,)? ) => {
            $(
                impl<T> PushAt for $collection<T>
                where
                    Self: scale::Encode,
                {
                    fn push_at(&self, at: Key) {
                        env::set_contract_storage::<Self>(at, self)
                    }
                }
            )*
        };
    }
    impl_push_at_for_collection!(Vec, BTreeSet, BinaryHeap, LinkedList, VecDeque,);
};

impl<K, V> PushAt for ink_prelude::collections::BTreeMap<K, V>
where
    Self: scale::Encode,
{
    fn push_at(&self, at: Key) {
        env::set_contract_storage::<Self>(at, self)
    }
}
