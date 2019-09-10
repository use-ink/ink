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

/// Generates code for the `ink_model` parts that dispatch constructors
/// and messages from the input and also handle the returning of data.
pub struct Model<'a> {
    /// The contract to generate code for.
    contract: &'a Contract,
}

impl GenerateCode for Model<'_> {
    fn generate_code(&self) -> TokenStream2 {
        quote! {}
    }
}

/// Generates code for the environmental types used by a contract.
pub struct MetaTypes<'a> {
    /// The contract to generate code for.
    contract: &'a Contract,
}

impl GenerateCode for MetaTypes<'_> {
    fn generate_code(&self) -> TokenStream2 {
        let env_types = &self.contract.meta_info.env_types.ty;

        quote! {
            mod env_types {
                use super::*;
                use ink_core::env::{ContractEnv, EnvTypes};

                pub type AccountId = <#env_types as EnvTypes>::AccountId;
                pub type Balance = <#env_types as EnvTypes>::Balance;
                pub type Hash = <#env_types as EnvTypes>::Hash;
                pub type Moment = <#env_types as EnvTypes>::Moment;
                pub type BlockNumber = <#env_types as EnvTypes>::BlockNumber;
            }

            type Env = ink_core::env::ContractEnv<#env_types>;
            use env_types::{
                AccountId,
                Balance,
                Hash,
                Moment,
                BlockNumber,
            };
        }
    }
}

/// Generates code for the entry points of a contract.
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
                #ident::instantiate().deploy().to_u32()
            }

            #[cfg(not(test))]
            #[no_mangle]
            fn call() -> u32 {
                #ident::instantiate().dispatch().to_u32()
            }
        }
    }
}
