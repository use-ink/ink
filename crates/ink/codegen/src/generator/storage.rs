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

use crate::GenerateCode;
use derive_more::From;
use proc_macro2::TokenStream as TokenStream2;
use quote::{
    quote,
    quote_spanned,
};
use syn::spanned::Spanned as _;

/// Generator to create the ink! storage struct and important trait implementations.
#[derive(From)]
pub struct Storage<'a> {
    contract: &'a ir::Contract,
}
impl_as_ref_for_generator!(Storage);

impl GenerateCode for Storage<'_> {
    fn generate_code(&self) -> TokenStream2 {
        let storage_span = self.contract.module().storage().span();
        let access_env_impls = self.generate_access_env_trait_impls();
        let storage_struct = self.generate_storage_struct();
        quote_spanned!(storage_span =>
            #storage_struct
            #access_env_impls

            const _: () = {
                // Used to make `self.env()` and `Self::env()` available in message code.
                #[allow(unused_imports)]
                use ::ink::codegen::{
                    Env as _,
                    StaticEnv as _,
                };
            };
        )
    }
}

impl Storage<'_> {
    fn generate_access_env_trait_impls(&self) -> TokenStream2 {
        let storage_ident = &self.contract.module().storage().ident();
        quote! {
            const _: () = {
                impl<'a> ::ink::codegen::Env for &'a #storage_ident {
                    type EnvAccess = ::ink::EnvAccess<
                        'a, <#storage_ident as ::ink::env::ContractEnv>::Env>;

                    fn env(self) -> Self::EnvAccess {
                        <<Self as ::ink::codegen::Env>::EnvAccess
                            as ::core::default::Default>::default()
                    }
                }

                impl<'a> ::ink::codegen::StaticEnv for #storage_ident {
                    type EnvAccess = ::ink::EnvAccess<
                        'static, <#storage_ident as ::ink::env::ContractEnv>::Env>;

                    fn env() -> Self::EnvAccess {
                        <<Self as ::ink::codegen::StaticEnv>::EnvAccess
                            as ::core::default::Default>::default()
                    }
                }
            };
        }
    }

    /// Generates the storage struct definition.
    ///
    /// # Developer Note
    ///
    /// The `fortanix` config attribute is used here to convey the
    /// information that the generated struct is an ink! storage struct to `dylint`.
    ///
    /// We decided on this attribute to mark the function, as it has to be a
    /// key-value pair that is well known to `cargo`. `fortanix` seems like an obscure
    /// vendor, for  which it is highly unlikely that someone will ever compile
    /// a contract for.
    fn generate_storage_struct(&self) -> TokenStream2 {
        let storage = self.contract.module().storage();
        let span = storage.span();
        let ident = storage.ident();
        let generics = storage.generics();
        let attrs = storage.attrs();
        let fields = storage.fields();
        quote_spanned!( span =>
            #(#attrs)*
            #[::ink::storage_item]
            #[cfg_attr(test, derive(::core::fmt::Debug))]
            #[cfg(not(target_vendor = "fortanix"))]
            pub struct #ident #generics {
                #( #fields ),*
            }

            const _: () = {
                impl ::ink::reflect::ContractName for #ident {
                    const NAME: &'static str = ::core::stringify!(#ident);
                }
            };
        )
    }
}
