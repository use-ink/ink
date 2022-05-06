// Copyright 2018-2022 Parity Technologies (UK) Ltd.
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
    AtomicGuard,
    StorageKeyHolder,
    StorageType,
};
use ink_env::{
    AccountId,
    Hash,
};
use ink_prelude::{
    boxed::Box,
    string::String,
};
use ink_primitives::{
    Key,
    StorageKey,
};

macro_rules! impl_storage_type_for_primitive {
    ( $($ty:ty),* $(,)? ) => {
        $(
            impl_always_storage_type!($ty);
        )*
    };
}

#[rustfmt::skip]
impl_storage_type_for_primitive!(
    // We do not include `f32` and `f64` since Wasm contracts currently
    // do not support them since they are non deterministic. We might add them
    // to this list once we add deterministic support for those primitives.
    Key, Hash, AccountId, (),
    String,
    bool,
    u8, u16, u32, u64, u128,
    i8, i16, i32, i64, i128,
);

impl StorageKeyHolder for () {
    const KEY: StorageKey = 0;
}

impl<T: AtomicGuard<true>> AtomicGuard<true> for Option<T> {}

impl<T: StorageType<Salt>, Salt: StorageKeyHolder> StorageType<Salt> for Option<T> {
    type Type = Option<<T as StorageType<Salt>>::Type>;
}

impl<T: AtomicGuard<true>, E: AtomicGuard<true>> AtomicGuard<true> for Result<T, E> {}

impl<T: StorageType<Salt>, E: StorageType<Salt>, Salt: StorageKeyHolder> StorageType<Salt>
    for Result<T, E>
{
    type Type = Result<<T as StorageType<Salt>>::Type, <E as StorageType<Salt>>::Type>;
}

impl<T: AtomicGuard<true>> AtomicGuard<true> for Box<T> {}

impl<T: StorageType<Salt>, Salt: StorageKeyHolder> StorageType<Salt> for Box<T> {
    type Type = Box<<T as StorageType<Salt>>::Type>;
}

#[cfg(test)]
mod tests {
    use crate::storage_type_works_for_primitive;
    use ink_env::AccountId;
    use ink_primitives::Key;

    storage_type_works_for_primitive!(bool);
    storage_type_works_for_primitive!(String);
    storage_type_works_for_primitive!(Key);
    storage_type_works_for_primitive!(AccountId);
    storage_type_works_for_primitive!(i8);
    storage_type_works_for_primitive!(i16);
    storage_type_works_for_primitive!(i32);
    storage_type_works_for_primitive!(i64);
    storage_type_works_for_primitive!(i128);
    storage_type_works_for_primitive!(u8);
    storage_type_works_for_primitive!(u16);
    storage_type_works_for_primitive!(u32);
    storage_type_works_for_primitive!(u64);
    storage_type_works_for_primitive!(u128);

    type OptionU8 = Option<u8>;
    storage_type_works_for_primitive!(OptionU8);

    type ResultU8 = Result<u8, bool>;
    storage_type_works_for_primitive!(ResultU8);

    type BoxU8 = Box<u8>;
    storage_type_works_for_primitive!(BoxU8);

    type BoxOptionU8 = Box<Option<u8>>;
    storage_type_works_for_primitive!(BoxOptionU8);
}
