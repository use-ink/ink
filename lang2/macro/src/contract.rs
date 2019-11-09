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

use crate::{
    codegen::GenerateCode as _,
    ir,
    lint,
};
use core::convert::TryFrom;
use proc_macro2::TokenStream as TokenStream2;
use syn::Result;

pub fn generate(attr: TokenStream2, input: TokenStream2) -> TokenStream2 {
    match generate_or_err(attr, input) {
        Ok(tokens) => tokens,
        Err(err) => err.to_compile_error(),
    }
}

pub fn generate_or_err(attr: TokenStream2, input: TokenStream2) -> Result<TokenStream2> {
    lint::idents_respect_pred(
        input.clone(),
        move |ident| !ident.to_string().starts_with("__ink"),
        move |ident| {
            format_err!(
                ident,
                "identifiers starting with `__ink` are forbidden in ink!"
            )
        },
    )?;
    let params = syn::parse2::<ir::Params>(attr)?;
    let rust_mod = syn::parse2::<syn::ItemMod>(input)?;
    let ink_ir = ir::Contract::try_from((params, rust_mod))?;
    Ok(ink_ir.generate_code())
}
