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

use core::convert::TryFrom as _;
use ink_lang_codegen::generate_code;
use ink_lang_ir::Blake2x256Macro;
use proc_macro2::TokenStream as TokenStream2;
use syn::Result;

pub fn generate_blake2x256_hash(input: TokenStream2) -> TokenStream2 {
    match generate_blake2x256_hash_or_err(input) {
        Ok(tokens) => tokens,
        Err(err) => err.to_compile_error(),
    }
}

pub fn generate_blake2x256_hash_or_err(input: TokenStream2) -> Result<TokenStream2> {
    let hash = Blake2x256Macro::try_from(input)?;
    Ok(generate_code(&hash))
}
