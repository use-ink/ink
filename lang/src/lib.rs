#![recursion_limit = "128"]

extern crate proc_macro;

use quote::{
	// quote,
	ToTokens,
};

#[proc_macro]
pub fn contract(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    match contract_gen_inner(input) {
        Ok(tokens) => tokens,
        Err(err) => { err.into_token_stream().into() },
    }
}

#[macro_use]
mod errors;

mod ast;
mod hir;
mod parser;
mod ident_ext;

use errors::Result;

fn contract_gen_inner(input: proc_macro::TokenStream) -> Result<proc_macro::TokenStream> {
    let ast_contract = parser::parse_contract(input.clone())?;
    let _hir_contract = hir::Contract::from_ast(&ast_contract)?;
    // gen::gir::generate(&hir_program)?;
    // let tokens = gen::codegen(&hir_program);
    // Ok(tokens.into())
	Ok(proc_macro::TokenStream::new())
}
