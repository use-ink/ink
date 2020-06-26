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

use super::{
    Constructor,
    Message,
};
use crate::{
    error::ExtError as _,
    ir2,
    ir2::attrs::Attrs as _,
};
use core::convert::TryFrom;
use syn::spanned::Spanned as _;

/// The visibility of an ink! message or constructor.
#[derive(Debug, Clone)]
pub enum Visibility {
    Public(syn::VisPublic),
    Inherited,
}

impl Visibility {
    /// Returns `true` if the visibility of the ink! message of constructor is public (`pub`).
    ///
    /// # Note
    ///
    /// Messages in normal implementation blocks must have public visibility.
    pub fn is_pub(&self) -> bool {
        matches!(self, Self::Public(_))
    }

    /// Returns `true` if the visibility of the ink! message of constructor is inherited.
    ///
    /// # Note
    ///
    /// Messages in trait implementation blocks must have inherited visibility.
    pub fn is_inherited(&self) -> bool {
        matches!(self, Self::Inherited)
    }
}

/// An item within an ink! implementation block.
#[derive(Debug, PartialEq, Eq)]
#[allow(clippy::large_enum_variant)]
pub enum ImplBlockItem {
    /// A `#[ink(constructor)]` marked inherent function.
    Constructor(Constructor),
    /// A `#[ink(message)]` marked method.
    Message(Message),
    /// Any other implementation block item.
    Other(syn::ImplItem),
}

impl TryFrom<syn::ImplItem> for ImplBlockItem {
    type Error = syn::Error;

    fn try_from(impl_item: syn::ImplItem) -> Result<Self, Self::Error> {
        match impl_item {
            syn::ImplItem::Method(method_item) => {
                if !ir2::contains_ink_attributes(&method_item.attrs) {
                    return Ok(Self::Other(method_item.into()))
                }
                let attr = ir2::first_ink_attribute(&method_item.attrs)?
                    .expect("missing expected ink! attribute for struct");
                match attr.first().kind() {
                    ir2::AttributeArgKind::Message => {
                        <Message as TryFrom<_>>::try_from(method_item)
                            .map(Into::into)
                            .map(Self::Message)
                    }
                    ir2::AttributeArgKind::Constructor => {
                        <Constructor as TryFrom<_>>::try_from(method_item)
                            .map(Into::into)
                            .map(Self::Constructor)
                    }
                    _ => Err(format_err!(
                        method_item,
                        "encountered invalid ink! attribute at this point, expected either \
                        #[ink(message)] or #[ink(constructor) attributes"
                    )),
                }
            }
            other_item => {
                // This is an error if the impl item contains any unexpected
                // ink! attributes. Otherwise it is a normal Rust item.
                if ir2::contains_ink_attributes(other_item.attrs()) {
                    let (ink_attrs, _) =
                        ir2::partition_attributes(other_item.attrs().iter().cloned())?;
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
                Ok(Self::Other(other_item))
            }
        }
    }
}

impl ImplBlockItem {
    /// Returns `true` if the impl block item is an ink! message.
    pub fn is_message(&self) -> bool {
        self.filter_map_message().is_some()
    }

    /// Returns `Some` if `self` is an ink! message.
    ///
    /// Otherwise, returns `None`.
    pub fn filter_map_message(&self) -> Option<&Message> {
        match self {
            ImplBlockItem::Message(message) => Some(message),
            _ => None,
        }
    }

    /// Returns `true` if the impl block item is an ink! message.
    pub fn is_constructor(&self) -> bool {
        self.filter_map_constructor().is_some()
    }

    /// Returns `Some` if `self` is an ink! constructor.
    ///
    /// Otherwise, returns `None`.
    pub fn filter_map_constructor(&self) -> Option<&Constructor> {
        match self {
            ImplBlockItem::Constructor(constructor) => Some(constructor),
            _ => None,
        }
    }

    /// Returns `true` if the impl block item is a non ink! specific item.
    pub fn is_other_item(&self) -> bool {
        self.filter_map_other_item().is_some()
    }

    /// Returns `Some` if `self` is a not an ink! specific item.
    ///
    /// Otherwise, returns `None`.
    pub fn filter_map_other_item(&self) -> Option<&syn::ImplItem> {
        match self {
            ImplBlockItem::Other(rust_item) => Some(rust_item),
            _ => None,
        }
    }
}
