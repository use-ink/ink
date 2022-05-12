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
    AutoKey,
    StorageKeyHolder,
    StorageType,
};

macro_rules! impl_storage_type_for_tuple {
    ( $($frag:ident),* $(,)? ) => {
        impl<$($frag),*> AtomicGuard<true> for ($($frag),* ,)
        where
            $(
                $frag: AtomicGuard<true>,
            )*
        {}

        impl<$($frag),*, Salt: StorageKeyHolder> StorageType<Salt> for ($($frag),* ,)
        where
            $(
                $frag: StorageType<Salt>,
            )*
        {
            type Type = ($(<$frag as StorageType<Salt>>::Type),* ,);
            type PreferredKey = AutoKey;
        }
    }
}
impl_storage_type_for_tuple!(A);
impl_storage_type_for_tuple!(A, B);
impl_storage_type_for_tuple!(A, B, C);
impl_storage_type_for_tuple!(A, B, C, D);
impl_storage_type_for_tuple!(A, B, C, D, E);
impl_storage_type_for_tuple!(A, B, C, D, E, F);
impl_storage_type_for_tuple!(A, B, C, D, E, F, G);
impl_storage_type_for_tuple!(A, B, C, D, E, F, G, H);
impl_storage_type_for_tuple!(A, B, C, D, E, F, G, H, I);
impl_storage_type_for_tuple!(A, B, C, D, E, F, G, H, I, J);

#[cfg(test)]
mod tests {
    use crate::storage_type_works_for_primitive;

    type TupleSix = (i32, u32, String, u8, bool, Box<Option<i32>>);
    storage_type_works_for_primitive!(TupleSix);
}
