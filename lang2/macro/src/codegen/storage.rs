// Copyright 2018-2019 Parity Technologies (UK) Ltd.
// This file is part of ink!.
//
// ink! is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// ink! is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with ink!.  If not, see <http://www.gnu.org/licenses/>.

use crate::{
    codegen::GenerateCode,
    ir::{
        Contract,
        Function,
        Marker,
    },
};
use core::convert::TryFrom as _;
use derive_more::From;
use proc_macro2::TokenStream as TokenStream2;
use quote::{
    quote,
    quote_spanned,
};

#[derive(From)]
pub struct Storage<'a> {
    contract: &'a Contract,
}

impl GenerateCode for Storage<'_> {
    fn generate_code(&self) -> TokenStream2 {
        let storage_ident = &self.contract.storage.ident;
        let storage_span = self.contract.storage.span();

        let aliases = self.generate_aliases();
        let trait_impls = self.generate_trait_impls_for_storage();
        let access_env_impls = self.generate_access_env_trait_impls();
        let message_impls = self.generate_message_impls();
        let storage_struct = self.generate_storage_struct();
        let storage_and_env_wrapper = self.generate_storage_and_env_wrapper();
        let layout_impls = self.generate_has_layout();

        quote_spanned!(storage_span =>
            #[doc(hidden)]
            mod __ink_storage {
                use super::*;

                #aliases
                #access_env_impls
                #trait_impls
                #storage_struct
                #storage_and_env_wrapper
                #layout_impls
            }

            pub use __ink_storage::StorageAndEnv;

            const _: () = {
                // Used to make `self.env()` available in message code.
                #[allow(unused_imports)]
                use ink_core::env2::AccessEnv as _;

                #message_impls
            };
        )
    }
}

impl Storage<'_> {
    fn generate_access_env_trait_impls(&self) -> TokenStream2 {
        let access_env_impls = if self.contract.meta_info.is_dynamic_allocation_enabled() {
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
                        self.#field_idents.flush();
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
        // Filter all `ink` attributes for code generation.
        let storage = &self.contract.storage;
        let attrs = storage
            .attrs
            .iter()
            .filter(|&attr| Marker::try_from(attr.clone()).is_err());

        quote! {
            #(#attrs)*
            #[cfg_attr(
                feature = "ink-generate-abi",
                derive(type_metadata::Metadata)
            )]
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
                    self.__storage.flush();
                    self.__env.flush();
                }
            }

            impl ink_core::storage::alloc::Initialize for StorageAndEnv {
                type Args = ();

                fn default_value() -> Option<Self::Args> {
                    Some(())
                }

                fn initialize(&mut self, _args: Self::Args) {
                    self.__storage.try_default_initialize();
                    self.__env.try_default_initialize();
                }
            }

            impl ink_lang2::Storage for StorageAndEnv {}
        }
    }

    /// Generates the storage struct definition.
    fn generate_storage_struct(&self) -> TokenStream2 {
        let storage = &self.contract.storage;
        let span = storage.span();
        // Filter all `ink` attributes for code generation.
        let attrs = storage
            .attrs
            .iter()
            .filter(|&attr| Marker::try_from(attr.clone()).is_err());
        let fields = storage.fields.named.iter();

        quote_spanned!( span =>
            #(#attrs)*
            #[cfg_attr(
                feature = "ink-generate-abi",
                derive(type_metadata::Metadata, ink_abi::HasLayout)
            )]
            pub struct Storage {
                #(
                    #fields ,
                )*
            }
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
        // Filter all `ink` attributes for code generation.
        let attrs = function
            .attrs
            .iter()
            .filter(|&attr| Marker::try_from(attr.clone()).is_err());
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
            impl StorageAndEnv {
                #(
                    #fns
                )*
            }
        )
    }
}
