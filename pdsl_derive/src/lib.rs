#![recursion_limit = "128"]

extern crate proc_macro;

#[proc_macro_attribute]
pub fn module(
	_args: proc_macro::TokenStream,
	input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
	input
}

#[proc_macro_attribute]
pub fn contract(
	_args: proc_macro::TokenStream,
	input: proc_macro::TokenStream
) -> proc_macro::TokenStream {
	input
}
