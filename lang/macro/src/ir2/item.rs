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

use crate::{
    error::ExtError as _,
    ir2,
    ir2::attrs::Attrs as _,
};
use core::convert::TryFrom;
use syn::spanned::Spanned as _;

/// An item in the root of the ink! module.
///
/// This is either an ink! specific item or a normal Rust item.
pub enum Item {
    /// The item is an ink! specific item.
    Ink(InkItem),
    /// The item is a normal Rust item.
    Rust(syn::Item),
}

impl TryFrom<syn::Item> for Item {
    type Error = syn::Error;

    fn try_from(item: syn::Item) -> Result<Self, Self::Error> {
        match item {
            syn::Item::Struct(item_struct) => {
                if !ir2::contains_ink_attributes(&item_struct.attrs) {
                    return Ok(Self::Rust(item_struct.into()))
                }
                // At this point we know that there must be at least one ink!
                // attribute. This can be either the ink! storage struct,
                // an ink! event or an invalid ink! attribute.
                let attr = ir2::first_ink_attribute(&item_struct.attrs)?
                    .expect("missing expected ink! attribute for struct");
                match attr.first().kind() {
                    ir2::AttributeArgKind::Storage => {
                        <ir2::Storage as TryFrom<_>>::try_from(item_struct)
                            .map(Into::into)
                            .map(Self::Ink)
                    }
                    ir2::AttributeArgKind::Event => {
                        <ir2::Event as TryFrom<_>>::try_from(item_struct)
                            .map(Into::into)
                            .map(Self::Ink)
                    }
                    _invalid => {
                        Err(format_err_span!(
                            attr.span(),
                            "encountered unsupported ink! attribute argument on struct",
                        ))
                    }
                }
            }
            syn::Item::Impl(item_impl) => {
                if !ir2::contains_ink_attributes(&item_impl.attrs)
                    && item_impl
                        .items
                        .iter()
                        .all(|item| !ir2::contains_ink_attributes(item.attrs()))
                {
                    return Ok(Self::Rust(item_impl.into()))
                }
                // At this point we know that there must be at least one ink!
                // attribute on either the impl block itself or one of its items.
                <ir2::ImplBlock as TryFrom<_>>::try_from(item_impl)
                    .map(Into::into)
                    .map(Self::Ink)
            }
            item => {
                // This is an error if the item contains any unexpected
                // ink! attributes. Otherwise it is a normal Rust item.
                if ir2::contains_ink_attributes(item.attrs()) {
                    let (ink_attrs, _) =
                        ir2::partition_attributes(item.attrs().iter().cloned())?;
                    assert!(!ink_attrs.is_empty());
                    fn into_err(attr: &ir2::InkAttribute) -> syn::Error {
                        format_err_span!(
                            attr.span(),
                            "encountered unexpected ink! attribute",
                        )
                    }
                    return Err(ink_attrs[1..]
                        .iter()
                        .map(into_err)
                        .fold(into_err(&ink_attrs[0]), |fst, snd| fst.into_combine(snd)))
                }
                Ok(Self::Rust(item))
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
