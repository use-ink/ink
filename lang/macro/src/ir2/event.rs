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
};
use core::convert::TryFrom;
use proc_macro2::Ident;
use syn::spanned::Spanned as _;

pub struct Event {
    ast: syn::ItemStruct,
}

impl TryFrom<syn::ItemStruct> for Event {
    type Error = syn::Error;

    fn try_from(item_struct: syn::ItemStruct) -> Result<Self, Self::Error> {
        let struct_span = item_struct.span();
        let (ink_attrs, other_attrs) = ir2::partition_attributes(item_struct.attrs)?;
        let normalized = ir2::InkAttribute::from_expanded(ink_attrs).map_err(|err| {
            err.into_combine(format_err_span!(struct_span, "at this invokation",))
        })?;
        normalized
            .ensure_first(&ir2::AttributeArgKind::Event)
            .map_err(|err| {
                err.into_combine(format_err_span!(
                    struct_span,
                    "expected `#[ink(event)]` as first ink! attribute argument",
                ))
            })?;
        normalized
            .ensure_no_conflicts(|arg| arg.kind() != &ir2::AttributeArgKind::Event)?;
        if !item_struct.generics.params.is_empty() {
            return Err(format_err!(
                item_struct.generics.params,
                "generic ink! event structs are not supported",
            ))
        }
        match &item_struct.vis {
            syn::Visibility::Inherited
            | syn::Visibility::Restricted(_)
            | syn::Visibility::Crate(_) => {
                return Err(format_err!(
                    &item_struct.vis,
                    "non `pub` ink! event structs are not supported",
                ))
            }
            _ => (),
        }
        for field in item_struct.fields.iter() {
            let field_span = field.span();
            let (ink_attrs, _) = ir2::partition_attributes(field.attrs.clone())?;
            if ink_attrs.is_empty() {
                continue;
            }
            let normalized =
                ir2::InkAttribute::from_expanded(ink_attrs).map_err(|err| {
                    err.into_combine(format_err_span!(field_span, "at this invokation",))
                })?;
            if normalized.first().kind() != &ir2::AttributeArgKind::Topic {
                return Err(format_err_span!(
                    field_span,
                    "first optional ink! attribute of an event field must be #[ink(topic)]",
                ))
            }
            for arg in normalized.args() {
                return Err(format_err_span!(
                    arg.span(),
                    "encountered conflicting ink! attribute for event field",
                ))
            }
        }
        Ok(Self {
            ast: syn::ItemStruct {
                attrs: other_attrs,
                ..item_struct
            },
        })
    }
}

impl Event {
    /// Returns the identifier of the event struct.
    pub fn ident(&self) -> &Ident {
        &self.ast.ident
    }

    /// Returns an iterator yielding all fields of the event struct.
    pub fn fields(&self) -> syn::punctuated::Iter<syn::Field> {
        self.ast.fields.iter()
    }

    /// Returns an iterator yielding all the `#[ink(topic)]` annotated fields
    /// of the event struct.
    pub fn topic_fields(&self) -> TopicFieldsIter {
        TopicFieldsIter::new(self)
    }
}

/// Iterator yielding all `#[ink(topic)]` annotated fields of an event struct.
pub struct TopicFieldsIter<'a> {
    iter: syn::punctuated::Iter<'a, syn::Field>,
}

impl<'a> TopicFieldsIter<'a> {
    /// Creates a new topics fields iterator for the given ink! event struct.
    fn new(event: &'a Event) -> Self {
        Self {
            iter: event.fields(),
        }
    }
}

impl<'a> Iterator for TopicFieldsIter<'a> {
    type Item = &'a syn::Field;

    fn next(&mut self) -> Option<Self::Item> {
        'outer: loop {
            match self.iter.next() {
                None => return None,
                Some(field) => {
                    if ir2::first_ink_attribute(&field.attrs)
                        .unwrap_or_default()
                        .map(|field| {
                            field.first().kind() == &ir2::AttributeArgKind::Event
                        })
                        .unwrap_or(false)
                    {
                        return Some(field)
                    }
                    continue 'outer
                }
            }
        }
    }
}
    }
}
