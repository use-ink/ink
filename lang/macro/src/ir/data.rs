// Copyright 2018-2019 Parity Technologies (UK) Ltd.
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

//! Contains all ink! IR data structures and definitions.

use derive_more::From;
use proc_macro2::{
    Ident,
    Span,
    TokenStream as TokenStream2,
};
use quote::ToTokens;
use syn::{
    punctuated::Punctuated,
    spanned::Spanned as _,
    Token,
};

/// The contract with all required information.
pub struct Contract {
    /// Outer and inner attributes of the module.
    pub attrs: Vec<syn::Attribute>,
    /// The `mod` token.
    pub mod_token: Token![mod],
    /// The modules snake case identifier.
    pub ident: Ident,
    /// Special ink! meta attributes.
    pub meta_info: MetaInfo,
    /// The state struct.
    pub storage: ItemStorage,
    /// All event structs.
    pub events: Vec<ItemEvent>,
    /// Messages, constructors and methods of the contract.
    pub functions: Vec<Function>,
    /// All non-ink! items defined in the contract module.
    ///
    /// These generally will just be carried and re-generated
    /// and not be restricted by any ink! related analysis.
    pub non_ink_items: Vec<RustItem>,
}

/// The meta information for a contract.
///
/// # Note
///
/// This is generally provided as parameters to the `#[ink::contract(..)]`
/// custom attribute. Mandatory parameters are `types` and `version`.
pub struct MetaInfo {
    /// The environmental types.
    pub env_types: MetaTypes,
    /// The used ink! version.
    pub ink_version: MetaVersion,
    /// If dynamic allocations have been enabled.
    pub dynamic_allocations_enabled: bool,
    /// If contract shall be compiled as dependency.
    pub compile_as_dependency: bool,
}

impl MetaInfo {
    /// Returns `true` if the user enabled dynamic storage allocation.
    #[allow(unused)] // We might need this again in the future! If not, remove.
    pub fn is_dynamic_allocation_enabled(&self) -> bool {
        self.dynamic_allocations_enabled
    }

    /// Returns `true` if the contract is set to compile as dependency.
    pub fn is_compiled_as_dependency(&self) -> bool {
        self.compile_as_dependency
    }
}

/// The specified environmental types.
pub struct MetaTypes {
    /// The specified types.
    pub ty: syn::Type,
}

impl Default for MetaTypes {
    fn default() -> Self {
        Self {
            ty: syn::parse_quote! { ink_core::env::DefaultEnvTypes },
        }
    }
}

/// The major, minor and patch version of the version parameter.
///
/// # Note
///
/// Uses semantic versioning rules.
/// Prerelease and build-metadata information are cut.
#[derive(Debug, Clone)]
pub struct MetaVersion {
    /// The major version.
    pub major: usize,
    /// The minor version.
    pub minor: usize,
    /// The patch version.
    pub patch: usize,
}

/// Either an ink! or a Rust item.
#[derive(From)]
#[allow(clippy::large_enum_variant)] // We should benchmark this somehow.
pub enum Item {
    /// An ink! item.
    Ink(InkItem),
    /// A Rust item.
    Rust(RustItem),
}

/// A simple wrapper around items that are identified to be Rust.
#[derive(From)]
pub struct RustItem {
    /// The inner Rust item.
    pub item: syn::Item,
}

impl ToTokens for RustItem {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        self.item.to_tokens(tokens)
    }
}

/// Intermediate description of a contracts components.
#[derive(From)]
pub enum InkItem {
    /// The ink! storage definition.
    Storage(ItemStorage),
    /// An ink! event definition.
    Event(ItemEvent),
    /// An ink! implementation block.
    Impl(ItemImpl),
}

/// ink! markers use to indicate certain ink! specific properties.
///
/// # Note
///
/// Generally these are the subset of Rust attributes that have `ink` as identifier.
///
/// # Examples
///
/// `#[ink(storage)]` on a `struct` indicates that the `struct` represents the contract's storage.
///
/// ```no_compile
/// #[ink(storage)]
/// struct MyStorage { ... }
/// ```
pub enum Marker {
    /// A simple ink! marker without additional data.
    Simple(SimpleMarker),
}

impl Marker {
    /// Returns the span of `self`.
    pub fn span(&self) -> Span {
        match self {
            Marker::Simple(marker_simple) => marker_simple.span(),
        }
    }

    /// Returns the identifier of `self`.
    ///
    /// # Example
    ///
    /// Every ink! marker has an identifier.
    /// For `#[ink(storage)]` this is `storage`, for `#[ink(event)]` it is `event`.
    pub fn ident(&self) -> &Ident {
        match self {
            Marker::Simple(marker_simple) => &marker_simple.ident,
        }
    }

    /// Returns the ink! attribute if it is simple and the idents match or `None`.
    ///
    /// # Examples
    ///
    /// Simple attributes are for example `#[ink(storage)]` where `storage` is its ident.
    fn filter_simple_by_ident(&self, ident: &str) -> Option<&SimpleMarker> {
        match self {
            Marker::Simple(marker_simple) if marker_simple == ident => {
                Some(marker_simple)
            }
            _ => None,
        }
    }

    /// Returns `true` if the ink! attribute is simple and equal to the given ident.
    ///
    /// # Examples
    ///
    /// Simple attributes are for example `#[ink(storage)]` where `storage` is its ident.
    pub fn is_simple(&self, ident: &str) -> bool {
        self.filter_simple_by_ident(ident).is_some()
    }
}

/// A simple ink! marker that consists of a single identifier.
///
/// # Examples
///
/// - `#[ink(storage)]`
/// - `#[ink(event)]`
pub struct SimpleMarker {
    /// The parentheses around the single identifier.
    pub paren_token: syn::token::Paren,
    /// The single identifier.
    pub ident: Ident,
}

impl SimpleMarker {
    /// Returns the span of `self`.
    pub fn span(&self) -> Span {
        self.paren_token.span
    }
}

impl PartialEq<str> for SimpleMarker {
    fn eq(&self, other: &str) -> bool {
        self.ident == other
    }
}

/// The state struct of the contract.
pub struct ItemStorage {
    /// The `struct` token.
    pub struct_token: Token![struct],
    /// The storage name.
    ///
    /// This normally is equal to the contract's name but camel case.
    pub ident: Ident,
    /// Outer attributes of the storage struct.
    pub attrs: Vec<syn::Attribute>,
    /// Fields of the storage struct.
    pub fields: syn::FieldsNamed,
    /// The original span of the struct definition.
    pub span: Span,
}

impl ItemStorage {
    /// Returns the span of the original `struct` definition.
    pub fn span(&self) -> Span {
        self.span
    }
}

/// An event struct.
pub struct ItemEvent {
    /// The `struct` token.
    pub struct_token: Token![struct],
    /// The name of the event.
    pub ident: Ident,
    /// Outer attributes of the event struct.
    pub attrs: Vec<syn::Attribute>,
    /// Fields of the event struct.
    pub fields: syn::FieldsNamed,
}

impl ItemEvent {
    /// Returns the span of the original `struct` definition.
    pub fn span(&self) -> Span {
        self.struct_token
            .span()
            .join(self.fields.span())
            .expect("spans of `struct_token` and `fields` must be in the same file; qed")
    }
}

/// An implementation block in ink!.
pub struct ItemImpl {
    /// Inner attributes.
    pub attrs: Vec<syn::Attribute>,
    /// The `impl` token.
    pub impl_token: Token![impl],
    /// The implementer type.
    pub self_ty: Ident,
    /// The `{` and `}` tokens.
    pub brace_token: syn::token::Brace,
    /// The functions.
    pub functions: Vec<Function>,
}

/// Represents an ink! message, an ink! constructor or a normal method.
pub struct Function {
    /// The attributes of the function.
    pub attrs: Vec<syn::Attribute>,
    /// The kind of the function.
    pub kind: FunctionKind,
    /// The signature of the function.
    pub sig: Signature,
    /// The statements of the function.
    pub block: syn::Block,
    /// The span of the original function definition.
    pub span: Span,
}

impl Function {
    /// Returns the span from the original function definition.
    pub fn span(&self) -> Span {
        self.span
    }
}

/// The kind of a function.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FunctionKind {
    /// A contract constructor.
    Constructor(KindConstructor),
    /// A contract message.
    Message(KindMessage),
    /// A normal (private) method.
    Method,
}

/// A function that is a contract constructor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KindConstructor {
    /// The function selector.
    pub selector: FunctionSelector,
}

/// A function that is a contract message.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KindMessage {
    /// The function selector.
    pub selector: FunctionSelector,
}

/// A function selector.
///
/// # Note
///
/// This is equal to the first four bytes of the SHA-3 hash of a function's name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FunctionSelector([u8; 4]);

impl FunctionSelector {
    /// Returns the underlying four bytes.
    pub fn as_bytes(&self) -> &[u8; 4] {
        &self.0
    }

    /// Returns a unique identifier as `usize`.
    pub fn unique_id(self) -> usize {
        u32::from_le_bytes(self.0) as usize
    }
}

impl From<&'_ Ident> for FunctionSelector {
    fn from(ident: &Ident) -> Self {
        Self::from(ident.to_string().as_str())
    }
}

impl From<&'_ str> for FunctionSelector {
    fn from(name: &str) -> Self {
        use sha3::digest::Digest as _;
        let mut hasher = sha3::Keccak256::default();
        hasher.input(name.as_bytes());
        let hash = hasher.result();
        Self([hash[0], hash[1], hash[2], hash[3]])
    }
}

impl Function {
    /// Returns the selector of the constructor or message or `None` if it is a method.
    pub fn selector(&self) -> Option<FunctionSelector> {
        match self.kind() {
            FunctionKind::Constructor(constructor) => Some(constructor.selector),
            FunctionKind::Message(message) => Some(message.selector),
            _ => None,
        }
    }

    /// Returns the kind of the function.
    ///
    /// The kind also provides special information associated to the concrete kind, e.g. function selector.
    pub fn kind(&self) -> &FunctionKind {
        &self.kind
    }

    /// Returns the constructor kind if the function is a constructor and otherwise `None`.
    pub fn filter_constructor(&self) -> Option<&KindConstructor> {
        match self.kind() {
            FunctionKind::Constructor(constructor) => Some(constructor),
            _ => None,
        }
    }

    /// Returns the message kind if the function is a message and otherwise `None`.
    pub fn filter_message(&self) -> Option<&KindMessage> {
        match self.kind() {
            FunctionKind::Message(message) => Some(message),
            _ => None,
        }
    }

    /// Returns `true` if the function is a contract constructor.
    pub fn is_constructor(&self) -> bool {
        self.filter_constructor().is_some()
    }

    /// Returns `true` if the function is a contract message.
    pub fn is_message(&self) -> bool {
        self.filter_message().is_some()
    }

    /// Returns `true` if the function is a method.
    #[allow(unused)]
    pub fn is_method(&self) -> bool {
        matches!(self.kind(), FunctionKind::Method)
    }
}

/// The signature of an ink! message, an ink! constructor or a normal method.
pub struct Signature {
    /// The `fn` token.
    pub fn_token: Token![fn],
    /// The identifier.
    pub ident: Ident,
    /// The functions generics.
    ///
    /// Only applicable to methods.
    pub generics: syn::Generics,
    /// The parentheses `(` and `)`.
    pub paren_token: syn::token::Paren,
    /// The function inputs, delimited by `,`.
    ///
    /// Includes the receiver: `&self` or `&mut self`
    pub inputs: Punctuated<FnArg, Token![,]>,
    /// The function output.
    pub output: syn::ReturnType,
}

impl Signature {
    /// Returns `true` if the signature is `&mut self`.
    ///
    /// Returns `None` in case the signature doesn't have a `self` receiver,
    /// e.g. in case for constructor messages.
    pub fn is_mut(&self) -> Option<bool> {
        self.self_arg()
            .map(|receiver| receiver.mutability.is_some())
    }

    /// Returns the `self` input.
    pub fn self_arg(&self) -> Option<&syn::Receiver> {
        if let Some(FnArg::Receiver(receiver)) = self.inputs.first() {
            return Some(&receiver)
        }
        None
    }

    /// Returns an iterator over the function arguments without the receiver.
    pub fn inputs(&self) -> impl Iterator<Item = &IdentType> {
        self.inputs.iter().filter_map(|arg| {
            match arg {
                FnArg::Receiver(_) => None,
                FnArg::Typed(ident_type) => Some(ident_type),
            }
        })
    }

    /// Returns the span of `self`.
    pub fn span(&self) -> Span {
        self.fn_token
            .span()
            .join(self.output.span())
            .expect("spans of `ident` and `ty` must be in the same file; qed")
    }
}

/// A single input of a function.
#[allow(clippy::large_enum_variant)] // We should benchmark this somehow.
pub enum FnArg {
    /// Either `&self` or `&mut self`.
    ///
    /// Note that `syn::Receiver` might also represent `self`.
    Receiver(syn::Receiver),
    /// A captured arguments: e.g. `a: i32`
    Typed(IdentType),
}

impl ToTokens for FnArg {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        match self {
            FnArg::Receiver(receiver) => receiver.to_tokens(tokens),
            FnArg::Typed(ident_type) => ident_type.to_tokens(tokens),
        }
    }
}

/// A captured argument.
///
/// # Examples
///
/// - `a: i32`
/// - `foo: Vec<u8>`
pub struct IdentType {
    /// The attributes of the argument.
    pub attrs: Vec<syn::Attribute>,
    /// The identifier.
    pub ident: Ident,
    /// The `:` token.
    pub colon_token: Token![:],
    /// The type.
    pub ty: syn::Type,
}

impl ToTokens for IdentType {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        for attr in &self.attrs {
            attr.to_tokens(tokens);
        }
        self.ident.to_tokens(tokens);
        self.colon_token.to_tokens(tokens);
        self.ty.to_tokens(tokens);
    }
}
