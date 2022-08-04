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
    ir::utils,
};
use proc_macro2::{
    Ident,
    Span,
    TokenStream as TokenStream2,
};
use syn::{
    spanned::Spanned as _,
    Result,
};

/// A checked ink! event definition.
#[derive(Debug, PartialEq, Eq)]
pub struct InkEventDefinition {
    pub item: syn::ItemStruct,
    pub anonymous: bool,
}

impl TryFrom<syn::ItemStruct> for InkEventDefinition {
    type Error = syn::Error;

    fn try_from(item_struct: syn::ItemStruct) -> Result<Self> {
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
        let item_struct = syn::ItemStruct {
            attrs: other_attrs,
            ..item_struct
        };
        Self::new(item_struct, ink_attrs.is_anonymous())
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
    /// Returns `Ok` if the input matches all requirements for an ink! event definition.
    pub fn new(item_struct: syn::ItemStruct, anonymous: bool) -> Result<Self> {
        if !item_struct.generics.params.is_empty() {
            return Err(format_err_spanned!(
                item_struct.generics.params,
                "generic ink! event structs are not supported",
            ))
        }
        let struct_span = item_struct.span();
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
            item: item_struct,
            anonymous,
        })
    }

    /// Returns `Ok` if the input matches all requirements for an ink! event definition.
    pub fn from_event_def_tokens(
        config: TokenStream2,
        input: TokenStream2,
    ) -> Result<Self> {
        let _parsed_config = syn::parse2::<crate::ast::AttributeArgs>(config)?;
        let anonymous = false; // todo parse this from attr config
        let item = syn::parse2::<syn::ItemStruct>(input)?;
        // let item = InkItemTrait::new(&config, parsed_item)?;
        Ok(Self { anonymous, item })
    }

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
    fn new(event: &'a InkEventDefinition) -> Self {
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
