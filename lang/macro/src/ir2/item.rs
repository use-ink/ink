// Copyright 2018-2020 Parity Technologies (UK) Ltd.
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

use crate::ir2;
use core::convert::TryFrom;

/// An item in the root of the ink! module.
///
/// This is either an ink! specific item or a normal Rust item.
pub enum Item {
    /// The item is an ink! specific item.
    Ink(InkItem),
    /// The item is a normal Rust item.
    Rust(syn::Item),
}

/// Returns a slice to the attributes of the given [`syn::Item`].
fn item_attrs(item: &syn::Item) -> &[syn::Attribute] {
    use syn::Item;
    match item {
        Item::Const(syn::ItemConst { attrs, .. }) => attrs,
        Item::Enum(syn::ItemEnum { attrs, .. }) => attrs.as_slice(),
        Item::ExternCrate(syn::ItemExternCrate { attrs, .. }) => attrs.as_slice(),
        Item::Fn(syn::ItemFn { attrs, .. }) => attrs.as_slice(),
        Item::ForeignMod(syn::ItemForeignMod { attrs, .. }) => attrs.as_slice(),
        Item::Impl(syn::ItemImpl { attrs, .. }) => attrs.as_slice(),
        Item::Macro(syn::ItemMacro { attrs, .. }) => attrs.as_slice(),
        Item::Macro2(syn::ItemMacro2 { attrs, .. }) => attrs.as_slice(),
        Item::Mod(syn::ItemMod { attrs, .. }) => attrs.as_slice(),
        Item::Static(syn::ItemStatic { attrs, .. }) => attrs.as_slice(),
        Item::Struct(syn::ItemStruct { attrs, .. }) => attrs.as_slice(),
        Item::Trait(syn::ItemTrait { attrs, .. }) => attrs.as_slice(),
        Item::TraitAlias(syn::ItemTraitAlias { attrs, .. }) => attrs.as_slice(),
        Item::Type(syn::ItemType { attrs, .. }) => attrs.as_slice(),
        Item::Union(syn::ItemUnion { attrs, .. }) => attrs.as_slice(),
        Item::Use(syn::ItemUse { attrs, .. }) => attrs.as_slice(),
        _ => &[],
    }
}

impl TryFrom<syn::Item> for Item {
    type Error = ir2::Error;

    fn try_from(item: syn::Item) -> Result<Self, Self::Error> {
        if !ir2::contains_ink_attributes(item_attrs(&item)) {
            return Ok(Self::Rust(item))
        }
        // At this point we know that there must be at least one ink! attribute.
        match item {
            syn::Item::Struct(item_struct) => {
                // This can be either the ink! storage struct or an ink! event.
                let (ink_attrs, other_attrs) =
                    ir2::partition_attributes(item_struct.attrs)?;
                todo!()
            }
            syn::Item::Impl(item_impl) => {
                <ir2::ImplBlock as TryFrom<_>>::try_from(item_impl)
                    .map(Into::into)
                    .map(Self::Ink)
            }
            invalid => {
                // Error since we do not expect to see an ink! attribute on any
                // other item kind.
                todo!()
            }
        }
    }
}

impl Item {
    /// Returns `true` if `self` is an ink! specific item.
    pub fn is_ink_item(&self) -> bool {
        self.map_ink_item().is_some()
    }

    /// Returns `true` if `self` is an normal Rust item.
    pub fn is_rust_item(&self) -> bool {
        self.map_rust_item().is_some()
    }

    /// Returns `Some` if `self` is an ink! specific item.
    ///
    /// Otherwise, returns `None`.
    pub fn map_ink_item(&self) -> Option<&InkItem> {
        match self {
            Item::Ink(ink_item) => Some(ink_item),
            _ => None,
        }
    }

    /// Returns `Some` if `self` is an ink! specific item.
    ///
    /// Otherwise, returns `None`.
    pub fn map_rust_item(&self) -> Option<&syn::Item> {
        match self {
            Item::Rust(rust_item) => Some(rust_item),
            _ => None,
        }
    }
}

/// An ink! specific item.
pub enum InkItem {
    /// The ink! storage struct definition.
    Storage(ir2::Storage),
    /// An ink! event definition.
    Event(ir2::Event),
    /// An ink! implementation block.
    ImplBlock(ir2::ImplBlock),
}

impl From<ir2::Storage> for InkItem {
    fn from(storage: ir2::Storage) -> Self {
        Self::Storage(storage)
    }
}

impl From<ir2::Event> for InkItem {
    fn from(event: ir2::Event) -> Self {
        Self::Event(event)
    }
}

impl From<ir2::ImplBlock> for InkItem {
    fn from(impl_block: ir2::ImplBlock) -> Self {
        Self::ImplBlock(impl_block)
    }
}

impl InkItem {
    /// Returns `Some` if `self` is the ink! storage struct definition.
    ///
    /// Otherwise, returns `None`.
    pub fn filter_map_storage_item(&self) -> Option<&ir2::Storage> {
        match self {
            InkItem::Storage(storage) => Some(storage),
            _ => None,
        }
    }

    /// Returns `true` if the ink! specific item is the storage struct definition.
    pub fn is_storage_item(&self) -> bool {
        self.filter_map_storage_item().is_some()
    }

    /// Returns `Some` if `self` is an ink! event struct definition.
    ///
    /// Otherwise, returns `None`.
    pub fn filter_map_event_item(&self) -> Option<&ir2::Event> {
        match self {
            InkItem::Event(event) => Some(event),
            _ => None,
        }
    }

    /// Returns `true` if the ink! specific item is an event struct definition.
    pub fn is_event_item(&self) -> bool {
        self.filter_map_event_item().is_some()
    }

    /// Returns `Some` if `self` is an ink! implementation block.
    ///
    /// Otherwise, returns `None`.
    pub fn filter_map_impl_block(&self) -> Option<&ir2::ImplBlock> {
        match self {
            InkItem::ImplBlock(impl_block) => Some(impl_block),
            _ => None,
        }
    }

    /// Returns `true` if the ink! specific item is an implementation block.
    pub fn is_impl_block(&self) -> bool {
        self.filter_map_impl_block().is_some()
    }
}
