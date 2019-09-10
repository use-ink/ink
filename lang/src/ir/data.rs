// Copyright 2018-2019 Parity Technologies (UK) Ltd.
// This file is part of ink!.
//
// ink! is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// ink! is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with ink!.  If not, see <http://www.gnu.org/licenses/>.

use derive_more::From;
use proc_macro2::{
    Ident,
    Span,
    TokenStream as TokenStream2,
};
use quote::quote;
use syn::{
    punctuated::Punctuated,
    spanned::Spanned as _,
    Token,
};

/// The contract with all required information.
pub struct Contract {
    /// The `mod` token.
    pub mod_token: Token![mod],
    /// The modules snake case identifier.
    pub ident: Ident,
    /// Special ink! meta attributes.
    pub meta_info: MetaInfo,
    /// Outer and inner attributes of the module.
    ///
    /// This also containes the environmental types definition
    /// as well as the ink! version.
    pub attrs: Vec<syn::Attribute>,
    /// The state struct.
    pub storage: ItemStorage,
    /// All event structs.
    pub events: Vec<ItemEvent>,
    /// Messages, constructors and methods of the contract.
    pub functions: Vec<Function>,
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
}

/// The specified environmental types.
pub struct MetaTypes {
    /// The specified types.
    pub ty: syn::Type,
}

/// The specified version of a contract.
pub struct MetaVersion {
    /// The major version, e.g. the `X` in `[X, Y, Z]`.
    pub major: u32,
    /// The minor version, e.g. the `Y` in `[X, Y, Z]`.
    pub minor: u32,
    /// The patch version, e.g. the `Z` in `[X, Y, Z]`.
    pub patch: u32,
}

/// Types implementing this trait are code generators for the ink! language.
pub trait GenerateCode {
    /// Generates ink! contract code.
    fn generate_code(&self) -> TokenStream2;
}

impl GenerateCode for Contract {
    /// Generates ink! contract code.
    fn generate_code(&self) -> TokenStream2 {
        quote! {}
    }
}

/// Intermediate description of a contracts components.
#[derive(From)]
pub enum Item {
    /// A storage definition.
    Storage(ItemStorage),
    /// An event definition.
    Event(ItemEvent),
    /// An implementation block.
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

    /// Returns the simple ink! attribute identifier and `None` if `self` is not simple.
    pub fn get_simple(&self) -> Option<&Ident> {
        match self {
            Marker::Simple(marker_simple) => Some(&marker_simple.ident),
        }
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
pub struct FunctionSelector(u32);

impl FunctionSelector {
    /// Returns the `u32` representation of the function selector.
    pub fn to_u32(&self) -> u32 {
        self.0
    }
}

impl From<&'_ Ident> for FunctionSelector {
    fn from(ident: &Ident) -> Self {
        Self::from(ident.to_string().as_str())
    }
}

impl From<&'_ str> for FunctionSelector {
    fn from(name: &str) -> Self {
        let sha3_hash = ink_utils::hash::keccak256(name.as_bytes());
        Self(u32::from_le_bytes([
            sha3_hash[0],
            sha3_hash[1],
            sha3_hash[2],
            sha3_hash[3],
        ]))
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

    /// Returns `true` if the function is a contract constructor.
    pub fn is_constructor(&self) -> bool {
        match self.kind() {
            FunctionKind::Constructor(_) => true,
            _ => false,
        }
    }

    /// Returns `true` if the function is a contract message.
    pub fn is_message(&self) -> bool {
        match self.kind() {
            FunctionKind::Message(_) => true,
            _ => false,
        }
    }

    /// Returns `true` if the function is a method.
    pub fn is_method(&self) -> bool {
        match self.kind() {
            FunctionKind::Method => true,
            _ => false,
        }
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
    pub inputs: Punctuated<FnArg, Token![,]>,
    /// The function output.
    pub output: syn::ReturnType,
}

impl Signature {
    /// Returns `true` if the signature is `&mut self`.
    pub fn is_mut(&self) -> bool {
        self.self_arg().mutability.is_some()
    }

    /// Returns the `self` input.
    pub fn self_arg(&self) -> &syn::Receiver {
        if let FnArg::Receiver(receiver) = &self.inputs[0] {
            return &receiver
        } else {
            unreachable!("must contain the receiver in the first argument position")
        }
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
pub enum FnArg {
    /// Either `&self` or `&mut self`.
    ///
    /// Note that `syn::Receiver` might also represent `self`.
    Receiver(syn::Receiver),
    /// A captured arguments: e.g. `a: i32`
    Typed(IdentType),
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

impl IdentType {
    /// Returns the span of `self`.
    pub fn span(&self) -> Span {
        self.ident
            .span()
            .join(self.ty.span())
            .expect("spans of `ident` and `ty` must be in the same file; qed")
    }
}
