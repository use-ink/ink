// Copyright 2018-2021 Parity Technologies (UK) Ltd.
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
use quote::quote;

/// Generates code for the ink! environment of the contract.
#[derive(From)]
pub struct Env<'a> {
    contract: &'a ir::Contract,
}

impl GenerateCode for Env<'_> {
    fn generate_code(&self) -> TokenStream2 {
        let env = self.contract.config().env();
        let storage_ident = self.contract.module().storage().ident();
        quote! {
            impl ::ink_lang::ContractEnv for #storage_ident {
                type Env = #env;
            }

            type Environment = <#storage_ident as ::ink_lang::ContractEnv>::Env;

            type AccountId = <<#storage_ident as ::ink_lang::ContractEnv>::Env as ::ink_env::Environment>::AccountId;
            type Balance = <<#storage_ident as ::ink_lang::ContractEnv>::Env as ::ink_env::Environment>::Balance;
            type Hash = <<#storage_ident as ::ink_lang::ContractEnv>::Env as ::ink_env::Environment>::Hash;
            type Timestamp = <<#storage_ident as ::ink_lang::ContractEnv>::Env as ::ink_env::Environment>::Timestamp;
            type BlockNumber = <<#storage_ident as ::ink_lang::ContractEnv>::Env as ::ink_env::Environment>::BlockNumber;
        }
    }
}
