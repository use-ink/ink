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

impl Storage<'_> {
    /// Generates the storage struct definition.
    fn generate_struct(&self) -> TokenStream2 {
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

                fn initialize(&mut self, args: Self::Args) {
                    #(
                        self.#fields.try_default_initialize();
                    )*
                    self.env.try_default_initialize();
                }
            }
        )
    }
}

impl GenerateCode for Storage<'_> {
    fn generate_code(&self) -> TokenStream2 {
        let struct_def = self.generate_struct();
        let allocate_using_impl = self.allocate_using_impl();
        let flush_impl = self.flush_impl();
        let initialize_impl = self.initialize_impl();

        quote_spanned!( self.contract.storage.span() =>
            #struct_def
            #allocate_using_impl
            #flush_impl
            #initialize_impl
        )
    }
}
