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
use quote::quote;

pub use crate::{
    codegen::{
        abi::GenerateAbi,
        cross_calling::CrossCalling,
        dispatch::Dispatch,
        env_types::EnvTypes,
        events::{
            EventHelpers,
            EventStructs,
        },
        storage::Storage,
        GenerateCode,
        GenerateCodeUsing,
    },
    ir::Contract,
};

/// Generates code for the entirety of the ink! contract.
#[derive(From)]
pub struct ContractModule<'a> {
    /// The contract to generate code for.
    contract: &'a Contract,
}

impl<'a> GenerateCodeUsing for ContractModule<'a> {
    fn contract(&self) -> &Contract {
        self.contract
    }
}

impl GenerateCode for ContractModule<'_> {
    /// Generates ink! contract code.
    fn generate_code(&self) -> TokenStream2 {
        let ident = &self.contract.ident;

        let env_types = self.generate_code_using::<EnvTypes>();
        let storage = self.generate_code_using::<Storage>();
        let dispatch = self.generate_code_using::<Dispatch>();
        let generate_abi = self.generate_code_using::<GenerateAbi>();
        let event_helpers = self.generate_code_using::<EventHelpers>();
        let event_structs = self.generate_code_using::<EventStructs>();
        let cross_calling = self.generate_code_using::<CrossCalling>();
        let non_ink_items = &self.contract.non_ink_items;

        quote! {
            mod #ident {
                #env_types
                #storage
                #event_helpers
                #dispatch
                #generate_abi
                #cross_calling
                #event_structs
                #( #non_ink_items )*
            }
        }
    }
}

impl GenerateCode for Contract {
    fn generate_code(&self) -> TokenStream2 {
        ContractModule::from(self).generate_code()
    }
}
