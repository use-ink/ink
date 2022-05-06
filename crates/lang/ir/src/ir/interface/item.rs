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

use crate::{
    error::ExtError as _,
    ir,
    ir::attrs::Attrs as _,
};
use syn::spanned::Spanned as _;

/// An item in the root of the ink! module ([`ir::ItemMod`](`crate::ir::ItemMod`)).
///
/// This is either an ink! specific item or a normal Rust item.
#[derive(Debug, PartialEq, Eq)]
pub enum Item {
    /// The item is an ink! specific item.
    Ink(InterfaceItem),
    /// The item is a normal Rust item.
    Rust(syn::Item),
}

impl quote::ToTokens for Item {
    /// We mainly implement this trait for this ink! type to have a derived
    /// [`Spanned`](`syn::spanned::Spanned`) implementation for it.
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Self::Ink(ink_item) => ink_item.to_tokens(tokens),
            Self::Rust(rust_item) => rust_item.to_tokens(tokens),
        }
    }
}

impl TryFrom<syn::Item> for Item {
    type Error = syn::Error;

    fn try_from(item: syn::Item) -> Result<Self, Self::Error> {
        match item {
            syn::Item::Struct(item_struct) => {
                if !ir::contains_ink_attributes(&item_struct.attrs) {
                    return Ok(Self::Rust(item_struct.into()))
                }
                // At this point we know that there must be at least one ink!
                // attribute. This can be either the ink! storage struct,
                // an ink! event or an invalid ink! attribute.
                let attr = ir::first_ink_attribute(&item_struct.attrs)?
                    .expect("missing expected ink! attribute for struct");
                match attr.first().kind() {
                    ir::AttributeArg::Storage => {
                        <ir::Storage as TryFrom<_>>::try_from(item_struct)
                            .map(Into::into)
                            .map(Self::Ink)
                    }
                    ir::AttributeArg::Event => {
                        <ir::Event as TryFrom<_>>::try_from(item_struct)
                            .map(Into::into)
                            .map(Self::Ink)
                    }
                    _invalid => {
                        Err(format_err!(
                            attr.span(),
                            "encountered unsupported ink! attribute argument on struct",
                        ))
                    }
                }
            }
            syn::Item::Impl(item_impl) => {
                if !ir::ItemImpl::is_ink_impl_block(&item_impl)? {
                    return Ok(Self::Rust(item_impl.into()))
                }
                // At this point we know that there must be at least one ink!
                // attribute on either the `impl` block itself or one of its items.
                <ir::ItemImpl as TryFrom<_>>::try_from(item_impl)
                    .map(Into::into)
                    .map(Self::Ink)
            }
            syn::Item::Type(item_type) => {
                if !ir::Event::is_ink_event(&item_type.attrs)? {
                    return Ok(Self::Rust(item_type.into()))
                }
                <ir::Event as TryFrom<_>>::try_from(item_type)
                    .map(Into::into)
                    .map(Self::Ink)
            }
            item => {
                // This is an error if the item contains any unexpected
                // ink! attributes. Otherwise it is a normal Rust item.
                if ir::contains_ink_attributes(item.attrs()) {
                    let (ink_attrs, _) =
                        ir::partition_attributes(item.attrs().iter().cloned())?;
                    assert!(!ink_attrs.is_empty());
                    fn into_err(attr: &ir::InkAttribute) -> syn::Error {
                        format_err!(attr.span(), "encountered unexpected ink! attribute",)
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
