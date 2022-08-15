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

//! Custom derive for `ink_storage` traits.
//!
//! This crate provides helpers to define your very own custom storage data
//! structures that work along the `ink_storage` data structures.

extern crate proc_macro;

mod item;
mod key_holder;
mod storage_layout;

#[cfg(test)]
mod tests;

use self::{
    item::item_derive,
    key_holder::key_holder_derive,
    storage_layout::storage_layout_derive,
};
synstructure::decl_derive!(
    [Item] =>
    /// Derives `ink_storage`'s `Item` trait for the given `struct` or `enum`.
    ///
    /// # Examples
    ///
    /// ```
    /// use ink_storage::traits::{
    ///     Item,
    ///     StorageKey,
    ///     AutoItem,
    ///     AutoKey,
    ///     ManualKey,
    ///     Storable,
    /// };
    ///
    /// #[derive(Default, Item, Storable)]
    /// struct NamedFields {
    ///     a: u32,
    ///     b: [u32; 32],
    /// }
    ///
    /// let _: NamedFields = <NamedFields as Item<AutoKey>>::Type::default();
    /// let _: NamedFields = <NamedFields as Item<ManualKey<123>>>::Type::default();
    ///
    /// #[derive(Item, StorageKey, Storable)]
    /// struct NamedFieldsStorage<KEY: ink_storage::traits::StorageKey> {
    ///     a: <u32 as AutoItem<ManualKey<0, KEY>>>::Type,
    ///     b: <[u32; 32] as AutoItem<ManualKey<1, KEY>>>::Type,
    /// }
    ///
    /// // (AutoKey | ManualKey<123>) -> ManualKey<123>
    /// assert_eq!(123, <NamedFieldsStorage<AutoKey> as AutoItem<ManualKey<123>>>::Type::KEY);
    /// // (ManualKey<321> | ManualKey<123>) -> ManualKey<321>
    /// assert_eq!(321, <NamedFieldsStorage<ManualKey<321>> as AutoItem<ManualKey<123>>>::Type::KEY);
    /// ```
    item_derive
);
synstructure::decl_derive!(
    [StorageKey] =>
    /// Derives `ink_storage`'s `StorageKey` trait for the given `struct` or `enum`.
    ///
    /// # Examples
    ///
    /// ```
    /// use ink_storage::traits::{
    ///     AutoItem,
    ///     StorageKey,
    ///     ManualKey,
    ///     AutoKey,
    /// };
    ///
    /// #[derive(StorageKey)]
    /// struct NamedFields {
    ///     a: u32,
    ///     b: [u32; 32],
    /// }
    ///
    /// assert_eq!(<NamedFields as StorageKey>::KEY, 0);
    ///
    /// #[derive(StorageKey)]
    /// struct NamedFieldsManualKey<KEY: StorageKey> {
    ///     a: <u32 as AutoItem<ManualKey<0, KEY>>>::Type,
    ///     b: <[u32; 32] as AutoItem<ManualKey<1, KEY>>>::Type,
    /// }
    ///
    /// assert_eq!(<NamedFieldsManualKey<()> as StorageKey>::KEY, 0);
    /// assert_eq!(<NamedFieldsManualKey<AutoKey> as StorageKey>::KEY, 0);
    /// assert_eq!(<NamedFieldsManualKey<ManualKey<123>> as StorageKey>::KEY, 123);
    /// ```
    key_holder_derive
);
synstructure::decl_derive!(
    [StorageLayout] =>
    /// Derives `ink_storage`'s `StorageLayout` trait for the given `struct` or `enum`.
    ///
    /// # Examples
    ///
    /// ```
    /// use ink_metadata::layout::Layout::Struct;
    /// use ink_storage::traits::StorageLayout;
    ///
    /// #[derive(StorageLayout)]
    /// struct NamedFields {
    ///     a: u32,
    ///     b: [u32; 32],
    /// }
    ///
    /// let key = 0x123;
    /// let mut value = NamedFields {
    ///     a: 123,
    ///     b: [22; 32],
    /// };
    ///
    /// if let Struct(layout) = <NamedFields as StorageLayout>::layout(&key) {
    ///     assert_eq!(*layout.fields()[0].name(), "a");
    ///     assert_eq!(*layout.fields()[1].name(), "b");
    /// }
    /// ```
    storage_layout_derive
);
