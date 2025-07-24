// Copyright (C) ink! contributors.
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

use ir::{
    Callable,
    InputsIter,
    IsDocAttribute,
    Message,
};
use proc_macro2::TokenStream as TokenStream2;
use quote::{
    quote,
    quote_spanned,
};
use syn::{
    spanned::Spanned,
    Attribute,
    Type,
};

/// Returns the equivalent Solidity ABI type for the given Rust/ink! type.
pub fn sol_type(ty: &Type) -> TokenStream2 {
    quote! {
        <#ty as ::ink::SolDecode>::SOL_NAME
    }
}

/// Returns the equivalent Solidity ABI type for the given Rust/ink! return type.
///
/// # Note
///
/// Use this function (instead of [`sol_type`]) when return type may be `Result<T, E>`,
/// because `Result<T, E>` implements `ink::SolEncode` but doesn't implement
/// `ink::SolDecode`.
pub fn sol_return_type(ty: &Type) -> TokenStream2 {
    quote! {
        <#ty as ::ink::SolEncode>::SOL_NAME
    }
}

/// Returns Solidity ABI compatible selector of an ink! message.
pub fn selector(message: &Message) -> TokenStream2 {
    let signature = call_signature(message.ident().to_string(), message.inputs());
    quote! {
        const {
            ::ink::codegen::sol::selector_bytes(#signature)
        }
    }
}

/// Returns a `u32` representation of the Solidity ABI compatible selector of an ink!
/// message.
pub fn selector_id(message: &Message) -> TokenStream2 {
    let selector_bytes = selector(message);
    quote!(
        {
            const {
                ::core::primitive::u32::from_be_bytes(#selector_bytes)
            }
        }
    )
}

/// Returns the Solidity ABI call signature for the given message name and inputs.
pub fn call_signature(name: String, inputs: InputsIter) -> TokenStream2 {
    let mut input_types_len = 0;
    let sig_param_tys: Vec<_> = inputs
        .map(|input| {
            let ty = &*input.ty;
            let sol_ty = sol_type(ty);
            let span = input.span();
            input_types_len += 1;

            quote_spanned!(span=>
                #sol_ty
            )
        })
        .collect();
    let sig_arg_fmt_params = (0..input_types_len)
        .map(|_| "{}")
        .collect::<Vec<_>>()
        .join(",");
    let sig_fmt_lit = format!("{{}}({sig_arg_fmt_params})");
    quote! {
        ::ink::codegen::utils::const_format!(#sig_fmt_lit, #name #(,#sig_param_tys)*)
    }
}

/// Returns the rustdoc string from the given item attributes.
pub fn extract_docs(attrs: &[Attribute]) -> String {
    attrs
        .iter()
        .filter_map(|attr| attr.extract_docs())
        .collect::<Vec<_>>()
        .join("\n")
}
