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
    ir::Contract,
    codegen::GenerateCode,
};
use derive_more::From;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned as _;

/// Generates code for the contract's event structs.
#[derive(From)]
pub struct Events<'a> {
    /// The contract to generate code for.
    contract: &'a Contract,
}

impl Events<'_> {
    fn generate_event_structs(&self) -> TokenStream2 {
        quote! {}
    }
}

impl GenerateCode for Events<'_> {
    fn generate_code(&self) -> TokenStream2 {
        let event_structs = self.generate_event_structs();

        quote_spanned!( self.contract.mod_token.span() =>
            #event_structs
        )
    }
}
