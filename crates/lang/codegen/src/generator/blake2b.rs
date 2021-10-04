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

use crate::GenerateCode;
use derive_more::From;
use ir::HexLiteral;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote_spanned;

/// Generates code for the `selector_id!` macro.
#[derive(From)]
pub struct Blake2x256<'a> {
    /// The `blake2x256!` macro input.
    macro_input: &'a ir::Blake2x256Macro,
}

impl GenerateCode for Blake2x256<'_> {
    /// Generates `selector_id!` macro code.
    fn generate_code(&self) -> TokenStream2 {
        let span = self.macro_input.input().span();
        let hash_bytes = self
            .macro_input
            .hash()
            .map(|byte| byte.hex_padded_suffixed());
        quote_spanned!(span=> [ #( #hash_bytes ),* ] )
    }
}
