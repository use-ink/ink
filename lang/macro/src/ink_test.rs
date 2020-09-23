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

use crate::{
    codegen::GenerateCode as _,
    ir,
    lint,
};
use core::convert::TryFrom;
use ink_lang_ir::format_err_spanned;
use proc_macro2::TokenStream as TokenStream2;
use syn::Result;

pub fn generate(input: TokenStream2) -> TokenStream2 {
    match generate_or_err(input) {
        Ok(tokens) => tokens,
        Err(err) => err.to_compile_error(),
    }
}

pub fn generate_or_err(input: TokenStream2) -> Result<TokenStream2> {
    lint::idents_respect_pred(
        input.clone(),
        move |ident| !ident.to_string().starts_with("__ink"),
        move |ident| {
            format_err_spanned!(
                ident,
                "identifiers starting with `__ink` are forbidden in ink!"
            )
        },
    )?;
    let rust_fn = syn::parse2::<syn::ItemFn>(input)?;
    let ink_ir = ir::InkTest::try_from(rust_fn)?;
    Ok(ink_ir.generate_code())
}
