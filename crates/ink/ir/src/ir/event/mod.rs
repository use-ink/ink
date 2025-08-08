// Copyright (C) Use Ink (UK) Ltd.
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

mod config;
mod signature_topic;

pub use config::EventConfig;
pub use signature_topic::SignatureTopic;

use proc_macro2::{
    Span,
    TokenStream as TokenStream2,
};
use quote::ToTokens;
use syn::spanned::Spanned as _;

use crate::{
    error::ExtError,
    ir,
    utils::extract_cfg_attributes,
};

/// A checked ink! event with its configuration.
#[derive(Debug, PartialEq, Eq)]
pub struct Event {
    item: syn::ItemStruct,
    config: EventConfig,
}

impl Event {
    /// Returns `Ok` if the input matches all requirements for an ink! event.
    pub fn new(config: TokenStream2, item: TokenStream2) -> Result<Self, syn::Error> {
        let item = syn::parse2::<syn::ItemStruct>(item.clone()).map_err(|err| {
            err.into_combine(format_err_spanned!(
                item,
                "event definition must be a `struct`",
            ))
        })?;
        let parsed_config = syn::parse2::<crate::ast::AttributeArgs>(config)?;
        let config = EventConfig::try_from(parsed_config)?;

        for attr in &item.attrs {
            if attr.path().to_token_stream().to_string().contains("event") {
                return Err(format_err_spanned!(
                    attr,
                    "only one `ink::event` is allowed",
                ));
            }
        }

        Ok(Self { item, config })
    }

    /// Returns the event definition .
    pub fn item(&self) -> &syn::ItemStruct {
        &self.item
    }

    /// Returns `true` if the first ink! annotation on the given struct is
    /// `#[ink(event)]`.
    ///
    /// # Errors
    ///
    /// If the first found ink! attribute is malformed.
    ///
    /// # Note
    ///
    /// This is used for legacy "inline" event definitions, i.e. event definitions that
    /// are defined within a module annotated with `#[ink::contract]`.
    pub(super) fn is_ink_event(
        item_struct: &syn::ItemStruct,
    ) -> Result<bool, syn::Error> {
        if !ir::contains_ink_attributes(&item_struct.attrs) {
            return Ok(false);
        }
        // At this point we know that there must be at least one ink!
        // attribute. This can be either the ink! storage struct,
        // an ink! event or an invalid ink! attribute.
        let attr = ir::first_ink_attribute(&item_struct.attrs)?
            .expect("missing expected ink! attribute for struct");
        Ok(matches!(attr.first().kind(), ir::AttributeArg::Event))
    }

    /// Returns if the event is marked as anonymous, if true then no signature topic is
    /// generated or emitted.
    pub fn anonymous(&self) -> bool {
        self.config.anonymous()
    }

    /// Return manually specified signature topic hash.
    ///
    /// # Note
    ///
    /// Conflicts with `anonymous`
    pub fn signature_topic(&self) -> Option<SignatureTopic> {
        self.config.signature_topic()
    }

    /// Returns a list of `cfg` attributes if any.
    pub fn get_cfg_attrs(&self, span: Span) -> Vec<TokenStream2> {
        extract_cfg_attributes(&self.item.attrs, span)
    }

    /// Returns the event name override (if any).
    pub fn name(&self) -> Option<&str> {
        self.config.name()
    }
}

impl ToTokens for Event {
    /// We mainly implement this trait for this ink! type to have a derived
    /// [`Spanned`](`syn::spanned::Spanned`) implementation for it.
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.item.to_tokens(tokens)
    }
}

impl TryFrom<syn::ItemStruct> for Event {
    type Error = syn::Error;

    fn try_from(item_struct: syn::ItemStruct) -> Result<Self, Self::Error> {
        let struct_span = item_struct.span();
        let (ink_attrs, other_attrs) = ir::sanitize_attributes(
            struct_span,
            item_struct.attrs.clone(),
            &ir::AttributeArgKind::Event,
            |arg| {
                match arg.kind() {
                    ir::AttributeArg::Event
                    | ir::AttributeArg::SignatureTopic(_)
                    | ir::AttributeArg::Anonymous
                    | ir::AttributeArg::Name(_) => Ok(()),
                    _ => Err(None),
                }
            },
        )?;
        if ink_attrs.is_anonymous() && ink_attrs.signature_topic().is_some() {
            return Err(format_err_spanned!(
                item_struct,
                "cannot use use `anonymous` with `signature_topic`",
            ));
        }
        Ok(Self {
            item: syn::ItemStruct {
                attrs: other_attrs,
                ..item_struct
            },
            config: EventConfig::new(
                ink_attrs.is_anonymous(),
                ink_attrs.signature_topic(),
                ink_attrs.name(),
            ),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_try_from_works() {
        let s = "11".repeat(32);
        let item_struct: syn::ItemStruct = syn::parse_quote! {
            #[ink(event)]
            #[ink(signature_topic = #s)]
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
        );
        assert_try_from_fails(
            syn::parse_quote! {
                #[ink(event)]
                #[ink(anonymous)]
                #[ink(anonymous)]
                pub struct MyEvent {
                    #[ink(topic)]
                    field_1: i32,
                    field_2: bool,
                }
            },
            "encountered duplicate ink! attribute",
        );
        let s = "11".repeat(32);
        assert_try_from_fails(
            syn::parse_quote! {
                #[ink(event)]
                #[ink(signature_topic = #s)]
                #[ink(signature_topic = #s)]
                pub struct MyEvent {
                    #[ink(topic)]
                    field_1: i32,
                    field_2: bool,
                }
            },
            "encountered duplicate ink! attribute",
        );
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

    #[test]
    fn anonymous_event_works() {
        fn assert_anonymous_event(event: syn::ItemStruct) {
            match Event::try_from(event) {
                Ok(event) => {
                    assert!(event.anonymous());
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
    #[test]
    fn signature_conflict_fails() {
        let s = "11".repeat(32);
        assert_try_from_fails(
            syn::parse_quote! {
                #[ink(event)]
                #[ink(anonymous)]
                #[ink(signature_topic = #s)]
                pub struct MyEvent {
                    #[ink(topic)]
                    field_1: i32,
                    field_2: bool,
                }
            },
            "cannot use use `anonymous` with `signature_topic`",
        )
    }

    #[test]
    fn signature_invalid_length_fails() {
        let s = "11".repeat(16);
        assert_try_from_fails(
            syn::parse_quote! {
                #[ink(event)]
                #[ink(signature_topic = #s)]
                pub struct MyEvent {
                    #[ink(topic)]
                    field_1: i32,
                    field_2: bool,
                }
            },
            "`signature_topic` is expected to be 32-byte hex string. \
                    Found 16 bytes",
        )
    }

    #[test]
    fn signature_invalid_hex_fails() {
        let s = "XY".repeat(32);
        assert_try_from_fails(
            syn::parse_quote! {
                #[ink(event)]
                #[ink(signature_topic = #s)]
                pub struct MyEvent {
                    #[ink(topic)]
                    field_1: i32,
                    field_2: bool,
                }
            },
            "`signature_topic` has invalid hex string",
        )
    }
}
