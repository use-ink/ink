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
    ManualKey,
    StorageKeyHolder,
    StorageType,
};
use ink_primitives::StorageKeyComposer;

// The storage key is generated based on the tuple name and field number concatenated with `::`.
// For `(A)` it is `(A)::0`
// For `(A, B)` it is `(A, B)::0`, `(A, B)::1`
// ...
// For `(A, B, ..., J)` it is `(A, B, ..., J)::0`, `(A, B, ..., J)::1`, ..., `(A, B, ..., J)::9`
macro_rules! manual_key {
    ( $tuple_name:expr, $id:literal) => {
        ManualKey<{ StorageKeyComposer::from_str(const_format::concatcp!($tuple_name, "::", stringify!($id))) }, Salt>
    };
}

macro_rules! impl_storage_type_for_tuple {
    ( $(($frag:ident, $id:literal)),* $(,)? ) => {
        const _: () = {
            // The name of the tuple looks like `(A)`, `(A, B)` ... `(A, B, ..., J)`
            const TUPLE_NAME: &'static str = stringify!(($($frag),*));

            impl<$($frag),*> AtomicGuard<true> for ($($frag),* ,)
            where
                $(
                    $frag: AtomicGuard<true>,
                )*
            {}

            impl<$($frag),*, Salt: StorageKeyHolder> StorageType<Salt> for ($($frag),* ,)
            where
                $(
                    $frag: StorageType<manual_key!(TUPLE_NAME, $id)>,
                )*
            {
                type Type = ($(<$frag as StorageType<manual_key!(TUPLE_NAME, $id)>>::Type),* ,);
                type PreferredKey = AutoKey;
            }
        };
    }
}

impl_storage_type_for_tuple!((A, 0));
impl_storage_type_for_tuple!((A, 0), (B, 1));
impl_storage_type_for_tuple!((A, 0), (B, 1), (C, 2));
impl_storage_type_for_tuple!((A, 0), (B, 1), (C, 2), (D, 3));
impl_storage_type_for_tuple!((A, 0), (B, 1), (C, 2), (D, 3), (E, 4));
impl_storage_type_for_tuple!((A, 0), (B, 1), (C, 2), (D, 3), (E, 4), (F, 5));
impl_storage_type_for_tuple!((A, 0), (B, 1), (C, 2), (D, 3), (E, 4), (F, 5), (G, 6));
impl_storage_type_for_tuple!(
    (A, 0),
    (B, 1),
    (C, 2),
    (D, 3),
    (E, 4),
    (F, 5),
    (G, 6),
    (H, 7)
);
impl_storage_type_for_tuple!(
    (A, 0),
    (B, 1),
    (C, 2),
    (D, 3),
    (E, 4),
    (F, 5),
    (G, 6),
    (H, 7),
    (I, 8)
);
impl_storage_type_for_tuple!(
    (A, 0),
    (B, 1),
    (C, 2),
    (D, 3),
    (E, 4),
    (F, 5),
    (G, 6),
    (H, 7),
    (I, 8),
    (J, 9)
);

#[cfg(test)]
mod tests {
    use crate::storage_type_works_for_primitive;

    type TupleSix = (i32, u32, String, u8, bool, Box<Option<i32>>);
    storage_type_works_for_primitive!(TupleSix);
}
