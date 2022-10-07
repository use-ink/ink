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

use crate::ir;
use proc_macro2::{
    Ident,
    Span,
    TokenStream as TokenStream2,
};
use syn::{
    spanned::Spanned as _,
    Result,
};

/// An ink! event enum definition.
#[derive(Debug, PartialEq, Eq)]
pub struct InkEventDefinition {
    pub item: syn::ItemEnum,
    variants: Vec<EventVariant>,
}

impl TryFrom<syn::ItemEnum> for InkEventDefinition {
    type Error = syn::Error;

    fn try_from(item_enum: syn::ItemEnum) -> Result<Self> {
        let mut variants = Vec::new();
        for (index, variant) in item_enum.variants.iter().enumerate() {
            let mut fields = Vec::new();
            let (ink_attrs, other_attrs) = ir::sanitize_optional_attributes(
                variant.span(),
                variant.attrs.clone(),
                |arg| {
                    match arg.kind() {
                        ir::AttributeArg::Anonymous => Ok(()),
                        _ => Err(None),
                    }
                },
            )?;
            // strip out the `#[ink(anonymous)] attributes, since the item will be used to
            // regenerate the event enum
            let variant = syn::Variant {
                attrs: other_attrs,
                ..variant.clone()
            };
            let anonymous = ink_attrs.map_or(false, |attrs| attrs.is_anonymous());
            for field in variant.fields.iter() {
                let (topic_attr, other_attrs) = ir::sanitize_optional_attributes(
                    field.span(),
                    field.attrs.clone(),
                    |arg| {
                        match arg.kind() {
                            ir::AttributeArg::Topic => Ok(()),
                            _ => Err(None),
                        }
                    },
                )?;
                let ident = field.ident.as_ref().ok_or_else(|| {
                    format_err_spanned!(variant, "event variants must have named fields")
                })?;
                // strip out the `#[ink(topic)] attributes, since the item will be used to
                // regenerate the event enum
                let field = syn::Field {
                    attrs: other_attrs,
                    ..field.clone()
                };
                fields.push(EventField {
                    is_topic: topic_attr.is_some(),
                    field,
                    ident: ident.clone(),
                })
            }
            let named_fields = matches!(variant.fields, syn::Fields::Named(_));
            variants.push(EventVariant {
                index,
                item: variant.clone(),
                named_fields,
                fields,
                anonymous,
            })
        }
        Ok(Self {
            item: item_enum,
            variants,
        })
    }
}

impl quote::ToTokens for InkEventDefinition {
    /// We mainly implement this trait for this ink! type to have a derived
    /// [`Spanned`](`syn::spanned::Spanned`) implementation for it.
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.item.to_tokens(tokens)
    }
}

impl InkEventDefinition {
    /// Create an [`InkEventDefinition`] for a event defined externally to a contract.
    ///
    /// This will be an enum annotated with the `#[ink::event_def]` attribute.
    pub fn from_event_def_tokens(
        config: TokenStream2,
        input: TokenStream2,
    ) -> Result<Self> {
        let _parsed_config = syn::parse2::<crate::ast::AttributeArgs>(config)?;
        let item = syn::parse2::<syn::ItemEnum>(input)?;
        // let item = InkItemTrait::new(&config, parsed_item)?;
        Self::try_from(item)
    }

    /// Returns the identifier of the event struct.
    pub fn ident(&self) -> &Ident {
        &self.item.ident
    }

    pub fn span(&self) -> Span {
        self.item.span()
    }

    /// Returns all non-ink! attributes.
    pub fn attrs(&self) -> &[syn::Attribute] {
        &self.item.attrs
    }

    /// Returns all event variants.
    pub fn variants(&self) -> impl Iterator<Item = &EventVariant> {
        self.variants.iter()
    }

    /// Returns the maximum number of topics of any event variant.
    pub fn max_len_topics(&self) -> usize {
        self.variants()
            .map(|v| {
                let topics_len = v.fields().filter(|event| event.is_topic).count();
                if v.anonymous {
                    topics_len
                } else {
                    topics_len + 1usize
                }
            })
            .max()
            .unwrap_or_default()
    }
}

/// A variant of an event.
#[derive(Debug, PartialEq, Eq)]
pub struct EventVariant {
    index: usize,
    item: syn::Variant,
    named_fields: bool,
    fields: Vec<EventField>,
    anonymous: bool,
}

impl EventVariant {
    /// Returns the span of the event variant.
    pub fn span(&self) -> Span {
        self.item.span()
    }

    /// The identifier of the event variant.
    pub fn ident(&self) -> &Ident {
        &self.item.ident
    }

    /// The index of the the event variant in the enum definition.
    pub fn index(&self) -> usize {
        self.index
    }

    /// Returns an iterator yielding all the `#[ink(topic)]` annotated fields
    /// of the event variant struct.
    pub fn fields(&self) -> impl Iterator<Item = &EventField> {
        self.fields.iter()
    }

    /// Returns true if the signature of the event variant should *not* be indexed by a topic.
    pub fn anonymous(&self) -> bool {
        self.anonymous
    }
}

/// An event field with a flag indicating if this field is an event topic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventField {
    /// The associated `field` is an event topic if this is `true`.
    pub is_topic: bool,
    /// The event field.
    field: syn::Field,
    /// The event field ident.
    ident: syn::Ident,
}

impl EventField {
    /// Returns the span of the event field.
    pub fn span(&self) -> Span {
        self.field.span()
    }

    /// Returns all non-ink! attributes of the event field.
    pub fn attrs(&self) -> Vec<syn::Attribute> {
        let (_, non_ink_attrs) = ir::partition_attributes(self.field.attrs.clone())
            .expect("encountered invalid event field attributes");
        non_ink_attrs
    }

    /// Returns the visibility of the event field.
    pub fn vis(&self) -> &syn::Visibility {
        &self.field.vis
    }

    /// Returns the identifier of the event field if any.
    pub fn ident(&self) -> &Ident {
        &self.ident
    }

    /// Returns the type of the event field.
    pub fn ty(&self) -> &syn::Type {
        &self.field.ty
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_try_from_works() {
        let item_struct: syn::ItemEnum = syn::parse_quote! {
            #[ink(event)]
            pub enum MyEvent {
                Event {
                    #[ink(topic)]
                    field_1: i32,
                    field_2: bool,
                }
            }
        };
        assert!(InkEventDefinition::try_from(item_struct).is_ok());
    }

    fn assert_try_from_fails(item_struct: syn::ItemEnum, expected: &str) {
        assert_eq!(
            InkEventDefinition::try_from(item_struct).map_err(|err| err.to_string()),
            Err(expected.to_string())
        )
    }

    #[test]
    fn conflicting_struct_attributes_fails() {
        assert_try_from_fails(
            syn::parse_quote! {
                #[ink(event)]
                #[ink(storage)]
                pub enum MyEvent {
                    Event {
                        #[ink(topic)]
                        field_1: i32,
                        field_2: bool,
                    }
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
                pub enum MyEvent {
                    Event {
                        #[ink(topic)]
                        field_1: i32,
                        field_2: bool,
                    }
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
                pub enum MyEvent {
                    Event {
                        #[ink(topic)]
                        field_1: i32,
                        field_2: bool,
                    }
                }
            },
            "unexpected first ink! attribute argument",
        )
    }

    #[test]
    fn missing_event_attribute_fails() {
        assert_try_from_fails(
            syn::parse_quote! {
                pub enum MyEvent {
                    Event {
                        #[ink(topic)]
                        field_1: i32,
                        field_2: bool,
                    }
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
                pub enum GenericEvent<T> {
                    Event {
                        #[ink(topic)]
                        field_1: T,
                        field_2: bool,
                    }
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
                enum PrivateEvent {
                    Event {
                        #[ink(topic)]
                        field_1: i32,
                        field_2: bool,
                    }
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
                pub enum MyEvent {
                    Event {
                        #[ink(topic)]
                        #[ink(topic)]
                        field_1: i32,
                        field_2: bool,
                    }
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
                pub enum MyEvent {
                    Event {
                        #[ink(message)]
                        field_1: i32,
                        field_2: bool,
                    }
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
                pub enum MyEvent {
                    Event {
                        #[ink(topic)]
                        #[ink(payable)]
                        field_1: i32,
                        field_2: bool,
                    }
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
        let event_def =
            <InkEventDefinition as TryFrom<syn::ItemEnum>>::try_from(syn::parse_quote! {
                #[ink(event)]
                pub enum MyEvent {
                    Event {
                        #[ink(topic)]
                        field_1: i32,
                        field_2: u64,
                        #[ink(topic)]
                        field_3: [u8; 32],
                    }
                }
            })
            .unwrap();
        let event_variant = event_def.variants().next().expect("Event variant");
        let mut fields_iter = event_variant.fields();
        for (is_topic, expected_field) in expected_fields {
            let field = fields_iter.next().unwrap();
            assert_eq!(field.is_topic, is_topic);
            assert_eq!(field.ident(), expected_field.ident());
            assert_eq!(field.ty(), expected_field.ty());
        }
    }

    #[test]
    fn anonymous_event_works() {
        fn assert_anonymous_event(event: syn::ItemEnum) {
            match InkEventDefinition::try_from(event) {
                Ok(event) => {
                    assert!(event.variants[0].anonymous);
                }
                Err(_) => panic!("encountered unexpected invalid anonymous event"),
            }
        }
        assert_anonymous_event(syn::parse_quote! {
            #[ink(event)]
            pub enum MyEvent {
                #[ink(anonymous)]
                Event {
                    #[ink(topic)]
                    field_1: i32,
                    field_2: bool,
                }
            }
        });
    }
}
