#![recursion_limit = "256"]

extern crate proc_macro;

use proc_macro2;
use syn::{
	self,
	parse_macro_input
};
use quote::quote;

mod error;
mod utils;
mod contract;

use crate::contract::ContractModule;

#[proc_macro_attribute]
pub fn module(
	_args: proc_macro::TokenStream,
	input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
	// println!("pdsl::module - begin");
	// let _args_toks = parse_macro_input!(args as syn::AttributeArgs);
	// println!("[substrate_contract] before parsing");
	let named_mod = parse_macro_input!(input as ContractModule);
	// println!("[substrate_contract] after parsing");
	// println!("{:#?}", named_mod);
	let out = quote!{ #named_mod };
	// println!("{:#?}", out);
	// println!("pdsl::module - end");
	out.into()
}

#[proc_macro_attribute]
pub fn contract(
	_args: proc_macro::TokenStream,
	input: proc_macro::TokenStream
) -> proc_macro::TokenStream {
	// println!("pdsl::contract");
	input
}
