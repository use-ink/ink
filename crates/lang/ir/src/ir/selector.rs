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
use proc_macro2::{
    Span,
    TokenStream as TokenStream2,
};
use std::marker::PhantomData;

/// The selector of an ink! dispatchable.
///
/// # Note
///
/// This is equal to the first four bytes of the SHA-3 hash of a function's name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Selector {
    bytes: [u8; 4],
}

impl Selector {
    /// Creates a new selector from the given raw bytes.
    pub fn from_bytes(bytes: [u8; 4]) -> Self {
        Self { bytes }
    }

    /// Computes the BLAKE-2 256-bit based selector from the given input bytes.
    pub fn new(input: &[u8]) -> Self {
        let mut output = [0; 32];
        blake2b_256(input, &mut output);
        Self::from_bytes([output[0], output[1], output[2], output[3]])
    }

    /// Returns the underlying four bytes.
    pub fn as_bytes(&self) -> &[u8; 4] {
        &self.bytes
    }

    /// Returns a unique identifier as `usize`.
    pub fn unique_id(self) -> usize {
        self.into_be_u32() as usize
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
        Self::from_bytes(bytes)
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
        let lit = syn::parse2::<syn::Lit>(input).map_err(|error| {
            format_err!(
                Span::call_site(),
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
        let selector = Selector::new(&input_bytes);
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
        let hex_lits = Selector::from_bytes([0xC0, 0xDE, 0xCA, 0xFE]).hex_lits();
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
