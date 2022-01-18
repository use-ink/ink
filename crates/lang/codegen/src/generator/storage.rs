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
        let use_emit_event =
            self.contract.module().events().next().is_some().then(|| {
                // Required to allow for `self.env().emit_event(...)` in messages and constructors.
                quote! { use ::ink_lang::codegen::EmitEvent as _; }
            });

        let new_storage_structs = self.generate_storage_value_structs();

        quote_spanned!(storage_span =>
            #storage_struct
            #access_env_impls
            #new_storage_structs

            const _: () = {
                // Used to make `self.env()` and `Self::env()` available in message code.
                #[allow(unused_imports)]
                use ::ink_lang::codegen::{
                    Env as _,
                    StaticEnv as _,
                };
                #use_emit_event
            };
        )
    }
}

impl Storage<'_> {
    fn generate_access_env_trait_impls(&self) -> TokenStream2 {
        let storage_ident = &self.contract.module().storage().ident();
        quote! {
            const _: () = {
                impl<'a> ::ink_lang::codegen::Env for &'a #storage_ident {
                    type EnvAccess = ::ink_lang::EnvAccess<
                        'a, <#storage_ident as ::ink_lang::reflect::ContractEnv>::Env>;

                    fn env(self) -> Self::EnvAccess {
                        <<Self as ::ink_lang::codegen::Env>::EnvAccess
                            as ::core::default::Default>::default()
                    }
                }

                impl<'a> ::ink_lang::codegen::StaticEnv for #storage_ident {
                    type EnvAccess = ::ink_lang::EnvAccess<
                        'static, <#storage_ident as ::ink_lang::reflect::ContractEnv>::Env>;

                    fn env() -> Self::EnvAccess {
                        <<Self as ::ink_lang::codegen::StaticEnv>::EnvAccess
                            as ::core::default::Default>::default()
                    }
                }
            };
        }
    }

    /// Generates the storage struct definition.
    fn generate_storage_struct(&self) -> TokenStream2 {
        let storage = self.contract.module().storage();
        let span = storage.span();
        let ident = storage.ident();
        let attrs = storage.attrs();
        let fields = storage.fields();
        quote_spanned!( span =>
            #(#attrs)*
            #[cfg_attr(
                feature = "std",
                derive(::ink_storage::traits::StorageLayout)
            )]
            #[derive(::ink_storage::traits::SpreadLayout)]
            #[cfg_attr(test, derive(::core::fmt::Debug))]
            pub struct #ident {
                #( #fields ),*
            }

            const _: () = {
                impl ::ink_lang::reflect::ContractName for #ident {
                    const NAME: &'static str = ::core::stringify!(#ident);
                }

                impl ::ink_lang::codegen::ContractRootKey for #ident {
                    const ROOT_KEY: ::ink_primitives::Key = ::ink_primitives::Key::new([0x00; 32]);
                }
            };
        )
    }

    fn generate_storage_value_structs(&self) -> TokenStream2 {
        let storage = self.contract.module().storage();
        let span = storage.span();
        let ident = storage.ident();
        let _attrs = storage.attrs();
        let fields = storage.fields();

        use heck::ToUpperCamelCase as _;

        // TODO: Stop cloning in places where we use `fields`
        let field_structs = fields.clone().map(|field| {
            let ident = field.ident.as_ref().unwrap();
            let struct_ident = quote::format_ident!(
                "__ink_StorageValue{}",
                ident.to_string().to_upper_camel_case()
            );
            let ty = &field.ty;

            let input = format!("{}:{}", ident, struct_ident);
            let mut storage_key = [0u8; 32];
            ir::blake2b_256(input.as_bytes(), &mut storage_key);

            // TODO: Ensure spans line up properly
            quote!(
                pub struct #struct_ident {
                    __private: ()
                }

                impl ::ink_lang::codegen::ContractRootKey for #struct_ident {
                    const ROOT_KEY: ::ink_primitives::Key = ::ink_primitives::Key::new([#(#storage_key,)*]);
                }

                impl ::ink_lang::codegen::StorageValue for #struct_ident {
                    type Value = #ty;
                }
            )
        });

        let new_contract_ident = quote::format_ident!("{}2", &ident);
        let internal_fields = fields.clone().map(|field| {
            let ident = field.ident.as_ref().unwrap();
            let ty = quote::format_ident!(
                "__ink_StorageValue{}",
                ident.to_string().to_upper_camel_case()
            );

            quote! {
                #ident: #ty
            }
        });

        let getter_methods = fields.clone().map(|field| {
            let ident = field.ident.as_ref().unwrap();
            let ty = quote::format_ident!(
                "__ink_StorageValue{}",
                ident.to_string().to_upper_camel_case()
            );

            let getter_mut = quote::format_ident!("{}_mut", ident);

            quote! {
                pub fn #ident(&self) -> &#ty { &self.#ident }
                pub fn #getter_mut(&mut self) -> &mut #ty { &mut self.#ident }
            }
        });

        let generated_pairs_init = fields.clone().map(|field| {
            let ident = field.ident.as_ref().unwrap();
            let internal_ty = quote::format_ident!(
                "__ink_StorageValue{}",
                ident.to_string().to_upper_camel_case()
            );

            quote! {
                #ident: #internal_ty { __private: (), }
            }
        });

        let initial_storage_write = fields.clone().map(|field| {
            let ident = field.ident.as_ref().unwrap();
            quote! {
                self.#ident.write(&#ident)
            }
        });

        let new_contract = quote! {
           use ::ink_lang::codegen::StorageValue;

           pub struct #new_contract_ident {
               #(#internal_fields,)*
           }

           impl #new_contract_ident {
               pub fn initialize(&mut self, #(#fields)*) -> Self {
                   let s = Self {
                       #(#generated_pairs_init,)*
                   };

                   #(#initial_storage_write;)*
                   s
               }

               #(#getter_methods)*
           }
        };

        quote_spanned!( span =>
            #new_contract

            #(#field_structs)*
        )
    }
}
