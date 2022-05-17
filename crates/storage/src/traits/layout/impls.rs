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
    StorageLayout,
};
use ink_env::{
    AccountId,
    Hash,
};
use ink_metadata::layout::{
    ArrayLayout,
    CellLayout,
    Discriminant,
    EnumLayout,
    FieldLayout,
    Layout,
    LayoutKey,
    StructLayout,
};
use ink_prelude::{
    boxed::Box,
    collections::{
        BTreeMap,
        BTreeSet,
        VecDeque,
    },
    string::String,
    vec::Vec,
};
use ink_primitives::{
    Key,
    StorageKey,
};
use scale_info::TypeInfo;

macro_rules! impl_storage_layout_for_primitives {
    ( $($name:ty),* $(,)? ) => {
        $(
            impl StorageLayout for $name {
                fn layout(key: &StorageKey) -> Layout {
                    Layout::Leaf(CellLayout::new::<$name>(LayoutKey::from(key)))
                }
            }
        )*
    };
}
#[rustfmt::skip]
impl_storage_layout_for_primitives!(
    Key, Hash, AccountId, String,
    bool, char, (),
    u8, u16, u32, u64, u128,
    i8, i16, i32, i64, i128,
);

macro_rules! impl_storage_layout_for_arrays {
    ( $($len:literal),* $(,)? ) => {
        $(
            impl<T> StorageLayout for [T; $len]
            where
                T: StorageLayout + AtomicGuard<true>,
            {
                fn layout(key: &StorageKey) -> Layout {
                    let len: u32 = $len;
                    // Generic type is atomic, so it doesn't take any cell
                    Layout::Array(ArrayLayout::new(
                        LayoutKey::from(key),
                        len,
                        <T as StorageLayout>::layout(&key),
                    ))
                }
            }
        )*
    };
}
#[rustfmt::skip]
impl_storage_layout_for_arrays!(
         1,  2,  3,  4,  5,  6,  7,  8,  9,
    10, 11, 12, 13, 14, 15, 16, 17, 18, 19,
    20, 21, 22, 23, 24, 25, 26, 27, 28, 29,
    30, 31, 32,
);

macro_rules! impl_layout_for_tuple {
    ( $(($frag:ident, $id:literal)),* $(,)? ) => {
        const _: () = {
            // The name of the tuple looks like `(A)`, `(A, B)` ... `(A, B, ..., J)`
            const TUPLE_NAME: &'static str = stringify!(($($frag),*));

            impl<$($frag),*> StorageLayout for ($($frag),* ,)
            where
                $(
                    $frag: StorageLayout,
                )*
            {
                fn layout(key: &StorageKey) -> Layout {
                    Layout::Struct(
                        StructLayout::new(
                            TUPLE_NAME,
                            [
                                $(
                                    FieldLayout::new(
                                        ::core::stringify!($id),
                                        <$frag as StorageLayout>::layout(key)
                                    ),
                                )*
                            ]
                        )
                    )
                }
            }
        };
    }
}

impl_layout_for_tuple!((A, 0));
impl_layout_for_tuple!((A, 0), (B, 1));
impl_layout_for_tuple!((A, 0), (B, 1), (C, 2));
impl_layout_for_tuple!((A, 0), (B, 1), (C, 2), (D, 3));
impl_layout_for_tuple!((A, 0), (B, 1), (C, 2), (D, 3), (E, 4));
impl_layout_for_tuple!((A, 0), (B, 1), (C, 2), (D, 3), (E, 4), (F, 5));
impl_layout_for_tuple!((A, 0), (B, 1), (C, 2), (D, 3), (E, 4), (F, 5), (G, 6));
impl_layout_for_tuple!(
    (A, 0),
    (B, 1),
    (C, 2),
    (D, 3),
    (E, 4),
    (F, 5),
    (G, 6),
    (H, 7)
);
impl_layout_for_tuple!(
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
impl_layout_for_tuple!(
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

impl<T> StorageLayout for Box<T>
where
    T: StorageLayout,
{
    fn layout(key: &StorageKey) -> Layout {
        <T as StorageLayout>::layout(key)
    }
}

impl<T> StorageLayout for Option<T>
where
    T: StorageLayout,
{
    fn layout(key: &StorageKey) -> Layout {
        Layout::Enum(EnumLayout::new(
            "Option",
            key,
            [
                (Discriminant::from(0), StructLayout::new("None", Vec::new())),
                (
                    Discriminant::from(1),
                    StructLayout::new(
                        "Some",
                        [FieldLayout::new("0", <T as StorageLayout>::layout(key))],
                    ),
                ),
            ],
        ))
    }
}

impl<T, E> StorageLayout for Result<T, E>
where
    T: StorageLayout,
    E: StorageLayout,
{
    fn layout(key: &StorageKey) -> Layout {
        Layout::Enum(EnumLayout::new(
            "Result",
            *key,
            [
                (
                    Discriminant::from(0),
                    StructLayout::new(
                        "Ok",
                        [FieldLayout::new("0", <T as StorageLayout>::layout(key))],
                    ),
                ),
                (
                    Discriminant::from(1),
                    StructLayout::new(
                        "Err",
                        [FieldLayout::new("1", <E as StorageLayout>::layout(key))],
                    ),
                ),
            ],
        ))
    }
}

impl<T> StorageLayout for Vec<T>
where
    T: TypeInfo + 'static + AtomicGuard<true>,
{
    fn layout(key: &StorageKey) -> Layout {
        Layout::Leaf(CellLayout::new::<Self>(LayoutKey::from(key)))
    }
}

impl<K, V> StorageLayout for BTreeMap<K, V>
where
    K: TypeInfo + 'static + AtomicGuard<true>,
    V: TypeInfo + 'static + AtomicGuard<true>,
{
    fn layout(key: &StorageKey) -> Layout {
        Layout::Leaf(CellLayout::new::<Self>(LayoutKey::from(key)))
    }
}

impl<T> StorageLayout for BTreeSet<T>
where
    T: TypeInfo + 'static + AtomicGuard<true>,
{
    fn layout(key: &StorageKey) -> Layout {
        Layout::Leaf(CellLayout::new::<Self>(LayoutKey::from(key)))
    }
}

impl<T> StorageLayout for VecDeque<T>
where
    T: TypeInfo + 'static + AtomicGuard<true>,
{
    fn layout(key: &StorageKey) -> Layout {
        Layout::Leaf(CellLayout::new::<Self>(LayoutKey::from(key)))
    }
}
