// Copyright (C) Use Ink (UK) Ltd.
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
use proc_macro2::{
    Span,
    TokenStream as TokenStream2,
    TokenStream,
};
use quote::{
    format_ident,
    quote,
    quote_spanned,
};

/// Returns the associated output type for an ink! trait message.
pub fn output_ident(message_name: &syn::Ident) -> syn::Ident {
    format_ident!("{}Output", message_name.to_string().to_lower_camel_case())
}

/// Returns the sequence of artificial input parameter bindings
/// for the message or constructor.
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

/// Returns the sequence of input idents for the message.
pub fn input_message_idents(inputs: ir::InputsIter) -> Vec<&syn::Ident> {
    inputs
        .map(|input| {
            match &*input.pat {
                syn::Pat::Ident(ident) => &ident.ident,
                _ => {
                    unreachable!(
                        "encountered ink! dispatch input with missing identifier"
                    )
                }
            }
        })
        .collect::<Vec<_>>()
}

/// Returns a tuple type representing the types yielded by the input types.
pub fn input_types_tuple(inputs: ir::InputsIter) -> TokenStream2 {
    let input_types = input_types(inputs);
    // println!("input_types_tuple {}", input_types.len());
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
pub fn generate_argument_list<'b, Args>(args: Args, abi: TokenStream) -> TokenStream2
where
    Args: IntoIterator<Item = &'b syn::Type>,
    <Args as IntoIterator>::IntoIter: Iterator,
{
    use syn::spanned::Spanned as _;
    args.into_iter().fold(
        quote! { ::ink::env::call::utils::EmptyArgumentList<#abi>},
        |rest, arg| {
            let span = arg.span();
            quote_spanned!(span=>
                ::ink::env::call::utils::ArgumentList<::ink::env::call::utils::Argument<#arg>, #rest, #abi>
            )
        }
    )
}

/// Generates code to uniquely identify a trait by its unique ID given only its
/// identifier.
///
/// # Note
///
/// As with all Rust macros identifiers can shadow each other so the given identifier
/// needs to be valid for the scope in which the returned code is generated.
pub fn generate_reference_to_trait_info(
    span: Span,
    trait_path: &syn::Path,
) -> TokenStream2 {
    quote_spanned!(span=>
        <<::ink::reflect::TraitDefinitionRegistry<Environment>
            as #trait_path>::__ink_TraitInfo as ::ink::reflect::TraitInfo>::ID
    )
}
