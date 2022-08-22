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

mod storable_hint;
mod storage_key;
mod storage_layout;

#[cfg(test)]
mod tests;

use self::{
    storable_hint::storable_hint_derive,
    storage_key::storage_key_derive,
    storage_layout::storage_layout_derive,
};
synstructure::decl_derive!(
    [StorableHint] =>
    /// Derives `ink_storage`'s [`StorableHint`](ink_storage::traits::StorableHint) trait for the given `struct`
    /// or `enum`.
    ///
    /// If the type declaration contains generic [`StorageKey`](ink_storage::traits::StorageKey),
    /// it will use it as salt to generate a combined storage key.
    ///
    /// # Examples
    ///
    /// ```
    /// use ink_storage::traits::{
    ///     StorableHint,
    ///     StorageKey,
    ///     AutoStorableHint,
    ///     AutoKey,
    ///     ManualKey,
    /// };
    /// use ink_primitives::traits::Storable;
    ///
    /// #[derive(Default, StorableHint, Storable)]
    /// struct NamedFields {
    ///     a: u32,
    ///     b: [u32; 32],
    /// }
    ///
    /// let _: NamedFields = <NamedFields as StorableHint<AutoKey>>::Type::default();
    /// let _: NamedFields = <NamedFields as StorableHint<ManualKey<123>>>::Type::default();
    /// ```
    storable_hint_derive
);
synstructure::decl_derive!(
    [StorageKey] =>
    /// Derives `ink_storage`'s [`StorageKey`](ink_storage::traits::StorageKey) trait for the given
    /// `struct` or `enum`.
    ///
    /// # Examples
    ///
    /// ```
    /// use ink_storage::traits::{
    ///     AutoStorableHint,
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
    ///     a: <u32 as AutoStorableHint<ManualKey<0, KEY>>>::Type,
    ///     b: <[u32; 32] as AutoStorableHint<ManualKey<1, KEY>>>::Type,
    /// }
    ///
    /// assert_eq!(<NamedFieldsManualKey<()> as StorageKey>::KEY, 0);
    /// assert_eq!(<NamedFieldsManualKey<AutoKey> as StorageKey>::KEY, 0);
    /// assert_eq!(<NamedFieldsManualKey<ManualKey<123>> as StorageKey>::KEY, 123);
    /// ```
    storage_key_derive
);
synstructure::decl_derive!(
    [StorageLayout] =>
    /// Derives `ink_storage`'s [`StorageLayout`](ink_storage::traits::StorageLayout) trait for the
    /// given `struct` or `enum`.
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
