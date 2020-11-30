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
use quote::{format_ident, quote_spanned};
use syn::{spanned::Spanned, FnArg};

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
        let input_bindings = inputs.iter().map(|fn_arg| {
            match fn_arg {
                FnArg::Typed(pat_type) => &*pat_type.pat,
                FnArg::Receiver(receiver) => {
                    panic!(
                        "encountered unexpected self receiver: {:?}",
                        receiver
                    )
                }
            }
        });
        let input_types = inputs.iter().map(|fn_arg| {
            match fn_arg {
                FnArg::Typed(pat_type) => &*pat_type.ty,
                FnArg::Receiver(receiver) => {
                    panic!(
                        "encountered unexpected self receiver: {:?}",
                        receiver
                    )
                }
            }
        });
        let output = &sig.output;
        let output_type = match output {
            syn::ReturnType::Default => quote_spanned!(output.span()=> "()"),
            syn::ReturnType::Type(_arrow, ty) => {
                quote_spanned!(output.span()=> #ty)
            }
        };
        quote_spanned!(span=>
            #( #attrs )*
            pub fn #ident(self, #inputs) -> ::core::result::Result<#output_type, ::ink_env::Error> {
                ::ink_env::call_chain_extension::< #( #input_types ),*, #output_type>(
                    #raw_id,
                    #( #input_bindings ),*
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
        let instance_methods = self.extension.iter_methods().map(Self::generate_for_instance_method);
        let instance_ident = format_ident!("__ink_{}Instance", ident);
        quote_spanned!(span =>
            #(#attrs)*
            pub enum #ident {}

            const _: () = {
                struct __ink_Private;
                pub struct #instance_ident {
                    __ink_private: __ink_Private
                }

                impl #instance_ident {
                    #( #instance_methods )*
                }

                impl ::ink_lang::ChainExtensionInstance {
                    type Instance = #ident;

                    fn instantiate() -> Self::Instance {
                        #instance_ident { __ink_private: __ink_Private }
                    }
                }
            };
        )
    }
}
