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
    ir2,
    ir2::Selector,
};
use core::{
    convert::TryFrom,
    result::Result,
};
use regex::Regex;

/// Either an ink! specific attribute, or another uninterpreted attribute.
#[derive(Debug, PartialEq, Eq)]
pub enum Attribute {
    /// An ink! specific attribute, e.g. `#[ink(storage)]`.
    Ink(InkAttribute),
    /// Any other attribute.
    ///
    /// This can be a known `#[derive(Debug)]` or a specific attribute of another
    /// crate.
    Other(syn::Attribute),
}

/// An ink! specific attribute.
///
/// # Examples
///
/// An attribute with a simple flag:
/// ```no_compile
/// #[ink(storage)]
/// ```
///
/// An attribute with a parameterized flag:
/// ```no_compile
/// #[ink(selector = "0xDEADBEEF")]
/// ```
///
/// An attribute with multiple flags:
/// ```no_compile
/// #[ink(message, payable, selector = "0xDEADBEEF")]
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct InkAttribute {
    /// The internal arguments of the ink! attribute.
    args: Vec<AttributeArgs>,
}

impl InkAttribute {
    /// Returns the first ink! attribute argument.
    ///
    /// # Panics
    ///
    /// If the ink! attribute argument list is empty.
    pub fn first(&self) -> &AttributeArgs {
        self.args
            .first()
            .expect("encountered empty ink! attribute list")
    }

    /// Returns an iterator over the non-empty flags of the ink! attribute.
    ///
    /// # Note
    ///
    /// This yields at least one ink! attribute flag.
    pub fn args(&self) -> core::slice::Iter<AttributeArgs> {
        self.args.iter()
    }
}

/// An ink! specific attribute flag.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum AttributeArgs {
    /// `#[ink(storage)]`
    ///
    /// Applied on `struct` or `enum` types in order to flag them for being
    /// the contract's storage definition.
    Storage,
    /// `#[ink(event)]`
    ///
    /// Applied on `struct` types in order to flag them for being an ink! event.
    Event,
    /// `#[ink(topic)]`
    ///
    /// Applied on fields of ink! event types to indicate that they are topics.
    Topic,
    /// `#[ink(message)]`
    ///
    /// Applied on `&self` or `&mut self` methods to flag them for being an ink!
    /// exported message.
    Message,
    /// `#[ink(constructor)]`
    ///
    /// Applied on inherent methods returning `Self` to flag them for being ink!
    /// exported contract constructors.
    Constructor,
    /// `#[ink(payable)]`
    ///
    /// Applied on ink! constructors or messages in order to specify that they
    /// can receive funds from callers.
    Payable,
    /// `#[ink(selector = "0xDEADBEEF")]`
    ///
    /// Applied on ink! constructors or messages to manually control their
    /// selectors.
    Selector(Selector),
    /// `#[ink(salt = "my_salt_message")]`
    ///
    /// Applied on ink! trait implementation blocks to disambiguate other trait
    /// implementation blocks with equal names.
    Salt(Salt),
    /// `#[ink(impl)]`
    ///
    /// This attribute supports a niche case that is rarely needed.
    ///
    /// Can be applied on ink! implementation blocks in order to make ink! aware
    /// of them. This is useful if such an implementation block doesn't contain
    /// any other ink! attributes, so it would be flagged by ink! as a Rust item.
    /// Adding `#[ink(impl)]` on such implementation blocks makes them treated
    /// as ink! implementation blocks thus allowing to access the environment
    /// etc. Note that ink! messages and constructors still need to be explicitely
    /// flagged as such.
    Implementation,
}

/// An ink! salt applicable to a trait implementation block.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Salt {
    /// The underlying bytes.
    bytes: Vec<u8>,
}

impl From<Vec<u8>> for Salt {
    fn from(bytes: Vec<u8>) -> Self {
        Self { bytes }
    }
}

impl Salt {
    /// Returns the salt as bytes.
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }
}

/// Errors that can occur upon parsing ink! attributes.
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    /// Invalid identifier or structure, e.g. `#[foo(..)]` instead of `#[ink(..)]`.
    Invalid {
        invalid: syn::Attribute,
        reason: String,
    },
    /// ```
    /// #[ink(storage)]
    /// #[ink(storage)]
    /// pub struct MyStorage { .. }
    /// ```
    DuplicateAttributes {
        fst: InkAttribute,
        snd: InkAttribute,
    },
    /// `#[ink(unknown_flag)]` or `#[ink(selector = true)]`
    InvalidArg {
        invalid: syn::NestedMeta,
        reason: String,
    },
    /// `#[ink(message, message)]`
    DuplicateArgs {
        fst: AttributeArgs,
        snd: AttributeArgs,
    },
}

impl Error {
    /// Creates a new `InvalidFlag` error.
    pub fn invalid<S>(origin: syn::Attribute, reason: S) -> Self
    where
        S: Into<String>,
    {
        Self::Invalid {
            invalid: origin,
            reason: reason.into(),
        }
    }

    /// Creates a new `InvalidFlag` error.
    pub fn invalid_flag<S>(origin: syn::NestedMeta, reason: S) -> Self
    where
        S: Into<String>,
    {
        Self::InvalidArg {
            invalid: origin,
            reason: reason.into(),
        }
    }
}

/// Returns `true` if the given iterator yields at least one attribute of the form
/// `#[ink(..)]` or `#[ink]`.
///
/// # Note
///
/// This does not check at this point whether the ink! attribute is valid since
/// this check is optimized for efficiency.
pub fn contains_ink_attributes<'a, I>(attrs: I) -> bool
where
    I: IntoIterator<Item = &'a syn::Attribute>,
{
    attrs
        .into_iter()
        .filter(|attr| attr.path.is_ident("ink"))
        .next()
        .is_some()
}

/// Returns the first valid ink! attribute, if any.
///
/// Returns `None` if there are no ink! attributes.
///
/// # Errors
///
/// Returns an error if the first ink! attribute is invalid.
pub fn first_ink_attribute<'a, I>(attrs: I) -> Result<Option<ir2::InkAttribute>, Error>
where
    I: IntoIterator<Item = &'a syn::Attribute>,
{
    let first = attrs
        .into_iter()
        .filter(|attr| attr.path.is_ident("ink"))
        .next();
    match first {
        None => Ok(None),
        Some(ink_attr) => InkAttribute::try_from(ink_attr.clone()).map(Some),
    }
}

/// Partitions the given attributes into ink! specific and non-ink! specific attributes.
///
/// # Error
///
/// Returns an error if some ink! specific attributes could not be successfully parsed.
pub fn partition_attributes<I>(
    attrs: I,
) -> Result<(Vec<InkAttribute>, Vec<syn::Attribute>), Error>
where
    I: IntoIterator<Item = syn::Attribute>,
{
    use either::Either;
    use itertools::Itertools as _;
    let (ink_attrs, others) = attrs
        .into_iter()
        .map(|attr| <Attribute as TryFrom<_>>::try_from(attr))
        .collect::<Result<Vec<Attribute>, Error>>()?
        .into_iter()
        .partition_map(|attr| {
            match attr {
                Attribute::Ink(ink_attr) => Either::Left(ink_attr),
                Attribute::Other(other_attr) => Either::Right(other_attr),
            }
        });
    Attribute::ensure_no_duplicates(&ink_attrs)?;
    Ok((ink_attrs, others))
}

impl Attribute {
    /// Returns `Ok` if the given iterator yields no duplicate ink! attributes.
    ///
    /// # Errors
    ///
    /// If the given iterator yields duplicate ink! attributes.
    /// Note: Duplicate non-ink! attributes are fine.
    fn ensure_no_duplicates<'a, I>(flags: I) -> Result<(), Error>
    where
        I: IntoIterator<Item = &'a InkAttribute>,
    {
        use std::collections::BTreeSet;
        let mut seen: BTreeSet<&InkAttribute> = BTreeSet::new();
        for flag in flags.into_iter() {
            if let Some(seen) = seen.get(flag) {
                return Err(Error::DuplicateAttributes {
                    // A call to `seen.clone()` clones the reference for whatever reason ...
                    fst: <InkAttribute as Clone>::clone(seen),
                    snd: flag.clone(),
                })
            }
            seen.insert(flag);
        }
        Ok(())
    }
}

impl TryFrom<syn::Attribute> for Attribute {
    type Error = Error;

    fn try_from(attr: syn::Attribute) -> Result<Self, Self::Error> {
        if attr.path.is_ident("ink") {
            return <InkAttribute as TryFrom<_>>::try_from(attr).map(Into::into)
        }
        Ok(Attribute::Other(attr))
    }
}

impl From<InkAttribute> for Attribute {
    fn from(ink_attribute: InkAttribute) -> Self {
        Attribute::Ink(ink_attribute)
    }
}

impl TryFrom<syn::Attribute> for InkAttribute {
    type Error = Error;

    fn try_from(attr: syn::Attribute) -> Result<Self, Self::Error> {
        if !attr.path.is_ident("ink") {
            return Err(Error::invalid(attr, "unexpected non-ink! attribute"))
        }
        match attr.parse_meta().map_err(|_| {
            Error::invalid(attr.clone(), "unexpected ink! attribute structure")
        })? {
            syn::Meta::List(meta_list) => {
                let flags = meta_list
                    .nested
                    .into_iter()
                    .map(|nested| <AttributeArgs as TryFrom<_>>::try_from(nested))
                    .collect::<Result<Vec<_>, Error>>()?;
                Self::ensure_no_duplicate_flags(&flags)?;
                Ok(InkAttribute { args: flags })
            }
            _ => Err(Error::invalid(attr, "unknown ink! attribute")),
        }
    }
}

impl InkAttribute {
    /// Returns `Ok` if the given iterator yields no duplicate attribute flags.
    ///
    /// # Errors
    ///
    /// If the given iterator yields duplicate attribute flags.
    fn ensure_no_duplicate_flags<'a, I>(flags: I) -> Result<(), Error>
    where
        I: IntoIterator<Item = &'a AttributeArgs>,
    {
        use std::collections::BTreeSet;
        let mut seen: BTreeSet<&AttributeArgs> = BTreeSet::new();
        for flag in flags.into_iter() {
            if let Some(seen) = seen.get(flag) {
                return Err(Error::DuplicateArgs {
                    // A call to `seen.clone()` clones the reference for whatever reason ...
                    fst: <AttributeArgs as Clone>::clone(seen),
                    snd: flag.clone(),
                })
            }
            seen.insert(flag);
        }
        Ok(())
    }
}

impl TryFrom<syn::NestedMeta> for AttributeArgs {
    type Error = Error;

    fn try_from(nested_meta: syn::NestedMeta) -> Result<Self, Self::Error> {
        match &nested_meta {
            syn::NestedMeta::Meta(meta) => {
                match meta {
                    syn::Meta::NameValue(name_value) => {
                        if name_value.path.is_ident("selector") {
                            if let syn::Lit::Str(lit_str) = &name_value.lit {
                                let regex = Regex::new(
                                    r"0x([\da-fA-F]{2})([\da-fA-F]{2})([\da-fA-F]{2})([\da-fA-F]{2})"
                                ).map_err(|_| Error::invalid_flag(nested_meta.clone(), "invalid selector bytes"))?;
                                let str = lit_str.value();
                                let cap = regex.captures(&str).unwrap();
                                let selector_bytes = [
                                    u8::from_str_radix(&cap[1], 16).expect(
                                        "encountered non-hex digit at position 0",
                                    ),
                                    u8::from_str_radix(&cap[2], 16).expect(
                                        "encountered non-hex digit at position 1",
                                    ),
                                    u8::from_str_radix(&cap[3], 16).expect(
                                        "encountered non-hex digit at position 2",
                                    ),
                                    u8::from_str_radix(&cap[4], 16).expect(
                                        "encountered non-hex digit at position 3",
                                    ),
                                ];
                                return Ok(AttributeArgs::Selector(Selector::new(
                                    selector_bytes,
                                )))
                            }
                        }
                        if name_value.path.is_ident("salt") {
                            if let syn::Lit::Str(lit_str) = &name_value.lit {
                                let bytes = lit_str.value().into_bytes();
                                return Ok(AttributeArgs::Salt(Salt::from(bytes)))
                            }
                        }
                        Err(Error::invalid_flag(
                            nested_meta,
                            "unknown ink! marker (name = value)",
                        ))
                    }
                    syn::Meta::Path(path) => {
                        if path.is_ident("storage") {
                            return Ok(AttributeArgs::Storage)
                        }
                        if path.is_ident("message") {
                            return Ok(AttributeArgs::Message)
                        }
                        if path.is_ident("constructor") {
                            return Ok(AttributeArgs::Constructor)
                        }
                        if path.is_ident("event") {
                            return Ok(AttributeArgs::Event)
                        }
                        if path.is_ident("topic") {
                            return Ok(AttributeArgs::Topic)
                        }
                        if path.is_ident("payable") {
                            return Ok(AttributeArgs::Payable)
                        }
                        if path.is_ident("impl") {
                            return Ok(AttributeArgs::Implementation)
                        }
                        Err(Error::invalid_flag(
                            nested_meta,
                            "unknown ink! marker (path)",
                        ))
                    }
                    syn::Meta::List(_) => {
                        Err(Error::invalid_flag(
                            nested_meta,
                            "unknown ink! marker (list)",
                        ))
                    }
                }
            }
            syn::NestedMeta::Lit(_) => {
                Err(Error::invalid_flag(
                    nested_meta,
                    "unknown ink! marker (lit)",
                ))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contains_ink_attributes_works() {
        assert!(!contains_ink_attributes(&[]));
        assert!(contains_ink_attributes(&[syn::parse_quote! { #[ink] }]));
        assert!(contains_ink_attributes(&[syn::parse_quote! { #[ink(..)] }]));
        assert!(contains_ink_attributes(&[
            syn::parse_quote! { #[inline] },
            syn::parse_quote! { #[likely] },
            syn::parse_quote! { #[ink(storage)] },
        ]));
        assert!(!contains_ink_attributes(&[
            syn::parse_quote! { #[inline] },
            syn::parse_quote! { #[likely] },
        ]));
    }

    #[test]
    fn first_ink_attribute_works() {
        assert_eq!(first_ink_attribute(&[]), Ok(None),);
        assert_eq!(
            first_ink_attribute(&[syn::parse_quote! { #[ink(storage)] }]),
            Ok(Some(InkAttribute {
                args: vec![AttributeArgs::Storage,]
            })),
        );
        {
            let invalid = syn::parse_quote! { invalid };
            assert_eq!(
                first_ink_attribute(&[syn::parse_quote! { #[ink(invalid)] }]),
                Err(Error::invalid_flag(invalid, "unknown ink! marker (path)")),
            );
        }
    }

    #[test]
    fn storage_works() {
        let attr: syn::Attribute = syn::parse_quote! {
            #[ink(storage)]
        };
        assert_eq!(
            <Attribute as TryFrom<_>>::try_from(attr),
            Ok(Attribute::Ink(InkAttribute {
                args: vec![AttributeArgs::Storage]
            }))
        );
    }

    #[test]
    fn impl_works() {
        let attr: syn::Attribute = syn::parse_quote! {
            #[ink(impl)]
        };
        assert_eq!(
            <Attribute as TryFrom<_>>::try_from(attr),
            Ok(Attribute::Ink(InkAttribute {
                args: vec![AttributeArgs::Implementation]
            }))
        );
    }

    #[test]
    fn selector_works() {
        let attr: syn::Attribute = syn::parse_quote! {
            #[ink(selector = "0xDEADBEEF")]
        };
        assert_eq!(
            <Attribute as TryFrom<_>>::try_from(attr),
            Ok(Attribute::Ink(InkAttribute {
                args: vec![AttributeArgs::Selector(Selector::new([
                    0xDE, 0xAD, 0xBE, 0xEF
                ]))]
            }))
        );
    }

    #[test]
    fn salt_works() {
        let attr: syn::Attribute = syn::parse_quote! {
            #[ink(salt = "take my salt!")]
        };
        assert_eq!(
            <Attribute as TryFrom<_>>::try_from(attr),
            Ok(Attribute::Ink(InkAttribute {
                args: vec![AttributeArgs::Salt(Salt::from(
                    "take my salt!".to_string().into_bytes()
                ))]
            }))
        );
    }

    #[test]
    fn compound_mixed_works() {
        let attr: syn::Attribute = syn::parse_quote! {
            #[ink(message, salt = "message_salt")]
        };
        assert_eq!(
            <Attribute as TryFrom<_>>::try_from(attr),
            Ok(Attribute::Ink(InkAttribute {
                args: vec![
                    AttributeArgs::Message,
                    AttributeArgs::Salt(Salt::from(
                        "message_salt".to_string().into_bytes()
                    )),
                ]
            }))
        );
    }

    #[test]
    fn compound_simple_works() {
        let attr: syn::Attribute = syn::parse_quote! {
            #[ink(
                storage,
                message,
                constructor,
                event,
                topic,
                payable,
                impl,
            )]
        };
        assert_eq!(
            <Attribute as TryFrom<_>>::try_from(attr),
            Ok(Attribute::Ink(InkAttribute {
                args: vec![
                    AttributeArgs::Storage,
                    AttributeArgs::Message,
                    AttributeArgs::Constructor,
                    AttributeArgs::Event,
                    AttributeArgs::Topic,
                    AttributeArgs::Payable,
                    AttributeArgs::Implementation,
                ]
            }))
        );
    }

    #[test]
    fn non_ink_attribute_works() {
        let attr: syn::Attribute = syn::parse_quote! {
            #[non_ink(message)]
        };
        assert_eq!(
            <Attribute as TryFrom<_>>::try_from(attr.clone()),
            Ok(Attribute::Other(attr.clone())),
        );
    }

    #[test]
    fn empty_ink_attribute_fails() {
        let naked: syn::Attribute = syn::parse_quote! {
            #[ink]
        };
        assert_eq!(
            <Attribute as TryFrom<_>>::try_from(naked.clone()),
            Err(Error::invalid(naked.clone(), "unknown ink! attribute"))
        );
        let no_args: syn::Attribute = syn::parse_quote! {
            #[ink]
        };
        assert_eq!(
            <Attribute as TryFrom<_>>::try_from(no_args.clone()),
            Err(Error::invalid(no_args.clone(), "unknown ink! attribute"))
        );
    }

    #[test]
    fn duplicate_flags_fails() {
        let duplicate_flags: syn::Attribute = syn::parse_quote! {
            #[ink(message, message)]
        };
        assert_eq!(
            <Attribute as TryFrom<_>>::try_from(duplicate_flags),
            Err(Error::DuplicateArgs {
                fst: AttributeArgs::Message,
                snd: AttributeArgs::Message,
            })
        );
    }

    #[test]
    fn parition_attributes_works() {
        let duplicate_attrs: Vec<syn::Attribute> = vec![
            syn::parse_quote! { #[ink(message)] },
            syn::parse_quote! { #[non_ink_attribute] },
        ];
        let ink_attr = <InkAttribute as TryFrom<syn::Attribute>>::try_from(
            syn::parse_quote! { #[ink(message)] },
        )
        .unwrap();
        let non_ink_attr: syn::Attribute = syn::parse_quote! { #[non_ink_attribute] };
        assert_eq!(
            partition_attributes(duplicate_attrs),
            Ok((vec![ink_attr], vec![non_ink_attr]))
        );
    }

    #[test]
    fn parition_duplicates_fails() {
        let duplicate_attrs: Vec<syn::Attribute> = vec![
            syn::parse_quote! { #[ink(message)] },
            syn::parse_quote! { #[ink(message)] },
        ];
        let dupe = <InkAttribute as TryFrom<syn::Attribute>>::try_from(
            syn::parse_quote! { #[ink(message)] },
        )
        .unwrap();
        assert_eq!(
            partition_attributes(duplicate_attrs),
            Err(Error::DuplicateAttributes {
                fst: dupe.clone(),
                snd: dupe.clone(),
            })
        );
    }
}
