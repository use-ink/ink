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

use proc_macro2::TokenStream as TokenStream2;
use syn::Result;

#[cfg(feature = "ink-generate-abi")]
use crate::old_abi;
use crate::{
    gen,
    hir,
    parser,
};

pub fn generate(input: TokenStream2) -> TokenStream2 {
    match generate_or_err(input) {
        Ok(tokens) => tokens,
        Err(err) => err.to_compile_error(),
    }
}

pub fn generate_or_err(input: TokenStream2) -> Result<TokenStream2> {
    let ast_contract = parser::parse_contract(input)?;
    let hir_contract = hir::Contract::from_ast(&ast_contract)?;
    #[cfg(feature = "ink-generate-abi")]
    old_abi::generate_old_abi(&hir_contract)?;
    let tokens = gen::generate_code(&hir_contract);
    Ok(tokens)
}
