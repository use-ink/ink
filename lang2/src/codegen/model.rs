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
use quote::{
    quote,
    quote_spanned,
};
use proc_macro2::TokenStream as TokenStream2;
use derive_more::From;

/// Generates code for the `ink_model` parts that dispatch constructors
/// and messages from the input and also handle the returning of data.
#[derive(From)]
pub struct Model<'a> {
    /// The contract to generate code for.
    contract: &'a Contract,
}

impl GenerateCode for Model<'_> {
    fn generate_code(&self) -> TokenStream2 {
        quote! {}
    }
}

/// Generates code for the `ink_model::Contract` instantiation procedure.
#[derive(From)]
pub struct Instantiate<'a> {
    contract: &'a Contract,
}

/// Generates code for the environmental types used by a contract.
#[derive(From)]
pub struct EnvTypes<'a> {
    /// The contract to generate code for.
    contract: &'a Contract,
}

impl GenerateCode for EnvTypes<'_> {
    fn generate_code(&self) -> TokenStream2 {
        let env_types = &self.contract.meta_info.env_types.ty;

        quote! {
            type Env = ink_core::env2::EnvImpl<#env_types>;

            type AccountId = <#env_types as ink_core::env2::EnvTypes>::AccountId;
            type Balance = <#env_types as ink_core::env2::EnvTypes>::Balance;
            type Hash = <#env_types as ink_core::env2::EnvTypes>::Hash;
            type Moment = <#env_types as ink_core::env2::EnvTypes>::Moment;
            type BlockNumber = <#env_types as ink_core::env2::EnvTypes>::BlockNumber;
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
                // #ident::instantiate().deploy().to_u32()
            }

            #[cfg(not(test))]
            #[no_mangle]
            fn call() -> u32 {
                // #ident::instantiate().dispatch().to_u32()
            }
        }
    }
}
