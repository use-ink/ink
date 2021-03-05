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

use crate::{
    error::ExtError as _,
    ir,
    ir::utils,
};
use core::convert::TryFrom;
use proc_macro2::{
    Ident,
    Span,
};
use syn::spanned::Spanned as _;

/// An ink! event struct definition.
///
/// # Example
///
/// ```
/// # use core::convert::TryFrom;
/// # let event = <ink_lang_ir::Event as TryFrom<syn::ItemStruct>>::try_from(syn::parse_quote! {
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
        if !item_struct.generics.params.is_empty() {
            return Err(format_err_spanned!(
                item_struct.generics.params,
                "generic ink! event structs are not supported",
            ))
        }
        utils::ensure_pub_visibility("event structs", struct_span, &item_struct.vis)?;
        'repeat: for field in item_struct.fields.iter() {
            let field_span = field.span();
            let (ink_attrs, _) = ir::partition_attributes(field.attrs.clone())?;
            if ink_attrs.is_empty() {
                continue 'repeat
            }
            let normalized =
                ir::InkAttribute::from_expanded(ink_attrs).map_err(|err| {
                    err.into_combine(format_err!(field_span, "at this invocation",))
                })?;
            if !matches!(normalized.first().kind(), ir::AttributeArg::Topic) {
                return Err(format_err!(
                    field_span,
                    "first optional ink! attribute of an event field must be #[ink(topic)]",
                ))
            }
            for arg in normalized.args() {
                if !matches!(arg.kind(), ir::AttributeArg::Topic) {
                    return Err(format_err!(
                        arg.span(),
                        "encountered conflicting ink! attribute for event field",
                    ))
                }
            }
        }
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

    /// Returns an iterator yielding all the `#[ink(topic)]` annotated fields
    /// of the event struct.
    pub fn fields(&self) -> EventFieldsIter {
        EventFieldsIter::new(self)
    }

    /// Returns all non-ink! attributes.
    pub fn attrs(&self) -> &[syn::Attribute] {
        &self.item.attrs
    }
}

/// An event field with a flag indicating if this field is an event topic.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct EventField<'a> {
    /// The associated `field` is an event topic if this is `true`.
    pub is_topic: bool,
    /// The event field.
    field: &'a syn::Field,
}

impl<'a> EventField<'a> {
    /// Returns the span of the event field.
    pub fn span(self) -> Span {
        self.field.span()
    }

    /// Returns all non-ink! attributes of the event field.
    pub fn attrs(self) -> Vec<syn::Attribute> {
        let (_, non_ink_attrs) = ir::partition_attributes(self.field.attrs.clone())
            .expect("encountered invalid event field attributes");
        non_ink_attrs
    }

    /// Returns the visibility of the event field.
    pub fn vis(self) -> &'a syn::Visibility {
        &self.field.vis
    }

    /// Returns the identifier of the event field if any.
    pub fn ident(self) -> Option<&'a Ident> {
        self.field.ident.as_ref()
    }

    /// Returns the type of the event field.
    pub fn ty(self) -> &'a syn::Type {
        &self.field.ty
    }
}

/// Iterator yielding all `#[ink(topic)]` annotated fields of an event struct.
pub struct EventFieldsIter<'a> {
    iter: syn::punctuated::Iter<'a, syn::Field>,
}

impl<'a> EventFieldsIter<'a> {
    /// Creates a new topics fields iterator for the given ink! event struct.
    fn new(event: &'a Event) -> Self {
        Self {
            iter: event.item.fields.iter(),
        }
    }
}

impl<'a> Iterator for EventFieldsIter<'a> {
    type Item = EventField<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next() {
            None => None,
            Some(field) => {
                let is_topic = ir::first_ink_attribute(&field.attrs)
                    .unwrap_or_default()
                    .map(|attr| matches!(attr.first().kind(), ir::AttributeArg::Topic))
                    .unwrap_or_default();
                Some(EventField { is_topic, field })
            }
        }
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
    fn missing_storage_attribute_fails() {
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

    #[test]
    fn generic_event_fails() {
        assert_try_from_fails(
            syn::parse_quote! {
                #[ink(event)]
                pub struct GenericEvent<T> {
                    #[ink(topic)]
                    field_1: T,
                    field_2: bool,
                }
            },
            "generic ink! event structs are not supported",
        )
    }

    #[test]
    fn non_pub_event_struct() {
        assert_try_from_fails(
            syn::parse_quote! {
                #[ink(event)]
                struct PrivateEvent {
                    #[ink(topic)]
                    field_1: i32,
                    field_2: bool,
                }
            },
            "non `pub` ink! event structs are not supported",
        )
    }

    #[test]
    fn duplicate_field_attributes_fails() {
        assert_try_from_fails(
            syn::parse_quote! {
                #[ink(event)]
                pub struct MyEvent {
                    #[ink(topic)]
                    #[ink(topic)]
                    field_1: i32,
                    field_2: bool,
                }
            },
            "encountered duplicate ink! attribute",
        )
    }

    #[test]
    fn invalid_field_attributes_fails() {
        assert_try_from_fails(
            syn::parse_quote! {
                #[ink(event)]
                pub struct MyEvent {
                    #[ink(message)]
                    field_1: i32,
                    field_2: bool,
                }
            },
            "first optional ink! attribute of an event field must be #[ink(topic)]",
        )
    }

    #[test]
    fn conflicting_field_attributes_fails() {
        assert_try_from_fails(
            syn::parse_quote! {
                #[ink(event)]
                pub struct MyEvent {
                    #[ink(topic)]
                    #[ink(payable)]
                    field_1: i32,
                    field_2: bool,
                }
            },
            "encountered conflicting ink! attribute for event field",
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
    fn event_fields_iter_works() {
        let expected_fields: Vec<(bool, NamedField)> = vec![
            (
                true,
                syn::parse_quote! {
                    field_1: i32
                },
            ),
            (
                false,
                syn::parse_quote! {
                    field_2: u64
                },
            ),
            (
                true,
                syn::parse_quote! {
                    field_3: [u8; 32]
                },
            ),
        ];
        let input = <Event as TryFrom<syn::ItemStruct>>::try_from(syn::parse_quote! {
            #[ink(event)]
            pub struct MyEvent {
                #[ink(topic)]
                field_1: i32,
                field_2: u64,
                #[ink(topic)]
                field_3: [u8; 32],
            }
        })
        .unwrap();
        let mut fields_iter = input.fields();
        for (is_topic, expected_field) in expected_fields {
            let field = fields_iter.next().unwrap();
            assert_eq!(field.is_topic, is_topic);
            assert_eq!(field.ident(), Some(expected_field.ident()));
            assert_eq!(field.ty(), expected_field.ty());
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
