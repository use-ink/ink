// Copyright 2018-2020 Parity Technologies (UK) Ltd.
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

use crate::GenerateCode;
use derive_more::From;
use ir::ChainExtensionMethod;
use proc_macro2::TokenStream as TokenStream2;
use quote::{
    format_ident,
    quote_spanned,
};
use syn::spanned::Spanned;

/// Generator to create an ink! chain extension.
#[derive(From)]
pub struct ChainExtension<'a> {
    extension: &'a ir::ChainExtension,
}

impl ChainExtension<'_> {
    fn generate_for_instance_method(method: &ChainExtensionMethod) -> TokenStream2 {
        let span = method.span();
        let attrs = method.attrs();
        let ident = method.ident();
        let id = method.id();
        let raw_id = id.into_u32();
        let sig = method.sig();
        let inputs = &sig.inputs;
        let input_bindings = method.inputs().map(|pat_type| &pat_type.pat);
        let input_types = method.inputs().map(|pat_type| &pat_type.ty);
        // Mostly tuple-based type representation of the inputs:
        // Special case for when there is exactly one input type.
        // - 0 inputs          -> ()
        // - 1 input T         -> T
        // - n inputs A, B. .. -> (A, B, ..)
        let compound_input_type = match inputs.len() {
            0 => quote_spanned!(span=> ()),
            1 => quote_spanned!(span=> #( #input_types )* ),
            _n => quote_spanned!(span=> ( #( #input_types ),* ) ),
        };
        // Mostly tuple-based value representation of the inputs:
        // Special case for when there is exactly one input value.
        // - 0 inputs          -> ()
        // - 1 input a         -> a
        // - n inputs a, b. .. -> (a, b, ..)
        let compound_input_bindings = match inputs.len() {
            0 => quote_spanned!(span=> ()),
            1 => quote_spanned!(span=> #( #input_bindings )* ),
            _n => quote_spanned!(span=> ( #( #input_bindings ),* ) ),
        };
        let output = &sig.output;
        let output_type = match output {
            syn::ReturnType::Default => quote_spanned!(output.span()=> ()),
            syn::ReturnType::Type(_arrow, ty) => {
                quote_spanned!(output.span()=> #ty)
            }
        };
        let expect_output = method.expect_output();
        let expect_ok = method.expect_ok();
        let where_output_is_result = if !expect_ok {
            // Enforce that the output type of the chain extension is a `Result`.
            quote_spanned!(span=>
                #output: ::ink_lang::chain_extension::IsResultType,
            )
        } else {
            quote::quote! {}
        };
        let where_output_impls_from_error_code = if !expect_output && !expect_ok {
            // Enforce that `E` of the `Result<T, E>` output of the chain extension
            // implements conversion from `Self::ErrorCode`.
            quote_spanned!(span=>
                <#output as ::ink_lang::chain_extension::IsResultType>::Err: ::core::convert::From<Self::ErrorCode>,
            )
        } else {
            quote::quote! {}
        };
        quote_spanned!(span=>
            #( #attrs )*
            pub fn #ident(self, #inputs) -> ::core::result::Result<#output_type, ::ink_env::Error>
            where
                #where_output_is_result
                #where_output_impls_from_error_code
            {
                ::ink_env::call_chain_extension::<#compound_input_type, #output_type>(
                    #raw_id,
                    &#compound_input_bindings
                )
            }
        )
    }
}

impl GenerateCode for ChainExtension<'_> {
    fn generate_code(&self) -> TokenStream2 {
        let span = self.extension.span();
        let attrs = self.extension.attrs();
        let ident = self.extension.ident();
        let instance_methods = self
            .extension
            .iter_methods()
            .map(Self::generate_for_instance_method);
        let instance_ident = format_ident!("__ink_{}Instance", ident);
        quote_spanned!(span =>
            #(#attrs)*
            pub enum #ident {}

            const _: () = {
                #[allow(non_camel_case_types)]
                struct __ink_Private;
                #[allow(non_camel_case_types)]
                pub struct #instance_ident {
                    __ink_private: __ink_Private
                }

                impl #instance_ident {
                    #( #instance_methods )*
                }

                impl ::ink_lang::ChainExtensionInstance for #ident {
                    type Instance = #instance_ident;

                    fn instantiate() -> Self::Instance {
                        Self::Instance { __ink_private: __ink_Private }
                    }
                }
            };
        )
    }
}
