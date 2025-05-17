//! Utilities for Solidity ABI compatible codegen.

use ir::{
    Callable,
    CallableWithSelector,
    InputsIter,
    Message,
};
use proc_macro2::{
    Ident,
    TokenStream as TokenStream2,
};
use quote::{
    format_ident,
    quote,
    quote_spanned,
};
use syn::spanned::Spanned;

/// Returns Solidity ABI compatible selector of an ink! message.
pub(crate) fn solidity_selector(message: &CallableWithSelector<Message>) -> TokenStream2 {
    let call_type_ident = solidity_call_type_ident(message);
    quote!(
        {
            <__ink_sol_interop__::#call_type_ident>::SELECTOR
        }
    )
}

/// Returns a `u32` representation of the Solidity ABI compatible selector of an ink!
/// message.
pub(crate) fn solidity_selector_id(
    message: &CallableWithSelector<Message>,
) -> TokenStream2 {
    let call_type_ident = solidity_call_type_ident(message);
    quote!(
        {
            <__ink_sol_interop__::#call_type_ident>::SELECTOR_ID
        }
    )
}

/// Returns the Solidity call signature for the given message name and inputs.
pub(crate) fn solidity_call_signature(name: String, inputs: InputsIter) -> TokenStream2 {
    let mut input_types_len = 0;
    let sig_param_tys: Vec<_> = inputs
        .map(|input| {
            let ty = &*input.ty;
            let span = input.span();
            input_types_len += 1;

            quote_spanned!(span=>
                <#ty as ::ink::SolDecode>::SOL_NAME
            )
        })
        .collect();
    let sig_arg_fmt_params = (0..input_types_len)
        .map(|_| "{}")
        .collect::<Vec<_>>()
        .join(",");
    let sig_fmt_lit = format!("{{}}({})", sig_arg_fmt_params);
    quote! {
        ::ink::codegen::const_format!(#sig_fmt_lit, #name #(,#sig_param_tys)*)
    }
}

/// Returns the Solidity call info type identifier for an ink! message.
pub(crate) fn solidity_call_type_ident(message: &CallableWithSelector<Message>) -> Ident {
    let ident = message.ident();
    let id = message.composed_selector().into_be_u32();
    format_ident!("{ident}{id}Call")
}
