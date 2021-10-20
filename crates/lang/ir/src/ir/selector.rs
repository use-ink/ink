// Copyright 2018-2021 Parity Technologies (UK) Ltd.
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

use super::blake2::blake2b_256;
use crate::literal::HexLiteral;
use core::convert::TryFrom;
use proc_macro2::TokenStream as TokenStream2;
use std::marker::PhantomData;
use syn::spanned::Spanned as _;

/// The selector of an ink! dispatchable.
///
/// # Note
///
/// This is equal to the first four bytes of the SHA-3 hash of a function's name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Selector {
    bytes: [u8; 4],
}

/// The trait prefix to compute a composed selector for trait implementation blocks.
#[derive(Debug, Copy, Clone)]
pub struct TraitPrefix<'a> {
    /// The namespace of the ink! trait definition.
    ///
    /// By default this is equal to the `module_path!` at the ink! trait definition site.
    /// It can be customized by the ink! trait definition author using `#[ink(namespace = N)]`
    /// ink! attribute.
    namespace: Option<&'a syn::LitStr>,
    /// The Rust identifier of the ink! trait definition.
    trait_ident: &'a syn::Ident,
}

impl<'a> TraitPrefix<'a> {
    /// Creates a new trait prefix.
    pub fn new(trait_ident: &'a syn::Ident, namespace: Option<&'a syn::LitStr>) -> Self {
        Self {
            namespace,
            trait_ident,
        }
    }

    /// Returns a vector over the bytes of the namespace.
    pub fn namespace_bytes(&self) -> Vec<u8> {
        self.namespace
            .map(|namespace| namespace.value().into_bytes())
            .unwrap_or_default()
    }

    /// Returns a shared reference to the Rust identifier of the trait.
    pub fn trait_ident(&self) -> &'a syn::Ident {
        self.trait_ident
    }
}

impl Selector {
    /// Computes the BLAKE-2 256-bit based selector from the given input bytes.
    pub fn compute(input: &[u8]) -> Self {
        let mut output = [0; 32];
        blake2b_256(input, &mut output);
        Self::from([output[0], output[1], output[2], output[3]])
    }

    /// # Note
    ///
    /// - `trait_prefix` is `None` when computing the selector of ink! constructors
    ///   and messages in inherent implementation blocks.
    /// - `trait_prefix` is `Some` when computing the selector of ink! constructors
    ///   and messages in trait implementation blocks. In this case the `namespace`
    ///   is either the full path of the trait definition gained by Rust's
    ///   `module_path!` macro by default or it is customized by manual application
    ///   of the `#[ink(namespace = "my_namespace")]` ink! attribute. In the
    ///   example `my_namespace` concatenated with `::` and the identifier of the
    ///   trait definition would then be part of the provided `trait_prefix` parameter.
    /// - `fn_ident` refers to the ink! constructor or message identifier.
    ///
    /// # Inherent Implementation Blocks
    ///
    /// For inherent implementation blocks, when `trait_prefix` is `None` the composed
    /// selector is computed as follows:
    ///
    /// 1. Apply `BLAKE2` 256-bit hash `H` on the bytes of the ASCII representation of
    ///   the `fn_ident` identifier.
    /// 1. The first 4 bytes of `H` make up the selector.
    ///
    /// # Trait Implementation Blocks
    ///
    /// For trait implementation blocks, when `trait_prefix` is
    /// `Some((namespace, trait_ident))` the composed selector is computed as follows:
    ///
    /// 1. Compute the ASCII byte representation of `fn_ident` and call it `F`.
    /// 1. Compute the ASCII byte representation of `namespace` and call it `N`.
    /// 1. Compute the ASCII byte representation of `trait_ident` and call it `T`.
    /// 1. Concatenate `N`, `T` and `F` using `::` as separator and call it `C`.
    /// 1. Apply the `BLAKE2` 256-bit hash `H` of `C`.
    /// 1. The first 4 bytes of `H` make up the selector.
    pub fn compose<'a, T>(trait_prefix: T, fn_ident: &syn::Ident) -> Self
    where
        T: Into<Option<TraitPrefix<'a>>>,
    {
        let fn_ident = fn_ident.to_string().into_bytes();
        let input_bytes: Vec<u8> = match trait_prefix.into() {
            Some(trait_prefix) => {
                let namespace = trait_prefix.namespace_bytes();
                let trait_ident = trait_prefix.trait_ident().to_string().into_bytes();
                let separator = &b"::"[..];
                if namespace.is_empty() {
                    [&trait_ident[..], &fn_ident[..]].join(separator)
                } else {
                    [&namespace[..], &trait_ident[..], &fn_ident[..]].join(separator)
                }
            }
            None => fn_ident.to_vec(),
        };
        Self::compute(&input_bytes)
    }

    /// Returns the underlying four bytes.
    pub fn to_bytes(&self) -> [u8; 4] {
        self.bytes
    }

    /// Returns the big-endian `u32` representation of the selector bytes.
    pub fn into_be_u32(self) -> u32 {
        u32::from_be_bytes(self.bytes)
    }

    /// Returns the 4 bytes that make up the selector as hex encoded bytes.
    pub fn hex_lits(self) -> [syn::LitInt; 4] {
        self.bytes.map(<u8 as HexLiteral>::hex_padded_suffixed)
    }
}

impl From<[u8; 4]> for Selector {
    fn from(bytes: [u8; 4]) -> Self {
        Self { bytes }
    }
}

/// Used as generic parameter for the `selector_id!` macro.
pub enum SelectorId {}

/// Used as generic parameter for the `selector_bytes!` macro.
pub enum SelectorBytes {}

/// The selector ID of an ink! dispatchable.
///
/// # Note
///
/// This is mainly used for analysis and codegen of the `selector_id!` macro.
#[derive(Debug)]
pub struct SelectorMacro<T> {
    selector: Selector,
    input: syn::Lit,
    _marker: PhantomData<fn() -> T>,
}

impl<T> SelectorMacro<T> {
    /// Returns the underlying selector.
    pub fn selector(&self) -> Selector {
        self.selector
    }

    /// Returns the literal input of the selector ID.
    pub fn input(&self) -> &syn::Lit {
        &self.input
    }
}

impl<T> TryFrom<TokenStream2> for SelectorMacro<T> {
    type Error = syn::Error;

    fn try_from(input: TokenStream2) -> Result<Self, Self::Error> {
        let input_span = input.span();
        let lit = syn::parse2::<syn::Lit>(input).map_err(|error| {
            format_err!(
                input_span,
                "expected string or byte string literal as input: {}",
                error
            )
        })?;
        let input_bytes = match lit {
            syn::Lit::Str(ref lit_str) => lit_str.value().into_bytes(),
            syn::Lit::ByteStr(ref byte_str) => byte_str.value(),
            invalid => {
                return Err(format_err!(
                    invalid.span(),
                    "expected string or byte string literal as input. found {:?}",
                    invalid,
                ))
            }
        };
        let selector = Selector::compute(&input_bytes);
        Ok(Self {
            selector,
            input: lit,
            _marker: PhantomData,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_lits_works() {
        let hex_lits = Selector::from([0xC0, 0xDE, 0xCA, 0xFE]).hex_lits();
        assert_eq!(
            hex_lits,
            [
                syn::parse_quote! { 0xC0_u8 },
                syn::parse_quote! { 0xDE_u8 },
                syn::parse_quote! { 0xCA_u8 },
                syn::parse_quote! { 0xFE_u8 },
            ]
        )
    }
}
