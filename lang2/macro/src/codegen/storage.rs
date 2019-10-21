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
use quote::quote_spanned;

/// Generates code for the contract's storage struct.
///
/// Also includes code generation for all required trait implementations.
#[derive(From)]
pub struct Storage<'a> {
    /// The contract to generate code for.
    contract: &'a Contract,
}

macro_rules! format_ident {
    ( $span_expr:expr => $format_str:literal $(, $format_frag:expr)* $(,)? ) => {
        syn::Ident::new(&format!($format_str, $($format_frag)*), $span_expr)
    };
}

impl Storage<'_> {
    /// Generates the storage struct definition.
    fn generate_storage_struct(&self) -> TokenStream2 {
        let storage = &self.contract.storage;
        let span = storage.span();
        // Filter all `ink` attributes for code generation.
        let attrs = storage
            .attrs
            .iter()
            .filter(|&attr| Marker::try_from(attr.clone()).is_err());
        let name = &storage.ident;
        let fields = storage.fields.named.iter();

        quote_spanned!( span =>
            #(#attrs)*
            pub struct #name {
                #(
                    #fields ,
                )*
                env: ink_core::env2::DynEnv<ink_core::env2::EnvAccessMut<Env>>,
            }
        )
    }

    /// Generates the storage and environment struct definition.
    fn generate_storage_and_env_struct(&self) -> TokenStream2 {
        let storage = &self.contract.storage;
        let span = storage.span();

        let attrs = storage
            .attrs
            .iter()
            .filter(|&attr| Marker::try_from(attr.clone()).is_err());
        let name = format_ident!(span => "StorageAndEnv{}", "Flipper");
        let name = syn::Ident::new(&format!("StorageAndEnvFor{}", "Flipper"), span);
        unimplemented!()
    }

    /// Generates the `AllocateUsing` trait implementation for the storage struct.
    fn allocate_using_impl(&self) -> TokenStream2 {
        let storage = &self.contract.storage;
        let span = storage.span();
        let ident = &storage.ident;
        let fields = storage.fields.named.iter().map(|field| {
            field
                .ident
                .as_ref()
                .expect("we only operate on named fields; qed")
        });
        quote_spanned!( span =>
            impl ink_core::storage::alloc::AllocateUsing for #ident {
                unsafe fn allocate_using<A>(alloc: &mut A) -> Self
                where
                    A: ink_core::storage::alloc::Allocate,
                {
                    Self {
                        #(
                            #fields: ink_core::storage::alloc::AllocateUsing::allocate_using(alloc),
                        )*
                        env: ink_core::storage::alloc::AllocateUsing::allocate_using(alloc),
                    }
                }
            }
        )
    }

    /// Generates the `Flush` trait implementation for the storage struct.
    fn flush_impl(&self) -> TokenStream2 {
        let storage = &self.contract.storage;
        let span = storage.span();
        let ident = &storage.ident;
        let fields = storage.fields.named.iter().map(|field| {
            field
                .ident
                .as_ref()
                .expect("we only operate on named fields; qed")
        });
        quote_spanned!( span =>
            impl ink_core::storage::Flush for #ident {
                fn flush(&mut self) {
                    #(
                        self.#fields.flush();
                    )*
                    self.env.flush();
                }
            }
        )
    }

    /// Generates the `Initialize` trait implementation for the storage struct.
    fn initialize_impl(&self) -> TokenStream2 {
        let storage = &self.contract.storage;
        let span = storage.span();
        let ident = &storage.ident;
        let fields = storage.fields.named.iter().map(|field| {
            field
                .ident
                .as_ref()
                .expect("we only operate on named fields; qed")
        });
        quote_spanned!( span =>
            impl ink_core::storage::alloc::Initialize for #ident {
                type Args = ();

                #[inline(always)]
                fn default_value() -> Option<Self::Args> {
                    Some(())
                }

                fn initialize(&mut self, _args: Self::Args) {
                    #(
                        self.#fields.try_default_initialize();
                    )*
                    self.env.try_default_initialize();
                }
            }
        )
    }

    /// Generate a single function defined on the storage struct.
    fn generate_function(&self, function: &Function) -> TokenStream2 {
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
    fn generate_functions(&self) -> TokenStream2 {
        let storage = &self.contract.storage;
        let span = storage.span();
        let ident = &storage.ident;
        let fns = self
            .contract
            .functions
            .iter()
            .map(|fun| self.generate_function(fun));
        quote_spanned!( span =>
            impl #ident {
                #(
                    #fns
                )*
            }
        )
    }
}

impl GenerateCode for Storage<'_> {
    fn generate_code(&self) -> TokenStream2 {
        let struct_def = self.generate_storage_struct();
        let allocate_using_impl = self.allocate_using_impl();
        let flush_impl = self.flush_impl();
        let initialize_impl = self.initialize_impl();
        let methods = self.generate_functions();

        quote_spanned!( self.contract.storage.span() =>
            #struct_def
            #allocate_using_impl
            #flush_impl
            #initialize_impl
            #methods
        )
    }
}
