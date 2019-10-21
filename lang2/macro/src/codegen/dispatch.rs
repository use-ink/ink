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
    codegen::{
        GenerateCode,
        GenerateCodeUsing,
    },
    ir::Contract,
};
use derive_more::From;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, quote_spanned};

/// Generates code for the dispatch parts that dispatch constructors
/// and messages from the input and also handle the returning of data.
#[derive(From)]
pub struct Dispatch<'a> {
    /// The contract to generate code for.
    contract: &'a Contract,
}

impl<'a> GenerateCodeUsing for Dispatch<'a> {
    fn contract(&self) -> &Contract {
        self.contract
    }
}

impl GenerateCode for Dispatch<'_> {
    fn generate_code(&self) -> TokenStream2 {
        quote! {
            impl ink_lang2::Dispatch for Flipper {
                fn dispatch(mode: ink_lang2::DispatchMode) -> ink_lang2::DispatchRetCode {
                    let entry_points = self.generate_code_using::<EntryPoints>();

                    quote! {

                    }
                }
            }
        }
    }
}

/// Generates code for the entry points of a contract.
#[derive(From)]
pub struct EntryPoints<'a> {
    /// The contract to generate code for.
    contract: &'a Contract,
}

impl GenerateCode for EntryPoints<'_> {
    fn generate_code(&self) -> TokenStream2 {
        let ident = &self.contract.ident;

        quote_spanned! { ident.span() =>
            #[cfg(not(test))]
            #[no_mangle]
            fn deploy() -> u32 {
                0
            }

            #[cfg(not(test))]
            #[no_mangle]
            fn call() -> u32 {
                0
            }
        }
    }
}
