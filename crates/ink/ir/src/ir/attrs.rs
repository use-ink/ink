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

use core::result::Result;
use std::collections::HashMap;

use ink_prelude::IIP2_WILDCARD_COMPLEMENT_SELECTOR;
use proc_macro2::{
    Span,
    TokenStream as TokenStream2,
};
use quote::ToTokens;
use syn::{
    parse::{
        Parse,
        ParseStream,
    },
    punctuated::Punctuated,
    spanned::Spanned,
    Token,
};

use crate::{
    ast,
    error::ExtError as _,
    ir,
    ir::{
        chain_extension::FunctionId,
        Selector,
    },
};

/// An extension trait for [`syn::Attribute`] in order to query for documentation.
pub trait IsDocAttribute {
    /// Returns `true` if the attribute is a Rust documentation attribute.
    fn is_doc_attribute(&self) -> bool;

    /// Returns the contents of the Rust documentation attribute or `None`.
    fn extract_docs(&self) -> Option<String>;
}

impl IsDocAttribute for syn::Attribute {
    fn is_doc_attribute(&self) -> bool {
        self.path().is_ident("doc")
    }

    fn extract_docs(&self) -> Option<String> {
        if !self.is_doc_attribute() {
            return None;
        }
        match &self.meta {
            syn::Meta::NameValue(nv) => {
                if let syn::Expr::Lit(l) = &nv.value {
                    if let syn::Lit::Str(s) = &l.lit {
                        return Some(s.value());
                    }
                }
            }
            _ => return None,
        }
        None
    }
}

#[allow(clippy::large_enum_variant)] // todo
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
            syn::ImplItem::Fn(item) => &item.attrs,
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
/// #[ink(selector = 0xDEADBEEF)]
/// ```
///
/// An attribute with multiple flags:
/// ```no_compile
/// #[ink(message, payable, selector = 0xDEADBEEF)]
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InkAttribute {
    /// The internal non-empty sequence of arguments of the ink! attribute.
    args: Vec<AttributeFrag>,
}

impl ToTokens for InkAttribute {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        for arg in &self.args {
            arg.to_tokens(tokens)
        }
    }
}

impl InkAttribute {
    /// Ensure that the first ink! attribute argument is of expected kind.
    ///
    /// # Errors
    ///
    /// If the first ink! attribute argument is not of expected kind.
    pub fn ensure_first(&self, expected: &AttributeArgKind) -> Result<(), syn::Error> {
        if &self.first().arg.kind() != expected {
            return Err(format_err!(
                self.span(),
                "unexpected first ink! attribute argument",
            ));
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
        A: IntoIterator<Item = &'a ir::AttributeFrag>,
    {
        use crate::error::ExtError as _;
        use std::collections::HashSet;
        let mut seen: HashSet<&AttributeFrag> = HashSet::new();
        let mut seen2: HashMap<AttributeArgKind, Span> = HashMap::new();
        for arg in args.into_iter() {
            if let Some(seen) = seen.get(arg) {
                return Err(format_err!(
                    arg.span(),
                    "encountered duplicate ink! attribute arguments"
                )
                .into_combine(format_err!(
                    seen.span(),
                    "first equal ink! attribute argument here"
                )));
            }
            if let Some(seen) = seen2.get(&arg.kind().kind()) {
                return Err(format_err!(
                    arg.span(),
                    "encountered ink! attribute arguments with equal kinds"
                )
                .into_combine(format_err!(
                    *seen,
                    "first equal ink! attribute argument with equal kind here"
                )));
            }
            seen.insert(arg);
            seen2.insert(arg.kind().kind(), arg.span());
        }
        Ok(())
    }

    /// Converts a sequence of `#[ink(...)]` attributes into a single flattened
    /// `#[ink(...)]` attribute that contains all of the input arguments.
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
            .flat_map(|attr| attr.args)
            .collect::<Vec<_>>();
        if args.is_empty() {
            return Err(format_err!(
                Span::call_site(),
                "encountered unexpected empty expanded ink! attribute arguments",
            ));
        }
        Self::ensure_no_duplicate_args(&args)?;
        Ok(Self { args })
    }

    /// Returns the first ink! attribute argument.
    pub fn first(&self) -> &AttributeFrag {
        self.args
            .first()
            .expect("encountered invalid empty ink! attribute list")
    }

    /// Returns an iterator over the non-empty flags of the ink! attribute.
    ///
    /// # Note
    ///
    /// This yields at least one ink! attribute flag.
    pub fn args(&self) -> ::core::slice::Iter<'_, AttributeFrag> {
        self.args.iter()
    }

    /// Returns the namespace of the ink! attribute if any.
    pub fn namespace(&self) -> Option<ir::Namespace> {
        self.args().find_map(|arg| {
            if let ir::AttributeArg::Namespace(namespace) = arg.kind() {
                return Some(namespace.clone());
            }
            None
        })
    }

    /// Returns the selector of the ink! attribute if any.
    pub fn selector(&self) -> Option<SelectorOrWildcard> {
        self.args().find_map(|arg| {
            if let ir::AttributeArg::Selector(selector) = arg.kind() {
                return Some(*selector);
            }
            None
        })
    }

    /// Returns the signature topic of the ink! attribute if any.
    pub fn signature_topic_hex(&self) -> Option<String> {
        self.args().find_map(|arg| {
            if let ir::AttributeArg::SignatureTopic(hash) = arg.kind() {
                return Some(hash.clone());
            }
            None
        })
    }

    /// Returns `true` if the ink! attribute contains the `payable` argument.
    pub fn is_payable(&self) -> bool {
        self.args()
            .any(|arg| matches!(arg.kind(), AttributeArg::Payable))
    }

    /// Returns `true` if the ink! attribute contains the `default` argument.
    pub fn is_default(&self) -> bool {
        self.args()
            .any(|arg| matches!(arg.kind(), AttributeArg::Default))
    }

    /// Returns `true` if the ink! attribute contains the wildcard selector.
    pub fn has_wildcard_selector(&self) -> bool {
        self.args().any(|arg| {
            matches!(
                arg.kind(),
                AttributeArg::Selector(SelectorOrWildcard::Wildcard)
            )
        })
    }

    /// Returns `true` if the ink! attribute contains the `anonymous` argument.
    pub fn is_anonymous(&self) -> bool {
        self.args()
            .any(|arg| matches!(arg.kind(), AttributeArg::Anonymous))
    }

    /// Returns `false` if the ink! attribute contains the `handle_status = false`
    /// argument.
    ///
    /// Otherwise returns `true`.
    pub fn is_handle_status(&self) -> bool {
        !self
            .args()
            .any(|arg| matches!(arg.kind(), AttributeArg::HandleStatus(false)))
    }
}

/// An ink! specific attribute argument.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AttributeFrag {
    ast: ast::Meta,
    arg: AttributeArg,
}

impl AttributeFrag {
    /// Returns a shared reference to the attribute argument kind.
    pub fn kind(&self) -> &AttributeArg {
        &self.arg
    }
}

impl ToTokens for AttributeFrag {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        self.ast.to_tokens(tokens)
    }
}

/// The kind of an ink! attribute argument.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AttributeArgKind {
    /// `#[ink(storage)]`
    Storage,
    /// `#[ink(event)]`
    Event,
    /// `#[ink(anonymous)]`
    Anonymous,
    /// `#[ink(message)]`
    Message,
    /// `#[ink(constructor)]`
    Constructor,
    /// `#[ink(payable)]`
    Payable,
    /// `#[ink(default)]`
    Default,
    /// `#[ink(selector = _)]`
    /// `#[ink(selector = 0xDEADBEEF)]`
    Selector,
    /// `#[ink(signature_topic =
    /// "325c98ff66bd0d9d1c10789ae1f2a17bdfb2dcf6aa3d8092669afafdef1cb72d")]`
    SignatureTopicArg,
    /// `#[ink(function = N: u16)]`
    Function,
    /// `#[ink(namespace = "my_namespace")]`
    Namespace,
    /// `#[ink(impl)]`
    Implementation,
    /// `#[ink(handle_status = flag: bool)]`
    HandleStatus,
}

/// An ink! specific attribute flag.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AttributeArg {
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
    /// Applied on ink! constructors or messages in order to indicate
    /// they are default.
    Default,
    /// Can be either one of:
    ///
    /// - `#[ink(selector = 0xDEADBEEF)]` Applied on ink! constructors or messages to
    ///   manually control their selectors.
    /// - `#[ink(selector = _)]` Applied on ink! messages to define a fallback messages
    ///   that is invoked if no other ink! message matches a given selector.
    Selector(SelectorOrWildcard),
    /// `#[ink(signature_topic =
    /// "325c98ff66bd0d9d1c10789ae1f2a17bdfb2dcf6aa3d8092669afafdef1cb72d")]`
    SignatureTopic(String),
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
    /// of them. This is useful if such an implementation block does not contain
    /// any other ink! attributes, so it would be flagged by ink! as a Rust item.
    /// Adding `#[ink(impl)]` on such implementation blocks makes them treated
    /// as ink! implementation blocks thus allowing to access the environment, etc..
    /// Note that ink! messages and constructors still need to be explicitly
    /// flagged as such.
    Implementation,
    /// `#[ink(function = N: u16)]`
    ///
    /// Applies on ink! chain extension method to set their `func_id` parameter.
    /// Every chain extension method must have exactly one ink! `function` attribute.
    ///
    /// Used by the `#[ink::chain_extension]` procedural macro.
    Function(FunctionId),
    /// `#[ink(handle_status = flag: bool)]`
    ///
    /// Used by the `#[ink::chain_extension]` procedural macro.
    ///
    /// Default value: `true`
    HandleStatus(bool),
}

impl core::fmt::Display for AttributeArgKind {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> Result<(), core::fmt::Error> {
        match self {
            Self::Storage => write!(f, "storage"),
            Self::Event => write!(f, "event"),
            Self::Anonymous => write!(f, "anonymous"),
            Self::Message => write!(f, "message"),
            Self::Constructor => write!(f, "constructor"),
            Self::Payable => write!(f, "payable"),
            Self::Selector => {
                write!(f, "selector = S:[u8; 4] || _")
            }
            Self::SignatureTopicArg => {
                write!(f, "signature_topic = S:[u8; 32]")
            }
            Self::Function => {
                write!(f, "function = N:u16)")
            }
            Self::Namespace => {
                write!(f, "namespace = N:string")
            }
            Self::Implementation => write!(f, "impl"),
            Self::HandleStatus => write!(f, "handle_status"),
            Self::Default => write!(f, "default"),
        }
    }
}

impl AttributeArg {
    /// Returns the kind of the ink! attribute argument.
    pub fn kind(&self) -> AttributeArgKind {
        match self {
            Self::Storage => AttributeArgKind::Storage,
            Self::Event => AttributeArgKind::Event,
            Self::Anonymous => AttributeArgKind::Anonymous,
            Self::Message => AttributeArgKind::Message,
            Self::Constructor => AttributeArgKind::Constructor,
            Self::Payable => AttributeArgKind::Payable,
            Self::Selector(_) => AttributeArgKind::Selector,
            Self::SignatureTopic(_) => AttributeArgKind::SignatureTopicArg,
            Self::Function(_) => AttributeArgKind::Function,
            Self::Namespace(_) => AttributeArgKind::Namespace,
            Self::Implementation => AttributeArgKind::Implementation,
            Self::HandleStatus(_) => AttributeArgKind::HandleStatus,
            Self::Default => AttributeArgKind::Default,
        }
    }
}

impl core::fmt::Display for AttributeArg {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> Result<(), core::fmt::Error> {
        match self {
            Self::Storage => write!(f, "storage"),
            Self::Event => write!(f, "event"),
            Self::Anonymous => write!(f, "anonymous"),
            Self::Message => write!(f, "message"),
            Self::Constructor => write!(f, "constructor"),
            Self::Payable => write!(f, "payable"),
            Self::Selector(selector) => core::fmt::Display::fmt(&selector, f),
            Self::SignatureTopic(hash) => {
                write!(f, "signature_topic = {hash:?}")
            }
            Self::Function(function) => {
                write!(f, "function = {:?}", function.into_u16())
            }
            Self::Namespace(namespace) => {
                write!(f, "namespace = {:?}", namespace.as_bytes())
            }
            Self::Implementation => write!(f, "impl"),
            Self::HandleStatus(value) => write!(f, "handle_status = {value:?}"),
            Self::Default => write!(f, "default"),
        }
    }
}

/// Either a wildcard selector or a specified selector.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SelectorOrWildcard {
    /// A wildcard selector. If no other selector matches, the message/constructor
    /// annotated with the wildcard selector will be invoked.
    Wildcard,
    /// A user provided selector.
    UserProvided(Selector),
}

impl SelectorOrWildcard {
    /// Create a new `SelectorOrWildcard::Selector` from the supplied bytes.
    fn selector(bytes: [u8; 4]) -> Self {
        SelectorOrWildcard::UserProvided(Selector::from(bytes))
    }

    /// The selector of the wildcard complement message.
    pub fn wildcard_complement() -> Self {
        Self::selector(IIP2_WILDCARD_COMPLEMENT_SELECTOR)
    }
}

impl TryFrom<&ast::MetaValue> for SelectorOrWildcard {
    type Error = syn::Error;

    fn try_from(value: &ast::MetaValue) -> Result<Self, Self::Error> {
        match value {
            ast::MetaValue::Lit(lit) => {
                if let syn::Lit::Str(_) = lit {
                    return Err(format_err_spanned!(
                        lit,
                        "#[ink(selector = ..)] attributes with string inputs are deprecated. \
                        use an integer instead, e.g. #[ink(selector = 1)] or #[ink(selector = 0xC0DECAFE)]."
                    ));
                }
                if let syn::Lit::Int(lit_int) = lit {
                    let selector_u32 = lit_int.base10_parse::<u32>()
                        .map_err(|error| {
                            format_err_spanned!(
                                lit_int,
                                "selector value out of range. selector must be a valid `u32` integer: {}",
                                error
                            )
                        })?;
                    let selector = Selector::from(selector_u32.to_be_bytes());
                    return Ok(SelectorOrWildcard::UserProvided(selector))
                }
                Err(format_err_spanned!(
                    value,
                    "expected 4-digit hexcode for `selector` argument, e.g. #[ink(selector = 0xC0FEBABE]"
                ))
            }
            ast::MetaValue::Symbol(symbol) => {
                match symbol {
                    ast::Symbol::Underscore(_) => Ok(SelectorOrWildcard::Wildcard),
                    ast::Symbol::AtSign(_) => Ok(SelectorOrWildcard::wildcard_complement()),
                }
            }
            ast::MetaValue::Path(path) => {
                Err(format_err_spanned!(
                    path,
                    "unexpected path for `selector` argument, expected a 4-digit hexcode or one of \
                    the wildcard symbols: `_` or `@`"
                ))
            }
        }
    }
}

impl core::fmt::Display for SelectorOrWildcard {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> Result<(), core::fmt::Error> {
        match self {
            Self::UserProvided(selector) => core::fmt::Debug::fmt(&selector, f),
            Self::Wildcard => write!(f, "_"),
        }
    }
}

/// An ink! namespace applicable to a trait implementation block.
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Namespace {
    /// The underlying bytes.
    bytes: Vec<u8>,
}

impl TryFrom<&ast::MetaValue> for Namespace {
    type Error = syn::Error;

    fn try_from(value: &ast::MetaValue) -> Result<Self, Self::Error> {
        if let ast::MetaValue::Lit(syn::Lit::Str(lit_str)) = value {
            let argument = lit_str.value();
            syn::parse_str::<syn::Ident>(&argument).map_err(|_error| {
                format_err_spanned!(
                    lit_str,
                    "encountered invalid Rust identifier for namespace argument",
                )
            })?;
            Ok(Namespace::from(argument.into_bytes()))
        } else {
            Err(format_err_spanned!(
                value,
                "expected string type for `namespace` argument, e.g. #[ink(namespace = \"hello\")]",
            ))
        }
    }
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
/// `#[ink(...)]` or `#[ink]`.
///
/// # Note
///
/// This does not check at this point whether the ink! attribute is valid since
/// this check is optimized for efficiency.
pub fn contains_ink_attributes<'a, I>(attrs: I) -> bool
where
    I: IntoIterator<Item = &'a syn::Attribute>,
{
    attrs.into_iter().any(|attr| attr.path().is_ident("ink"))
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
    let first = attrs.into_iter().find(|attr| attr.path().is_ident("ink"));
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
/// # Parameters
///
/// The `is_conflicting_attr` closure returns `Ok` if the attribute does not conflict,
/// returns `Err(None)` if the attribute conflicts but without providing further reasoning
/// and `Err(Some(reason))` if the attribute conflicts given additional context
/// information.
///
/// # Errors
///
/// - If there are invalid ink! attributes.
/// - If there are duplicate ink! attributes.
/// - If the first ink! attribute is not matching the expected.
/// - If there are conflicting ink! attributes.
/// - if there are no ink! attributes.
pub fn sanitize_attributes<I, C>(
    parent_span: Span,
    attrs: I,
    is_valid_first: &ir::AttributeArgKind,
    is_conflicting_attr: C,
) -> Result<(InkAttribute, Vec<syn::Attribute>), syn::Error>
where
    I: IntoIterator<Item = syn::Attribute>,
    C: FnMut(&ir::AttributeFrag) -> Result<(), Option<syn::Error>>,
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
    normalized.ensure_no_conflicts(is_conflicting_attr)?;
    Ok((normalized, other_attrs))
}

/// Sanitizes the given optional attributes.
///
/// This partitions the attributes into ink! and non-ink! attributes.
/// If there are ink! attributes they are normalized and deduplicated.
/// Also checks to guard against conflicting ink! attributes are provided.
///
/// Returns the optional partitioned ink! and non-ink! attributes.
///
/// # Parameters
///
/// The `is_conflicting_attr` closure returns `Ok` if the attribute does not conflict,
/// returns `Err(None)` if the attribute conflicts but without providing further reasoning
/// and `Err(Some(reason))` if the attribute conflicts given additional context
/// information.
///
/// # Errors
///
/// - If there are invalid ink! attributes.
/// - If there are duplicate ink! attributes.
/// - If there are conflicting ink! attributes.
pub fn sanitize_optional_attributes<I, C>(
    parent_span: Span,
    attrs: I,
    is_conflicting_attr: C,
) -> Result<(Option<InkAttribute>, Vec<syn::Attribute>), syn::Error>
where
    I: IntoIterator<Item = syn::Attribute>,
    C: FnMut(&ir::AttributeFrag) -> Result<(), Option<syn::Error>>,
{
    let (ink_attrs, rust_attrs) = ir::partition_attributes(attrs)?;
    if ink_attrs.is_empty() {
        return Ok((None, rust_attrs));
    }
    let normalized = ir::InkAttribute::from_expanded(ink_attrs).map_err(|err| {
        err.into_combine(format_err!(parent_span, "at this invocation",))
    })?;
    normalized.ensure_no_conflicts(is_conflicting_attr)?;
    Ok((Some(normalized), rust_attrs))
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
                .into_combine(format_err!(seen.span(), "first ink! attribute here")));
            }
            seen.insert(attr);
        }
        Ok(())
    }
}

impl TryFrom<syn::Attribute> for Attribute {
    type Error = syn::Error;

    fn try_from(attr: syn::Attribute) -> Result<Self, Self::Error> {
        if attr.path().is_ident("ink") {
            return <InkAttribute as TryFrom<_>>::try_from(attr).map(Into::into);
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
        if !attr.path().is_ident("ink") {
            return Err(format_err_spanned!(attr, "unexpected non-ink! attribute"));
        }

        let args: Vec<_> = attr
            .parse_args_with(Punctuated::<AttributeFrag, Token![,]>::parse_terminated)?
            .into_iter()
            .collect();

        Self::ensure_no_duplicate_args(&args)?;
        if args.is_empty() {
            return Err(format_err_spanned!(
                attr,
                "encountered unsupported empty ink! attribute"
            ));
        }
        Ok(InkAttribute { args })
    }
}

impl InkAttribute {
    /// Ensures that there are no conflicting ink! attribute arguments in `self`.
    ///
    /// The given `is_conflicting` describes for every ink! attribute argument
    /// found in `self` if it is in conflict.
    ///
    /// # Parameters
    ///
    /// The `is_conflicting_attr` closure returns `Ok` if the attribute does not conflict,
    /// returns `Err(None)` if the attribute conflicts but without providing further
    /// reasoning and `Err(Some(reason))` if the attribute conflicts given additional
    /// context information.
    pub fn ensure_no_conflicts<'a, P>(
        &'a self,
        mut is_conflicting: P,
    ) -> Result<(), syn::Error>
    where
        P: FnMut(&'a ir::AttributeFrag) -> Result<(), Option<syn::Error>>,
    {
        let mut err: Option<syn::Error> = None;
        for arg in self.args() {
            if let Err(reason) = is_conflicting(arg) {
                let conflict_err = format_err!(
                    arg.span(),
                    "encountered conflicting ink! attribute argument",
                );
                match &mut err {
                    Some(err) => {
                        err.combine(conflict_err);
                    }
                    None => {
                        err = Some(conflict_err);
                    }
                }
                if let Some(reason) = reason {
                    err.as_mut()
                        .expect("must be `Some` at this point")
                        .combine(reason);
                }
            }
        }
        if let Some(err) = err {
            return Err(err);
        }
        Ok(())
    }
}

impl Parse for AttributeFrag {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ast: ast::Meta = input.parse()?;

        let arg = match &ast {
            ast::Meta::NameValue(name_value) => {
                let ident = name_value.name.get_ident().ok_or_else(|| {
                    format_err_spanned!(
                        name_value.name,
                        "expected identifier for ink! attribute argument",
                    )
                })?;
                match ident.to_string().as_str() {
                    "selector" => {
                        SelectorOrWildcard::try_from(&name_value.value)
                            .map(AttributeArg::Selector)
                    }
                    "namespace" => {
                        Namespace::try_from(&name_value.value)
                            .map(AttributeArg::Namespace)
                    }
                    "signature_topic" => {
                        if let Some(hash) = name_value.value.as_string() {
                            Ok(AttributeArg::SignatureTopic(hash))
                        } else {
                            Err(format_err_spanned!(
                                name_value.value,
                                "expected String type for `S` in #[ink(signature_topic = S)]",
                            ))
                        }
                    }
                    "function" => {
                        if let Some(lit_int) = name_value.value.as_lit_int() {
                            let id = lit_int.base10_parse::<u16>()
                                .map_err(|error| {
                                    format_err_spanned!(
                                        lit_int,
                                        "could not parse `N` in `#[ink(function = N)]` into a `u16` integer: {}", error)
                                })?;
                            Ok(AttributeArg::Function(FunctionId::from_u16(id)))
                        } else {
                            Err(format_err_spanned!(
                                name_value.value,
                                "expected `u16` integer type for `N` in #[ink(function = N)]",
                            ))
                        }
                    }
                    "handle_status" => {
                        if let Some(value) = name_value.value.as_bool() {
                            Ok(AttributeArg::HandleStatus(value))
                        } else {
                            Err(format_err_spanned!(
                                name_value.value,
                                "expected `bool` value type for `flag` in #[ink(handle_status = flag)]",
                            ))
                        }
                    }
                    _ => {
                        Err(format_err_spanned!(
                            ident,
                            "encountered unknown ink! attribute argument: {}",
                            ident
                        ))
                    }
                }
            }
            ast::Meta::Path(path) => {
                let ident = path.get_ident().ok_or_else(|| {
                    format_err_spanned!(
                        path,
                        "expected identifier for ink! attribute argument",
                    )
                })?;
                match ident.to_string().as_str() {
                    "storage" => Ok(AttributeArg::Storage),
                    "message" => Ok(AttributeArg::Message),
                    "constructor" => Ok(AttributeArg::Constructor),
                    "event" => Ok(AttributeArg::Event),
                    "anonymous" => Ok(AttributeArg::Anonymous),
                    "payable" => Ok(AttributeArg::Payable),
                    "default" => Ok(AttributeArg::Default),
                    "impl" => Ok(AttributeArg::Implementation),
                    _ => match ident.to_string().as_str() {
                        "function" => Err(format_err_spanned!(
                            path,
                            "encountered #[ink(function)] that is missing its `id` parameter. \
                            Did you mean #[ink(function = id: u16)] ?"
                        )),
                        "handle_status" => Err(format_err_spanned!(
                            path,
                           "encountered #[ink(handle_status)] that is missing its `flag: bool` parameter. \
                            Did you mean #[ink(handle_status = flag: bool)] ?"
                        )),
                        "namespace" => Err(format_err_spanned!(
                            path,
                           "encountered #[ink(namespace)] that is missing its string parameter. \
                            Did you mean #[ink(namespace = name: str)] ?"
                        )),
                        "selector" => Err(format_err_spanned!(
                            path,
                           "encountered #[ink(selector)] that is missing its u32 parameter. \
                            Did you mean #[ink(selector = value: u32)] ?"
                        )),
                        _ => Err(format_err_spanned!(
                            path,
                            "encountered unknown ink! attribute argument: {}",
                            ident
                        )),
                    },
                }
            }
        }?;

        Ok(Self { ast, arg })
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
        expected: Result<Option<Vec<ir::AttributeArg>>, &'static str>,
    ) {
        assert_eq!(
            first_ink_attribute(input)
                .map(|maybe_attr: Option<ir::InkAttribute>| {
                    maybe_attr.map(|attr: ir::InkAttribute| {
                        attr.args.into_iter().map(|arg| arg.arg).collect::<Vec<_>>()
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
            Ok(Some(vec![AttributeArg::Storage])),
        );
        assert_first_ink_attribute(
            &[syn::parse_quote! { #[ink(invalid)] }],
            Err("encountered unknown ink! attribute argument: invalid"),
        );
    }

    mod test {
        use crate::ir;

        /// Mock for `ir::Attribute` to improve the ability to test.
        #[derive(Debug, PartialEq, Eq)]
        #[allow(clippy::large_enum_variant)] // todo
        pub enum Attribute {
            Ink(Vec<ir::AttributeArg>),
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
                                .map(|arg| arg.arg)
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

        /// Mock for `ir::InkAttribute` to improve the ability to test.
        #[derive(Debug, PartialEq, Eq)]
        pub struct InkAttribute {
            args: Vec<ir::AttributeArg>,
        }

        impl From<ir::InkAttribute> for InkAttribute {
            fn from(ink_attr: ir::InkAttribute) -> Self {
                Self {
                    args: ink_attr
                        .args
                        .into_iter()
                        .map(|arg| arg.arg)
                        .collect::<Vec<_>>(),
                }
            }
        }

        impl<I> From<I> for InkAttribute
        where
            I: IntoIterator<Item = ir::AttributeArg>,
        {
            fn from(args: I) -> Self {
                Self {
                    args: args.into_iter().collect::<Vec<_>>(),
                }
            }
        }
    }

    /// Asserts that the given [`syn::Attribute`] is converted into the expected
    /// [`ir::Attribute`] or yields the expected error message.
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
            Ok(test::Attribute::Ink(vec![AttributeArg::Storage])),
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
            Ok(test::Attribute::Ink(vec![AttributeArg::Implementation])),
        );
    }

    #[test]
    fn selector_works() {
        assert_attribute_try_from(
            syn::parse_quote! {
                #[ink(selector = 42)]
            },
            Ok(test::Attribute::Ink(vec![AttributeArg::Selector(
                SelectorOrWildcard::UserProvided(Selector::from([0, 0, 0, 42])),
            )])),
        );
        assert_attribute_try_from(
            syn::parse_quote! {
                #[ink(selector = 0xDEADBEEF)]
            },
            Ok(test::Attribute::Ink(vec![AttributeArg::Selector(
                SelectorOrWildcard::selector([0xDE, 0xAD, 0xBE, 0xEF]),
            )])),
        );
    }

    #[test]
    fn wildcard_selector_works() {
        assert_attribute_try_from(
            syn::parse_quote! {
                #[ink(selector = _)]
            },
            Ok(test::Attribute::Ink(vec![AttributeArg::Selector(
                SelectorOrWildcard::Wildcard,
            )])),
        );
    }

    #[test]
    fn selector_negative_number() {
        assert_attribute_try_from(
            syn::parse_quote! {
                #[ink(selector = -1)]
            },
            Err(
                "selector value out of range. selector must be a valid `u32` integer: \
                invalid digit found in string",
            ),
        );
    }

    #[test]
    fn selector_out_of_range() {
        assert_attribute_try_from(
            syn::parse_quote! {
                #[ink(selector = 0xFFFF_FFFF_FFFF_FFFF)]
            },
            Err(
                "selector value out of range. \
                selector must be a valid `u32` integer: number too large to fit in target type"
            ),
        );
    }

    #[test]
    fn selector_invalid_type() {
        assert_attribute_try_from(
            syn::parse_quote! {
                #[ink(selector = true)]
            },
            Err("expected 4-digit hexcode for `selector` argument, e.g. #[ink(selector = 0xC0FEBABE]"),
        );
    }

    #[test]
    fn default_works() {
        assert_attribute_try_from(
            syn::parse_quote! {
                #[ink(default)]
            },
            Ok(test::Attribute::Ink(vec![AttributeArg::Default])),
        )
    }

    #[test]
    fn namespace_works() {
        assert_attribute_try_from(
            syn::parse_quote! {
                #[ink(namespace = "my_namespace")]
            },
            Ok(test::Attribute::Ink(vec![AttributeArg::Namespace(
                Namespace::from("my_namespace".to_string().into_bytes()),
            )])),
        );
    }

    #[test]
    fn namespace_invalid_identifier() {
        assert_attribute_try_from(
            syn::parse_quote! {
                #[ink(namespace = "::invalid_identifier")]
            },
            Err("encountered invalid Rust identifier for namespace argument"),
        );
    }

    #[test]
    fn namespace_invalid_type() {
        assert_attribute_try_from(
            syn::parse_quote! {
                #[ink(namespace = 42)]
            },
            Err("expected string type for `namespace` argument, e.g. #[ink(namespace = \"hello\")]"),
        );
    }

    #[test]
    fn namespace_missing_parameter() {
        assert_attribute_try_from(
            syn::parse_quote! {
                #[ink(namespace)]
            },
            Err(
                "encountered #[ink(namespace)] that is missing its string parameter. \
                Did you mean #[ink(namespace = name: str)] ?",
            ),
        );
    }

    #[test]
    fn extension_works() {
        assert_attribute_try_from(
            syn::parse_quote! {
                #[ink(function = 42)]
            },
            Ok(test::Attribute::Ink(vec![AttributeArg::Function(
                FunctionId::from_u16(42),
            )])),
        );
    }

    #[test]
    fn extension_invalid_value_type() {
        assert_attribute_try_from(
            syn::parse_quote! {
                #[ink(function = "string")]
            },
            Err("expected `u16` integer type for `N` in #[ink(function = N)]"),
        );
    }

    #[test]
    fn extension_negative_integer() {
        assert_attribute_try_from(
            syn::parse_quote! {
                #[ink(function = -1)]
            },
            Err("could not parse `N` in `#[ink(function = N)]` into a `u16` integer: invalid digit found in string")
        );
    }

    #[test]
    fn extension_too_big_integer() {
        let max_u32_plus_1 = (u32::MAX as u64) + 1;
        assert_attribute_try_from(
            syn::parse_quote! {
                #[ink(function = #max_u32_plus_1)]
            },
            Err("could not parse `N` in `#[ink(function = N)]` into a `u16` integer: number too large to fit in target type"),
        );
    }

    #[test]
    fn extension_missing_parameter() {
        assert_attribute_try_from(
            syn::parse_quote! {
                #[ink(function)]
            },
            Err(
                "encountered #[ink(function)] that is missing its `id` parameter. \
                Did you mean #[ink(function = id: u16)] ?",
            ),
        );
    }

    #[test]
    fn handle_status_works() {
        fn expected_ok(value: bool) -> Result<test::Attribute, &'static str> {
            Ok(test::Attribute::Ink(vec![AttributeArg::HandleStatus(
                value,
            )]))
        }
        assert_attribute_try_from(
            syn::parse_quote! {
                #[ink(handle_status = true)]
            },
            expected_ok(true),
        );
        assert_attribute_try_from(
            syn::parse_quote! {
                #[ink(handle_status = false)]
            },
            expected_ok(false),
        );
    }

    #[test]
    fn handle_status_missing_parameter() {
        assert_attribute_try_from(
            syn::parse_quote! {
                #[ink(handle_status)]
            },
            Err(
                "encountered #[ink(handle_status)] that is missing its `flag: bool` parameter. \
                Did you mean #[ink(handle_status = flag: bool)] ?",
            ),
        );
    }

    #[test]
    fn handle_status_invalid_parameter_type() {
        assert_attribute_try_from(
            syn::parse_quote! {
                #[ink(handle_status = "string")]
            },
            Err("expected `bool` value type for `flag` in #[ink(handle_status = flag)]"),
        );
    }

    #[test]
    fn compound_mixed_works() {
        assert_attribute_try_from(
            syn::parse_quote! {
                #[ink(message, namespace = "my_namespace")]
            },
            Ok(test::Attribute::Ink(vec![
                AttributeArg::Message,
                AttributeArg::Namespace(Namespace::from(
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
                    payable,
                    impl,
                )]
            },
            Ok(test::Attribute::Ink(vec![
                AttributeArg::Storage,
                AttributeArg::Message,
                AttributeArg::Constructor,
                AttributeArg::Event,
                AttributeArg::Payable,
                AttributeArg::Implementation,
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
            Err("expected attribute arguments in parentheses: #[ink(...)]"),
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
                vec![test::InkAttribute::from(vec![AttributeArg::Message])],
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
    #[test]
    fn signature_topic_works() {
        let s = "11".repeat(32);
        assert_attribute_try_from(
            syn::parse_quote! {
                #[ink(signature_topic = #s)]
            },
            Ok(test::Attribute::Ink(vec![AttributeArg::SignatureTopic(s)])),
        );
    }
}
