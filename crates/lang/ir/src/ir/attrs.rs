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
    ir,
    ir::{Extension, Selector},
};
use core::{
    convert::TryFrom,
    result::Result,
};
use proc_macro2::{
    Ident,
    Span,
};
use regex::Regex;
use syn::spanned::Spanned;

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

/// Types implementing this trait can return a slice over their `syn` attributes.
pub trait Attrs {
    /// Returns the slice of attributes of an AST entity.
    fn attrs(&self) -> &[syn::Attribute];
}

impl Attrs for syn::ImplItem {
    fn attrs(&self) -> &[syn::Attribute] {
        match self {
            syn::ImplItem::Const(item) => &item.attrs,
            syn::ImplItem::Method(item) => &item.attrs,
            syn::ImplItem::Type(item) => &item.attrs,
            syn::ImplItem::Macro(item) => &item.attrs,
            _ => &[],
        }
    }
}

impl Attrs for syn::Item {
    fn attrs(&self) -> &[syn::Attribute] {
        use syn::Item;
        match self {
            Item::Const(syn::ItemConst { attrs, .. })
            | Item::Enum(syn::ItemEnum { attrs, .. })
            | Item::ExternCrate(syn::ItemExternCrate { attrs, .. })
            | Item::Fn(syn::ItemFn { attrs, .. })
            | Item::ForeignMod(syn::ItemForeignMod { attrs, .. })
            | Item::Impl(syn::ItemImpl { attrs, .. })
            | Item::Macro(syn::ItemMacro { attrs, .. })
            | Item::Macro2(syn::ItemMacro2 { attrs, .. })
            | Item::Mod(syn::ItemMod { attrs, .. })
            | Item::Static(syn::ItemStatic { attrs, .. })
            | Item::Struct(syn::ItemStruct { attrs, .. })
            | Item::Trait(syn::ItemTrait { attrs, .. })
            | Item::TraitAlias(syn::ItemTraitAlias { attrs, .. })
            | Item::Type(syn::ItemType { attrs, .. })
            | Item::Union(syn::ItemUnion { attrs, .. })
            | Item::Use(syn::ItemUse { attrs, .. }) => attrs,
            _ => &[],
        }
    }
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
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InkAttribute {
    /// The internal non-empty sequence of arguments of the ink! attribute.
    args: Vec<AttributeArg>,
}

impl Spanned for InkAttribute {
    fn span(&self) -> Span {
        self.args
            .iter()
            .map(|arg| arg.span())
            .fold(self.first().span(), |fst, snd| {
                fst.join(snd).unwrap_or_else(|| self.first().span())
            })
    }
}

impl InkAttribute {
    /// Ensure that the first ink! attribute argument is of expected kind.
    ///
    /// # Errors
    ///
    /// If the first ink! attribute argument is not of expected kind.
    pub fn ensure_first(&self, expected: &AttributeArgKind) -> Result<(), syn::Error> {
        if &self.first().kind != expected {
            return Err(format_err!(
                self.span(),
                "unexpected first ink! attribute argument",
            ))
        }
        Ok(())
    }

    /// Ensures that the given iterator of ink! attribute arguments do not have
    /// duplicates.
    ///
    /// # Errors
    ///
    /// If the given iterator yields duplicate ink! attribute arguments.
    fn ensure_no_duplicate_args<'a, A>(args: A) -> Result<(), syn::Error>
    where
        A: IntoIterator<Item = &'a ir::AttributeArg>,
    {
        use crate::error::ExtError as _;
        use std::collections::HashSet;
        let mut seen: HashSet<&AttributeArg> = HashSet::new();
        for arg in args.into_iter() {
            if let Some(seen) = seen.get(arg) {
                return Err(format_err!(
                    arg.span(),
                    "encountered duplicate ink! attribute arguments"
                )
                .into_combine(format_err!(
                    seen.span(),
                    "first equal ink! attribute argument here"
                )))
            }
            seen.insert(arg);
        }
        Ok(())
    }

    /// Converts a sequence of `#[ink(..)]` attributes into a single flattened
    /// `#[ink(..)]` attribute that contains all of the input arguments.
    ///
    /// # Example
    ///
    /// Given the input ink! attribute sequence `[ #[ink(message)], #[ink(payable)] ]`
    /// this procedure returns the single attribute `#[ink(message, payable)]`.
    ///
    /// # Errors
    ///
    /// - If the sequence of input ink! attributes contains duplicates.
    /// - If the input sequence is empty.
    pub fn from_expanded<A>(attrs: A) -> Result<Self, syn::Error>
    where
        A: IntoIterator<Item = Self>,
    {
        let args = attrs
            .into_iter()
            .map(|attr| attr.args)
            .flatten()
            .collect::<Vec<_>>();
        if args.is_empty() {
            return Err(format_err!(
                Span::call_site(),
                "encountered unexpected empty expanded ink! attribute arguments",
            ))
        }
        Self::ensure_no_duplicate_args(&args)?;
        Ok(Self { args })
    }

    /// Returns the first ink! attribute argument.
    pub fn first(&self) -> &AttributeArg {
        self.args
            .first()
            .expect("encountered invalid empty ink! attribute list")
    }

    /// Returns an iterator over the non-empty flags of the ink! attribute.
    ///
    /// # Note
    ///
    /// This yields at least one ink! attribute flag.
    pub fn args(&self) -> ::core::slice::Iter<AttributeArg> {
        self.args.iter()
    }

    /// Returns the namespace of the ink! attribute if any.
    pub fn namespace(&self) -> Option<ir::Namespace> {
        self.args().find_map(|arg| {
            if let ir::AttributeArgKind::Namespace(namespace) = arg.kind() {
                return Some(namespace.clone())
            }
            None
        })
    }

    /// Returns the selector of the ink! attribute if any.
    pub fn selector(&self) -> Option<ir::Selector> {
        self.args().find_map(|arg| {
            if let ir::AttributeArgKind::Selector(selector) = arg.kind() {
                return Some(*selector)
            }
            None
        })
    }

    /// Returns `true` if the ink! attribute contains the `payable` argument.
    pub fn is_payable(&self) -> bool {
        self.args()
            .any(|arg| matches!(arg.kind(), AttributeArgKind::Payable))
    }

    /// Returns `true` if the ink! attribute contains the `anonymous` argument.
    pub fn is_anonymous(&self) -> bool {
        self.args()
            .any(|arg| matches!(arg.kind(), AttributeArgKind::Anonymous))
    }
}

/// An ink! specific attribute argument.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AttributeArg {
    pub ast: syn::Meta,
    pub kind: AttributeArgKind,
}

impl AttributeArg {
    /// Returns a shared reference to the attribute argument kind.
    pub fn kind(&self) -> &AttributeArgKind {
        &self.kind
    }
}

impl Spanned for AttributeArg {
    fn span(&self) -> Span {
        self.ast.span()
    }
}

/// An ink! specific attribute flag.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AttributeArgKind {
    /// `#[ink(storage)]`
    ///
    /// Applied on `struct` types in order to flag them for being the
    /// contract's storage definition.
    Storage,
    /// `#[ink(event)]`
    ///
    /// Applied on `struct` types in order to flag them for being an ink! event.
    Event,
    /// `#[ink(anonymous)]`
    ///
    /// Applied on `struct` event types in order to flag them as anonymous.
    /// Anonymous events have similar semantics as in Solidity in that their
    /// event signature won't be included in their event topics serialization
    /// to reduce event emitting overhead. This is especially useful for user
    /// defined events.
    Anonymous,
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
    /// `#[ink(extension = N: usize)]`
    ///
    /// Applies on ink! chain extension method to set their `func_id` parameter.
    /// Every chain extension method must have exactly one ink! `extension` attribute.
    Extension(Extension),
    /// `#[ink(namespace = "my_namespace")]`
    ///
    /// Applied on ink! trait implementation blocks to disambiguate other trait
    /// implementation blocks with equal names.
    Namespace(Namespace),
    /// `#[ink(impl)]`
    ///
    /// This attribute supports a niche case that is rarely needed.
    ///
    /// Can be applied on ink! implementation blocks in order to make ink! aware
    /// of them. This is useful if such an implementation block doesn't contain
    /// any other ink! attributes, so it would be flagged by ink! as a Rust item.
    /// Adding `#[ink(impl)]` on such implementation blocks makes them treated
    /// as ink! implementation blocks thus allowing to access the environment
    /// etc. Note that ink! messages and constructors still need to be explicitly
    /// flagged as such.
    Implementation,
}

impl core::fmt::Display for AttributeArgKind {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> Result<(), core::fmt::Error> {
        match self {
            Self::Storage => write!(f, "storage"),
            Self::Event => write!(f, "event"),
            Self::Anonymous => write!(f, "anonymous"),
            Self::Topic => write!(f, "topic"),
            Self::Message => write!(f, "message"),
            Self::Constructor => write!(f, "constructor"),
            Self::Payable => write!(f, "payable"),
            Self::Selector(selector) => write!(f, "selector = {:?}", selector.as_bytes()),
            Self::Extension(extension) => {
                write!(f, "extension = {:?}", extension.id())
            }
            Self::Namespace(namespace) => {
                write!(f, "namespace = {:?}", namespace.as_bytes())
            }
            Self::Implementation => write!(f, "impl"),
        }
    }
}

/// An ink! namespace applicable to a trait implementation block.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Namespace {
    /// The underlying bytes.
    bytes: Vec<u8>,
}

impl From<Vec<u8>> for Namespace {
    fn from(bytes: Vec<u8>) -> Self {
        Self { bytes }
    }
}

impl Namespace {
    /// Returns the namespace as bytes.
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
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
    attrs.into_iter().any(|attr| attr.path.is_ident("ink"))
}

/// Returns the first valid ink! attribute, if any.
///
/// Returns `None` if there are no ink! attributes.
///
/// # Errors
///
/// Returns an error if the first ink! attribute is invalid.
pub fn first_ink_attribute<'a, I>(
    attrs: I,
) -> Result<Option<ir::InkAttribute>, syn::Error>
where
    I: IntoIterator<Item = &'a syn::Attribute>,
{
    let first = attrs.into_iter().find(|attr| attr.path.is_ident("ink"));
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
) -> Result<(Vec<InkAttribute>, Vec<syn::Attribute>), syn::Error>
where
    I: IntoIterator<Item = syn::Attribute>,
{
    use either::Either;
    use itertools::Itertools as _;
    let (ink_attrs, others) = attrs
        .into_iter()
        .map(<Attribute as TryFrom<_>>::try_from)
        .collect::<Result<Vec<Attribute>, syn::Error>>()?
        .into_iter()
        .partition_map(|attr| {
            match attr {
                Attribute::Ink(ink_attr) => Either::Left(ink_attr),
                Attribute::Other(other_attr) => Either::Right(other_attr),
            }
        });
    Attribute::ensure_no_duplicate_attrs(&ink_attrs)?;
    Ok((ink_attrs, others))
}

/// Sanitizes the given attributes.
///
/// This partitions the attributes into ink! and non-ink! attributes.
/// All ink! attributes are normalized, they are checked to have a valid first
/// ink! attribute argument and no conflicts given the conflict predicate.
///
/// Returns the partitioned ink! and non-ink! attributes.
///
/// # Errors
///
/// - If there are invalid ink! attributes.
/// - If there are duplicate ink! attributes.
/// - If the first ink! attribute is not matching the expected.
/// - If there are conflicting ink! attributes.
pub fn sanitize_attributes<I, C>(
    parent_span: Span,
    attrs: I,
    is_valid_first: &ir::AttributeArgKind,
    mut is_conflicting_attr: C,
) -> Result<(InkAttribute, Vec<syn::Attribute>), syn::Error>
where
    I: IntoIterator<Item = syn::Attribute>,
    C: FnMut(&AttributeArgKind) -> bool,
{
    let (ink_attrs, other_attrs) = ir::partition_attributes(attrs)?;
    let normalized = ir::InkAttribute::from_expanded(ink_attrs).map_err(|err| {
        err.into_combine(format_err!(parent_span, "at this invocation",))
    })?;
    normalized.ensure_first(is_valid_first).map_err(|err| {
        err.into_combine(format_err!(
            parent_span,
            "expected {} as first ink! attribute argument",
            is_valid_first,
        ))
    })?;
    normalized.ensure_no_conflicts(|arg| is_conflicting_attr(arg.kind()))?;
    Ok((normalized, other_attrs))
}

impl Attribute {
    /// Returns `Ok` if the given iterator yields no duplicate ink! attributes.
    ///
    /// # Errors
    ///
    /// If the given iterator yields duplicate ink! attributes.
    /// Note: Duplicate non-ink! attributes are fine.
    fn ensure_no_duplicate_attrs<'a, I>(attrs: I) -> Result<(), syn::Error>
    where
        I: IntoIterator<Item = &'a InkAttribute>,
    {
        use std::collections::HashSet;
        let mut seen: HashSet<&InkAttribute> = HashSet::new();
        for attr in attrs.into_iter() {
            if let Some(seen) = seen.get(attr) {
                use crate::error::ExtError as _;
                return Err(format_err!(
                    attr.span(),
                    "encountered duplicate ink! attribute"
                )
                .into_combine(format_err!(seen.span(), "first ink! attribute here")))
            }
            seen.insert(attr);
        }
        Ok(())
    }
}

impl TryFrom<syn::Attribute> for Attribute {
    type Error = syn::Error;

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
    type Error = syn::Error;

    fn try_from(attr: syn::Attribute) -> Result<Self, Self::Error> {
        if !attr.path.is_ident("ink") {
            return Err(format_err_spanned!(attr, "unexpected non-ink! attribute"))
        }
        match attr.parse_meta().map_err(|_| {
            format_err_spanned!(attr, "unexpected ink! attribute structure")
        })? {
            syn::Meta::List(meta_list) => {
                let args = meta_list
                    .nested
                    .into_iter()
                    .map(<AttributeArg as TryFrom<_>>::try_from)
                    .collect::<Result<Vec<_>, syn::Error>>()?;
                Self::ensure_no_duplicate_args(&args)?;
                if args.is_empty() {
                    return Err(format_err_spanned!(
                        attr,
                        "encountered unsupported empty ink! attribute"
                    ))
                }
                Ok(InkAttribute { args })
            }
            _ => Err(format_err_spanned!(attr, "unknown ink! attribute")),
        }
    }
}

impl InkAttribute {
    /// Ensures that there are no conflicting ink! attribute arguments in `self`.
    ///
    /// The given `is_conflicting` describes for every ink! attribute argument
    /// found in `self` if it is in conflict.
    pub fn ensure_no_conflicts<'a, P>(
        &'a self,
        mut is_conflicting: P,
    ) -> Result<(), syn::Error>
    where
        P: FnMut(&'a ir::AttributeArg) -> bool,
    {
        for arg in self.args() {
            if is_conflicting(arg) {
                return Err(format_err!(
                    arg.span(),
                    "encountered conflicting ink! attribute argument",
                ))
            }
        }
        Ok(())
    }
}

/// Returns an error to notify about non-hex digits at a position.
fn err_non_hex(meta: &syn::Meta, pos: usize) -> syn::Error {
    format_err_spanned!(meta, "encountered non-hex digit at position {}", pos)
}

/// Returns an error to notify about an invalid ink! selector.
fn invalid_selector_err_regex(meta: &syn::Meta) -> syn::Error {
    format_err_spanned!(
        meta,
        "invalid selector - a selector must consist of four bytes in hex (e.g. `selector = \"0xCAFEBABE\"`)"
    )
}

impl TryFrom<syn::NestedMeta> for AttributeArg {
    type Error = syn::Error;

    fn try_from(nested_meta: syn::NestedMeta) -> Result<Self, Self::Error> {
        match nested_meta {
            syn::NestedMeta::Meta(meta) => {
                match &meta {
                    syn::Meta::NameValue(name_value) => {
                        if name_value.path.is_ident("selector") {
                            if let syn::Lit::Str(lit_str) = &name_value.lit {
                                let regex = Regex::new(
                                    r"0x([\da-fA-F]{2})([\da-fA-F]{2})([\da-fA-F]{2})([\da-fA-F]{2})"
                                ).map_err(|_| invalid_selector_err_regex(&meta))?;
                                let str = lit_str.value();
                                let cap = regex
                                    .captures(&str)
                                    .ok_or_else(|| invalid_selector_err_regex(&meta))?;
                                let selector_bytes = [
                                    u8::from_str_radix(&cap[1], 16)
                                        .map_err(|_| err_non_hex(&meta, 0))?,
                                    u8::from_str_radix(&cap[2], 16)
                                        .map_err(|_| err_non_hex(&meta, 1))?,
                                    u8::from_str_radix(&cap[3], 16)
                                        .map_err(|_| err_non_hex(&meta, 2))?,
                                    u8::from_str_radix(&cap[4], 16)
                                        .map_err(|_| err_non_hex(&meta, 3))?,
                                ];
                                return Ok(AttributeArg {
                                    ast: meta,
                                    kind: AttributeArgKind::Selector(Selector::new(
                                        selector_bytes,
                                    )),
                                })
                            }
                        }
                        if name_value.path.is_ident("namespace") {
                            if let syn::Lit::Str(lit_str) = &name_value.lit {
                                let bytes = lit_str.value().into_bytes();
                                return Ok(AttributeArg {
                                    ast: meta,
                                    kind: AttributeArgKind::Namespace(Namespace::from(
                                        bytes,
                                    )),
                                })
                            }
                        }
                        Err(format_err_spanned!(
                            meta,
                            "unknown ink! attribute argument (name = value)",
                        ))
                    }
                    syn::Meta::Path(path) => {
                        let kind: Option<AttributeArgKind> =
                            path.get_ident().map(Ident::to_string).and_then(|ident| {
                                match ident.as_str() {
                                    "storage" => Some(AttributeArgKind::Storage),
                                    "message" => Some(AttributeArgKind::Message),
                                    "constructor" => Some(AttributeArgKind::Constructor),
                                    "event" => Some(AttributeArgKind::Event),
                                    "anonymous" => Some(AttributeArgKind::Anonymous),
                                    "topic" => Some(AttributeArgKind::Topic),
                                    "payable" => Some(AttributeArgKind::Payable),
                                    "impl" => Some(AttributeArgKind::Implementation),
                                    _ => None,
                                }
                            });
                        if let Some(kind) = kind {
                            return Ok(AttributeArg { ast: meta, kind })
                        }
                        Err(format_err_spanned!(meta, "unknown ink! attribute (path)"))
                    }
                    syn::Meta::List(_) => {
                        Err(format_err_spanned!(
                            meta,
                            "unknown ink! attribute argument (list)"
                        ))
                    }
                }
            }
            syn::NestedMeta::Lit(_) => {
                Err(format_err_spanned!(
                    nested_meta,
                    "unknown ink! attribute argument (literal)"
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

    /// Asserts that the given input yields the expected first argument or the
    /// expected error string.
    ///
    /// # Note
    ///
    /// Can be used to assert against the success and failure path.
    fn assert_first_ink_attribute(
        input: &[syn::Attribute],
        expected: Result<Option<Vec<ir::AttributeArgKind>>, &'static str>,
    ) {
        assert_eq!(
            first_ink_attribute(input)
                .map(|maybe_attr: Option<ir::InkAttribute>| {
                    maybe_attr.map(|attr: ir::InkAttribute| {
                        attr.args
                            .into_iter()
                            .map(|arg| arg.kind)
                            .collect::<Vec<_>>()
                    })
                })
                .map_err(|err| err.to_string()),
            expected.map_err(ToString::to_string),
        )
    }

    #[test]
    fn first_ink_attribute_works() {
        assert_first_ink_attribute(&[], Ok(None));
        assert_first_ink_attribute(
            &[syn::parse_quote! { #[ink(storage)] }],
            Ok(Some(vec![AttributeArgKind::Storage])),
        );
        assert_first_ink_attribute(
            &[syn::parse_quote! { #[ink(invalid)] }],
            Err("unknown ink! attribute (path)"),
        );
    }

    mod test {
        use crate::ir;

        /// Mock for `ir::Attribute` to improve testability.
        #[derive(Debug, PartialEq, Eq)]
        pub enum Attribute {
            Ink(Vec<ir::AttributeArgKind>),
            Other(syn::Attribute),
        }

        impl From<ir::Attribute> for Attribute {
            fn from(attr: ir::Attribute) -> Self {
                match attr {
                    ir::Attribute::Ink(ink_attr) => {
                        Self::Ink(
                            ink_attr
                                .args
                                .into_iter()
                                .map(|arg| arg.kind)
                                .collect::<Vec<_>>(),
                        )
                    }
                    ir::Attribute::Other(other_attr) => Self::Other(other_attr),
                }
            }
        }

        impl From<ir::InkAttribute> for Attribute {
            fn from(ink_attr: ir::InkAttribute) -> Self {
                Attribute::from(ir::Attribute::Ink(ink_attr))
            }
        }

        /// Mock for `ir::InkAttribute` to improve testability.
        #[derive(Debug, PartialEq, Eq)]
        pub struct InkAttribute {
            args: Vec<ir::AttributeArgKind>,
        }

        impl From<ir::InkAttribute> for InkAttribute {
            fn from(ink_attr: ir::InkAttribute) -> Self {
                Self {
                    args: ink_attr
                        .args
                        .into_iter()
                        .map(|arg| arg.kind)
                        .collect::<Vec<_>>(),
                }
            }
        }

        impl<I> From<I> for InkAttribute
        where
            I: IntoIterator<Item = ir::AttributeArgKind>,
        {
            fn from(args: I) -> Self {
                Self {
                    args: args.into_iter().collect::<Vec<_>>(),
                }
            }
        }
    }

    /// Asserts that the given [`syn::Attribute`] is converted into the expected
    /// [`ir::Attribute]` or yields the expected error message.
    fn assert_attribute_try_from(
        input: syn::Attribute,
        expected: Result<test::Attribute, &'static str>,
    ) {
        assert_eq!(
            <ir::Attribute as TryFrom<_>>::try_from(input)
                .map(test::Attribute::from)
                .map_err(|err| err.to_string()),
            expected.map_err(ToString::to_string),
        )
    }

    #[test]
    fn storage_works() {
        assert_attribute_try_from(
            syn::parse_quote! {
                #[ink(storage)]
            },
            Ok(test::Attribute::Ink(vec![AttributeArgKind::Storage])),
        );
    }

    /// This tests that `#[ink(impl)]` works which can be non-trivial since
    /// `impl` is also a Rust keyword.
    #[test]
    fn impl_works() {
        assert_attribute_try_from(
            syn::parse_quote! {
                #[ink(impl)]
            },
            Ok(test::Attribute::Ink(vec![AttributeArgKind::Implementation])),
        );
    }

    #[test]
    fn selector_works() {
        assert_attribute_try_from(
            syn::parse_quote! {
                #[ink(selector = "0xDEADBEEF")]
            },
            Ok(test::Attribute::Ink(vec![AttributeArgKind::Selector(
                Selector::new([0xDE, 0xAD, 0xBE, 0xEF]),
            )])),
        );
    }

    #[test]
    fn selector_non_hexcode() {
        assert_attribute_try_from(
            syn::parse_quote! {
                #[ink(selector = "0xhelloworld")]
            },
            Err("invalid selector - a selector must consist of four bytes in hex (e.g. `selector = \"0xCAFEBABE\"`)"),
        );
    }

    #[test]
    fn selector_too_long() {
        assert_attribute_try_from(
            syn::parse_quote! {
                #[ink(selector = "0xDEADBEEFC0FEBABE")]
            },
            Err("expected 4-digit hexcode for `selector` argument, found 8 digits"),
        );
    }

    #[test]
    fn selector_invalid_type() {
        assert_attribute_try_from(
            syn::parse_quote! {
                #[ink(selector = 42)]
            },
            Err("expecteded 4-digit hexcode for `selector` argument, e.g. #[ink(selector = 0xC0FEBABE]"),
        );
    }

    #[test]
    fn namespace_works() {
        assert_attribute_try_from(
            syn::parse_quote! {
                #[ink(namespace = "my_namespace")]
            },
            Ok(test::Attribute::Ink(vec![AttributeArgKind::Namespace(
                Namespace::from("my_namespace".to_string().into_bytes()),
            )])),
        );
    }

    #[test]
    fn namespace_invalid_type() {
        assert_attribute_try_from(
            syn::parse_quote! {
                #[ink(namespace = 42)]
            },
            Err("expecteded string type for `namespace` argument, e.g. #[ink(namespace = \"hello\")]"),
        );
    }

    #[test]
    fn compound_mixed_works() {
        assert_attribute_try_from(
            syn::parse_quote! {
                #[ink(message, namespace = "my_namespace")]
            },
            Ok(test::Attribute::Ink(vec![
                AttributeArgKind::Message,
                AttributeArgKind::Namespace(Namespace::from(
                    "my_namespace".to_string().into_bytes(),
                )),
            ])),
        )
    }

    #[test]
    fn compound_simple_works() {
        assert_attribute_try_from(
            syn::parse_quote! {
                #[ink(
                    storage,
                    message,
                    constructor,
                    event,
                    topic,
                    payable,
                    impl,
                )]
            },
            Ok(test::Attribute::Ink(vec![
                AttributeArgKind::Storage,
                AttributeArgKind::Message,
                AttributeArgKind::Constructor,
                AttributeArgKind::Event,
                AttributeArgKind::Topic,
                AttributeArgKind::Payable,
                AttributeArgKind::Implementation,
            ])),
        );
    }

    #[test]
    fn non_ink_attribute_works() {
        let attr: syn::Attribute = syn::parse_quote! {
            #[non_ink(message)]
        };
        assert_attribute_try_from(attr.clone(), Ok(test::Attribute::Other(attr)));
    }

    #[test]
    fn empty_ink_attribute_fails() {
        assert_attribute_try_from(
            syn::parse_quote! {
                #[ink]
            },
            Err("unknown ink! attribute"),
        );
        assert_attribute_try_from(
            syn::parse_quote! {
                #[ink()]
            },
            Err("encountered unsupported empty ink! attribute"),
        );
    }

    #[test]
    fn duplicate_flags_fails() {
        assert_attribute_try_from(
            syn::parse_quote! {
                #[ink(message, message)]
            },
            Err("encountered duplicate ink! attribute arguments"),
        );
    }

    /// Asserts that the given sequence of [`syn::Attribute`] is correctly
    /// partitioned into the expected tuple of ink! and non-ink! attributes
    /// or that the expected error is returned.
    fn assert_parition_attributes(
        input: Vec<syn::Attribute>,
        expected: Result<(Vec<test::InkAttribute>, Vec<syn::Attribute>), &'static str>,
    ) {
        assert_eq!(
            partition_attributes(input)
                .map(|(ink_attr, other_attr)| {
                    (
                        ink_attr
                            .into_iter()
                            .map(test::InkAttribute::from)
                            .collect::<Vec<_>>(),
                        other_attr,
                    )
                })
                .map_err(|err| err.to_string()),
            expected.map_err(ToString::to_string)
        );
    }

    #[test]
    fn parition_attributes_works() {
        assert_parition_attributes(
            vec![
                syn::parse_quote! { #[ink(message)] },
                syn::parse_quote! { #[non_ink_attribute] },
            ],
            Ok((
                vec![test::InkAttribute::from(vec![AttributeArgKind::Message])],
                vec![syn::parse_quote! { #[non_ink_attribute] }],
            )),
        )
    }

    #[test]
    fn parition_duplicates_fails() {
        assert_parition_attributes(
            vec![
                syn::parse_quote! { #[ink(message)] },
                syn::parse_quote! { #[ink(message)] },
            ],
            Err("encountered duplicate ink! attribute"),
        )
    }
}
