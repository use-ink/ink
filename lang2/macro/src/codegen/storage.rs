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
        let aliases = self.generate_aliases();
        let trait_impls = self.generate_trait_impls_for_storage();
        let access_env_impls = self.generate_access_env_trait_impls();
        let message_impls = self.generate_message_impls();
        let storage_struct = self.generate_storage_struct();
        let storage_and_env_wrapper = self.generate_storage_and_env_wrapper();
        let layout_impls = self.generate_has_layout();

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

                #aliases
                #access_env_impls
                #trait_impls
                #storage_struct
                #storage_and_env_wrapper
                #layout_impls
            }

            #conflic_depedency_cfg
            pub use __ink_storage::StorageAndEnv;

            #conflic_depedency_cfg
            const _: () = {
                // Used to make `self.env()` available in message code.
                #[allow(unused_imports)]
                use ink_core::env2::AccessEnv as _;

                #use_emit_event
                #message_impls
            };
        )
    }
}

impl Storage<'_> {
    fn generate_access_env_trait_impls(&self) -> TokenStream2 {
        let access_env_impls = if self.contract.meta_info.is_dynamic_allocation_enabled()
        {
            quote! {
                impl ink_lang2::AccessEnv<Env> for StorageAndEnv {
                    fn access_env(&mut self) -> &mut ink_core::env2::EnvAccess<Env> {
                        self.__env.env_mut()
                    }
                }
            }
        } else {
            quote! {
                impl ink_lang2::AccessEnv<Env> for StorageAndEnv {
                    fn access_env(&mut self) -> &mut ink_core::env2::EnvAccess<Env> {
                        &mut self.__env
                    }
                }
            }
        };
        quote! {
            #access_env_impls

            impl<'a> ink_core::env2::AccessEnv for &'a StorageAndEnv {
                type Target = <&'a UsedEnv as ink_core::env2::AccessEnv>::Target;

                fn env(self) -> Self::Target {
                    ink_core::env2::AccessEnv::env(&self.__env)
                }
            }

            impl<'a> ink_core::env2::AccessEnv for &'a mut StorageAndEnv {
                type Target = <&'a mut UsedEnv as ink_core::env2::AccessEnv>::Target;

                fn env(self) -> Self::Target {
                    ink_core::env2::AccessEnv::env(&mut self.__env)
                }
            }
        }
    }

    fn generate_aliases(&self) -> TokenStream2 {
        if self.contract.meta_info.is_dynamic_allocation_enabled() {
            quote! {
                pub type UsedEnv = ink_core::env2::DynEnv<ink_core::env2::EnvAccess<Env>>;
            }
        } else {
            quote! {
                pub type UsedEnv = ink_core::env2::EnvAccess<Env>;
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

    fn generate_has_layout(&self) -> TokenStream2 {
        let env_layout = if self.contract.meta_info.is_dynamic_allocation_enabled() {
            quote! { ink_abi::LayoutField::new("env", self.__env.layout()), }
        } else {
            quote! {}
        };
        quote! {
            #[cfg(feature = "ink-generate-abi")]
            impl ink_abi::HasLayout for StorageAndEnv {
                fn layout(&self) -> ink_abi::StorageLayout {
                    use type_metadata::Metadata as _;
                    ink_abi::LayoutStruct::new(
                        Self::meta_type(),
                        vec![
                            ink_abi::LayoutField::new("storage", self.__storage.layout()),
                            #env_layout
                        ],
                    )
                    .into()
                }
            }

        }
    }

    fn generate_storage_and_env_wrapper(&self) -> TokenStream2 {
        let attrs = utils::filter_non_ink_attributes(&self.contract.storage.attrs);

        quote! {
            #(#attrs)*
            #[cfg_attr(
                feature = "ink-generate-abi",
                derive(type_metadata::Metadata)
            )]
            #[cfg_attr(any(test, feature = "test-env"), derive(Debug))]
            pub struct StorageAndEnv {
                __storage: Storage,
                __env: UsedEnv,
            }

            impl core::ops::Deref for StorageAndEnv {
                type Target = Storage;

                fn deref(&self) -> &Self::Target {
                    &self.__storage
                }
            }

            impl core::ops::DerefMut for StorageAndEnv {
                fn deref_mut(&mut self) -> &mut Self::Target {
                    &mut self.__storage
                }
            }

            impl ink_core::storage::alloc::AllocateUsing for StorageAndEnv {
                unsafe fn allocate_using<A>(alloc: &mut A) -> Self
                where
                    A: ink_core::storage::alloc::Allocate,
                {
                    Self {
                        __storage: ink_core::storage::alloc::AllocateUsing::allocate_using(alloc),
                        __env: ink_core::storage::alloc::AllocateUsing::allocate_using(alloc),
                    }
                }
            }

            impl ink_core::storage::Flush for StorageAndEnv {
                fn flush(&mut self) {
                    ink_core::storage::Flush::flush(&mut self.__storage);
                    ink_core::storage::Flush::flush(&mut self.__env);
                }
            }



            impl ink_core::storage::alloc::Initialize for StorageAndEnv {
                type Args = ();

                fn default_value() -> Option<Self::Args> {
                    Some(())
                }

                fn initialize(&mut self, _args: Self::Args) {
                    ink_core::storage::alloc::Initialize::try_default_initialize(&mut self.__storage);
                    ink_core::storage::alloc::Initialize::try_default_initialize(&mut self.__env);
                }
            }

            impl ink_lang2::Storage for StorageAndEnv {}
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
            impl StorageAndEnv {
                #(
                    #fns
                )*
            }
        )
    }
}
