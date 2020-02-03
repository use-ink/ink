// Copyright 2018-2019 Parity Technologies (UK) Ltd.
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

use derive_more::From;
use proc_macro2::TokenStream as TokenStream2;
use quote::{
    quote,
    quote_spanned,
};

use crate::{
    codegen::{
        cross_calling::CrossCallingConflictCfg,
        GenerateCode,
        GenerateCodeUsing,
    },
    ir::{
        self,
        utils,
        Contract,
        Function,
    },
};

#[derive(From)]
pub struct Storage<'a> {
    contract: &'a Contract,
}

impl<'a> GenerateCodeUsing for Storage<'a> {
    fn contract(&self) -> &ir::Contract {
        self.contract
    }
}

impl GenerateCode for Storage<'_> {
    fn generate_code(&self) -> TokenStream2 {
        let storage_span = self.contract.storage.span();

        let conflic_depedency_cfg = self.generate_code_using::<CrossCallingConflictCfg>();
        let trait_impls = self.generate_trait_impls_for_storage();
        let access_env_impls = self.generate_access_env_trait_impls();
        let message_impls = self.generate_message_impls();
        let storage_struct = self.generate_storage_struct();

        let use_emit_event = if !self.contract.events.is_empty() {
            // Required to allow for `self.env().emit_event(..)` in messages and constructors.
            quote! { use __ink_private::EmitEvent as _; }
        } else {
            quote! {}
        };

        quote_spanned!(storage_span =>
            #[doc(hidden)]
            #conflic_depedency_cfg
            mod __ink_storage {
                use super::*;

                #access_env_impls
                #trait_impls
                #storage_struct
            }

            #conflic_depedency_cfg
            pub use __ink_storage::Storage;

            #conflic_depedency_cfg
            const _: () = {
                // Used to make `self.env()` available in message code.
                #[allow(unused_imports)]
                use ink_lang::Env as _;

                #use_emit_event
                #message_impls
            };
        )
    }
}

impl Storage<'_> {
    fn generate_access_env_trait_impls(&self) -> TokenStream2 {
        quote! {
            impl<'a> ink_lang::Env for &'a Storage {
                type EnvAccess = ink_lang::EnvAccess<'a, EnvTypes>;

                fn env(self) -> Self::EnvAccess {
                    Default::default()
                }
            }
        }
    }

    fn generate_trait_impls_for_storage(&self) -> TokenStream2 {
        let field_idents = &self
            .contract
            .storage
            .fields
            .named
            .iter()
            .map(|named_field| &named_field.ident)
            .collect::<Vec<_>>();

        quote! {
            impl ink_core::storage::alloc::AllocateUsing for Storage {
                unsafe fn allocate_using<A>(alloc: &mut A) -> Self
                where
                    A: ink_core::storage::alloc::Allocate,
                {
                    Self {
                        #(
                            #field_idents: ink_core::storage::alloc::AllocateUsing::allocate_using(alloc),
                        )*
                    }
                }
            }

            impl ink_core::storage::Flush for Storage {
                fn flush(&mut self) {
                    #(
                        ink_core::storage::Flush::flush(&mut self.#field_idents);
                    )*
                }
            }

            impl ink_core::storage::alloc::Initialize for Storage {
                type Args = ();

                fn default_value() -> Option<Self::Args> {
                    Some(())
                }

                fn initialize(&mut self, _args: Self::Args) {
                    #(
                        self.#field_idents.try_default_initialize();
                    )*
                }
            }
        }
    }

    /// Generates the storage struct definition.
    fn generate_storage_struct(&self) -> TokenStream2 {
        let storage = &self.contract.storage;
        let span = storage.span();
        let attrs = utils::filter_non_ink_attributes(&storage.attrs);
        let mut fields = storage.fields.clone();
        fields.named.iter_mut().for_each(|field| {
            field.vis = syn::Visibility::Public(syn::VisPublic {
                pub_token: Default::default(),
            })
        });

        quote_spanned!( span =>
            #(#attrs)*
            #[cfg_attr(
                feature = "ink-generate-abi",
                derive(type_metadata::Metadata, ink_abi::HasLayout)
            )]
            #[cfg_attr(any(test, feature = "test-env"), derive(Debug))]
            pub struct Storage
                #fields

            impl ink_lang::Storage for Storage {}
        )
    }

    /// Generate a single message defined on the storage struct.
    fn generate_message(&self, function: &Function) -> TokenStream2 {
        let span = function.span();
        // Generate `pub` functions for constructors and messages only.
        let vis = if function.is_constructor() || function.is_message() {
            quote_spanned!(span => pub)
        } else {
            quote_spanned!(span => )
        };
        let attrs = utils::filter_non_ink_attributes(&self.contract.storage.attrs);
        let ident = &function.sig.ident;
        let (_, type_generics, where_clause) = function.sig.generics.split_for_impl();
        let inputs = &function.sig.inputs;
        let output = &function.sig.output;
        let block = &function.block;
        quote_spanned!( span =>
            #( #attrs )*
            #vis fn #ident #type_generics (
                #inputs,
            ) #output
            #where_clause
            #block
        )
    }

    /// Generates all the constructors, messages and methods defined on the storage struct.
    fn generate_message_impls(&self) -> TokenStream2 {
        let storage = &self.contract.storage;
        let span = storage.span();
        let fns = self
            .contract
            .functions
            .iter()
            .map(|fun| self.generate_message(fun));
        quote_spanned!( span =>
            #[cfg_attr(feature = "cargo-clippy", allow(clippy::new_ret_no_self))]
            impl Storage {
                #(
                    #fns
                )*
            }
        )
    }
}
