// Copyright 2018-2021 Parity Technologies (UK) Ltd.
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

use super::StorageLayout;
use crate::traits::{
    ExtKeyPtr as _,
    KeyPtr,
    SpreadLayout,
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
    collections::BTreeMap,
    string::String,
    vec::Vec,
};
use ink_primitives::Key;
use scale_info::TypeInfo;

macro_rules! impl_storage_layout_for_primitives {
    ( $($name:ty),* $(,)? ) => {
        $(
            impl StorageLayout for $name {
                fn layout(key_ptr: &mut KeyPtr) -> Layout {
                    Layout::Cell(CellLayout::new::<$name>(LayoutKey::from(key_ptr.advance_by(1))))
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
                T: StorageLayout + SpreadLayout,
            {
                fn layout(key_ptr: &mut KeyPtr) -> Layout {
                    let len: u32 = $len;
                    let elem_footprint = <T as SpreadLayout>::FOOTPRINT;
                    Layout::Array(ArrayLayout::new(
                        LayoutKey::from(key_ptr.next_for::<[T; $len]>()),
                        len,
                        elem_footprint,
                        <T as StorageLayout>::layout(&mut key_ptr.clone()),
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
    ( $($frag:ident),* $(,)? ) => {
        impl<$($frag),*> StorageLayout for ($($frag),* ,)
        where
            $(
                $frag: StorageLayout,
            )*
        {
            fn layout(key_ptr: &mut KeyPtr) -> Layout {
                Layout::Struct(
                    StructLayout::new(vec![
                        $(
                            FieldLayout::new(None, <$frag as StorageLayout>::layout(key_ptr)),
                        )*
                    ])
                )
            }
        }
    }
}
impl_layout_for_tuple!(A);
impl_layout_for_tuple!(A, B);
impl_layout_for_tuple!(A, B, C);
impl_layout_for_tuple!(A, B, C, D);
impl_layout_for_tuple!(A, B, C, D, E);
impl_layout_for_tuple!(A, B, C, D, E, F);
impl_layout_for_tuple!(A, B, C, D, E, F, G);
impl_layout_for_tuple!(A, B, C, D, E, F, G, H);
impl_layout_for_tuple!(A, B, C, D, E, F, G, H, I);
impl_layout_for_tuple!(A, B, C, D, E, F, G, H, I, J);

impl<T> StorageLayout for Box<T>
where
    T: StorageLayout,
{
    fn layout(key_ptr: &mut KeyPtr) -> Layout {
        <T as StorageLayout>::layout(key_ptr)
    }
}

impl<T> StorageLayout for Option<T>
where
    T: StorageLayout,
{
    fn layout(key_ptr: &mut KeyPtr) -> Layout {
        let dispatch_key = key_ptr.advance_by(1);
        Layout::Enum(EnumLayout::new(
            *dispatch_key,
            vec![
                (
                    Discriminant::from(0),
                    StructLayout::new(vec![FieldLayout::new(
                        None,
                        <T as StorageLayout>::layout(&mut key_ptr.clone()),
                    )]),
                ),
                (Discriminant::from(1), StructLayout::new(Vec::new())),
            ],
        ))
    }
}

impl<T, E> StorageLayout for Result<T, E>
where
    T: StorageLayout,
    E: StorageLayout,
{
    fn layout(key_ptr: &mut KeyPtr) -> Layout {
        let dispatch_key = key_ptr.advance_by(1);
        Layout::Enum(EnumLayout::new(
            *dispatch_key,
            vec![
                (
                    Discriminant::from(0),
                    StructLayout::new(vec![FieldLayout::new(
                        None,
                        <T as StorageLayout>::layout(&mut key_ptr.clone()),
                    )]),
                ),
                (
                    Discriminant::from(1),
                    StructLayout::new(vec![FieldLayout::new(
                        None,
                        <E as StorageLayout>::layout(&mut key_ptr.clone()),
                    )]),
                ),
            ],
        ))
    }
}

impl<T> StorageLayout for Vec<T>
where
    T: TypeInfo + 'static,
{
    fn layout(key_ptr: &mut KeyPtr) -> Layout {
        Layout::Cell(CellLayout::new::<Self>(LayoutKey::from(
            key_ptr.advance_by(1),
        )))
    }
}

impl<K, V> StorageLayout for BTreeMap<K, V>
where
    K: TypeInfo + 'static,
    V: TypeInfo + 'static,
{
    fn layout(key_ptr: &mut KeyPtr) -> Layout {
        Layout::Cell(CellLayout::new::<Self>(LayoutKey::from(
            key_ptr.advance_by(1),
        )))
    }
}
