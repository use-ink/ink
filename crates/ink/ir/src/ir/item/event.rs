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

use crate::ir::{
    self,
    utils::extract_cfg_attributes,
};
use proc_macro2::{
    Ident,
    Span,
    TokenStream,
};
use syn::spanned::Spanned as _;

/// An ink! event struct definition.
///
/// # Example
///
/// ```
/// # let event = <ink_ir::Event as TryFrom<syn::ItemStruct>>::try_from(syn::parse_quote! {
/// #[ink(event)]
/// pub struct Transaction {
///     #[ink(topic)]
///     from: AccountId,
///     #[ink(topic)]
///     to: AccountId,
///     value: Balance,
/// }
/// # }).unwrap();
/// ```
#[derive(Debug, PartialEq, Eq)]
pub struct Event {
    item: syn::ItemStruct,
    pub anonymous: bool,
}

impl quote::ToTokens for Event {
    /// We mainly implement this trait for this ink! type to have a derived
    /// [`Spanned`](`syn::spanned::Spanned`) implementation for it.
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.item.to_tokens(tokens)
    }
}

impl Event {
    /// Returns `true` if the first ink! annotation on the given struct is
    /// `#[ink(event)]`.
    ///
    /// # Errors
    ///
    /// If the first found ink! attribute is malformed.
    pub(super) fn is_ink_event(
        item_struct: &syn::ItemStruct,
    ) -> Result<bool, syn::Error> {
        if !ir::contains_ink_attributes(&item_struct.attrs) {
            return Ok(false)
        }
        // At this point we know that there must be at least one ink!
        // attribute. This can be either the ink! storage struct,
        // an ink! event or an invalid ink! attribute.
        let attr = ir::first_ink_attribute(&item_struct.attrs)?
            .expect("missing expected ink! attribute for struct");
        Ok(matches!(attr.first().kind(), ir::AttributeArg::Event))
    }
}

impl TryFrom<syn::ItemStruct> for Event {
    type Error = syn::Error;

    fn try_from(item_struct: syn::ItemStruct) -> Result<Self, Self::Error> {
        let struct_span = item_struct.span();
        let (ink_attrs, other_attrs) = ir::sanitize_attributes(
            struct_span,
            item_struct.attrs,
            &ir::AttributeArgKind::Event,
            |arg| {
                match arg.kind() {
                    ir::AttributeArg::Event | ir::AttributeArg::Anonymous => Ok(()),
                    _ => Err(None),
                }
            },
        )?;
        Ok(Self {
            item: syn::ItemStruct {
                attrs: other_attrs,
                ..item_struct
            },
            anonymous: ink_attrs.is_anonymous(),
        })
    }
}

impl Event {
    /// Returns the identifier of the event struct.
    pub fn ident(&self) -> &Ident {
        &self.item.ident
    }

    /// Returns all non-ink! attributes.
    pub fn attrs(&self) -> &[syn::Attribute] {
        &self.item.attrs
    }

    /// Returns a list of `cfg` attributes if any.
    pub fn get_cfg_attrs(&self, span: Span) -> Vec<TokenStream> {
        extract_cfg_attributes(self.attrs(), span)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_try_from_works() {
        let item_struct: syn::ItemStruct = syn::parse_quote! {
            #[ink(event)]
            pub struct MyEvent {
                #[ink(topic)]
                field_1: i32,
                field_2: bool,
            }
        };
        assert!(Event::try_from(item_struct).is_ok());
    }

    fn assert_try_from_fails(item_struct: syn::ItemStruct, expected: &str) {
        assert_eq!(
            Event::try_from(item_struct).map_err(|err| err.to_string()),
            Err(expected.to_string())
        )
    }

    #[test]
    fn conflicting_struct_attributes_fails() {
        assert_try_from_fails(
            syn::parse_quote! {
                #[ink(event)]
                #[ink(storage)]
                pub struct MyEvent {
                    #[ink(topic)]
                    field_1: i32,
                    field_2: bool,
                }
            },
            "encountered conflicting ink! attribute argument",
        )
    }

    #[test]
    fn duplicate_struct_attributes_fails() {
        assert_try_from_fails(
            syn::parse_quote! {
                #[ink(event)]
                #[ink(event)]
                pub struct MyEvent {
                    #[ink(topic)]
                    field_1: i32,
                    field_2: bool,
                }
            },
            "encountered duplicate ink! attribute",
        )
    }

    #[test]
    fn wrong_first_struct_attribute_fails() {
        assert_try_from_fails(
            syn::parse_quote! {
                #[ink(storage)]
                #[ink(event)]
                pub struct MyEvent {
                    #[ink(topic)]
                    field_1: i32,
                    field_2: bool,
                }
            },
            "unexpected first ink! attribute argument",
        )
    }

    #[test]
    fn missing_event_attribute_fails() {
        assert_try_from_fails(
            syn::parse_quote! {
                pub struct MyEvent {
                    #[ink(topic)]
                    field_1: i32,
                    field_2: bool,
                }
            },
            "encountered unexpected empty expanded ink! attribute arguments",
        )
    }

    /// Used for the event fields iterator unit test because `syn::Field` does
    /// not provide a `syn::parse::Parse` implementation.
    #[derive(Debug, PartialEq, Eq)]
    struct NamedField(syn::Field);

    impl syn::parse::Parse for NamedField {
        fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
            Ok(Self(syn::Field::parse_named(input)?))
        }
    }

    impl NamedField {
        /// Returns the identifier of the named field.
        pub fn ident(&self) -> &Ident {
            self.0.ident.as_ref().unwrap()
        }

        /// Returns the type of the named field.
        pub fn ty(&self) -> &syn::Type {
            &self.0.ty
        }
    }

    #[test]
    fn anonymous_event_works() {
        fn assert_anonymous_event(event: syn::ItemStruct) {
            match Event::try_from(event) {
                Ok(event) => {
                    assert!(event.anonymous);
                }
                Err(_) => panic!("encountered unexpected invalid anonymous event"),
            }
        }
        assert_anonymous_event(syn::parse_quote! {
            #[ink(event)]
            #[ink(anonymous)]
            pub struct MyEvent {
                #[ink(topic)]
                field_1: i32,
                field_2: bool,
            }
        });
        assert_anonymous_event(syn::parse_quote! {
            #[ink(event, anonymous)]
            pub struct MyEvent {
                #[ink(topic)]
                field_1: i32,
                field_2: bool,
            }
        });
    }
}
