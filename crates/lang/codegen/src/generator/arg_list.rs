// Copyright 2018-2022 Parity Technologies (UK) Ltd.
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

use heck::ToLowerCamelCase as _;
use proc_macro2::TokenStream as TokenStream2;
use quote::{
    format_ident,
    quote,
    quote_spanned,
};

/// Returns the associated output type for an ink! trait message.
pub fn output_ident(message_name: &syn::Ident) -> syn::Ident {
    format_ident!("{}Output", message_name.to_string().to_lower_camel_case())
}

/// Returns the sequence of artificial input parameter bindings for the message.
///
/// # Note
///
/// This returns `__ink_binding_N` for every message input where `N` is the number
/// of the input from first to last.
pub fn input_bindings(inputs: ir::InputsIter) -> Vec<syn::Ident> {
    inputs
        .enumerate()
        .map(|(n, _)| format_ident!("__ink_binding_{}", n))
        .collect::<Vec<_>>()
}

/// Returns the sequence of input types for the message.
pub fn input_types(inputs: ir::InputsIter) -> Vec<&syn::Type> {
    inputs.map(|pat_type| &*pat_type.ty).collect::<Vec<_>>()
}

/// Returns a tuple type representing the types yielded by the input types.
pub fn input_types_tuple(inputs: ir::InputsIter) -> TokenStream2 {
    let input_types = input_types(inputs);
    if input_types.len() != 1 {
        // Pack all types into a tuple if they are not exactly 1.
        // This results in `()` for zero input types.
        quote! { ( #( #input_types ),* ) }
    } else {
        // Return the single type without turning it into a tuple.
        quote! { #( #input_types )* }
    }
}

/// Returns a tuple expression representing the bindings yielded by the inputs.
pub fn input_bindings_tuple(inputs: ir::InputsIter) -> TokenStream2 {
    let input_bindings = input_bindings(inputs);
    match input_bindings.len() {
        0 => quote! { _ },
        1 => quote! { #( #input_bindings ),* },
        _ => quote! { ( #( #input_bindings ),* ) },
    }
}

/// Builds up the `ink_env::call::utils::ArgumentList` type structure for the given types.
pub fn generate_argument_list<'b, Args>(args: Args) -> TokenStream2
where
    Args: IntoIterator<Item = &'b syn::Type>,
    <Args as IntoIterator>::IntoIter: Iterator,
{
    use syn::spanned::Spanned as _;
    args.into_iter().fold(
        quote! { ::ink_env::call::utils::EmptyArgumentList },
        |rest, arg| {
            let span = arg.span();
            quote_spanned!(span=>
                ::ink_env::call::utils::ArgumentList<::ink_env::call::utils::Argument<#arg>, #rest>
            )
        }
    )
}
