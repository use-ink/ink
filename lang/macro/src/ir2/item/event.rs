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

#[derive(Debug, PartialEq, Eq)]
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
                continue
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
                if arg.kind() != &ir2::AttributeArgKind::Topic {
                    return Err(format_err_span!(
                        arg.span(),
                        "encountered conflicting ink! attribute for event field",
                    ))
                }
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
                    if let Some(attr) =
                        ir2::first_ink_attribute(&field.attrs).unwrap_or_default()
                    {
                        if attr.first().kind() == &ir2::AttributeArgKind::Topic {
                            return Some(field)
                        }
                    }
                    continue 'outer
                }
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
}
