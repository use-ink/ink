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

use derive_more::From;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

pub use crate::{
    codegen::{
        abi::GenerateAbi,
        cross_calling::{
            CrossCalling,
            CrossCallingConflictCfg,
        },
        dispatch::Dispatch,
        env_types::EnvTypes,
        events::{
            EventHelpers,
            EventImports,
            EventStructs,
        },
        storage::Storage,
        testable::TestWrapper,
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
        let storage_ident = &self.contract.storage.ident;

        let conflic_depedency_cfg = self.generate_code_using::<CrossCallingConflictCfg>();
        let env_types = self.generate_code_using::<EnvTypes>();
        let storage = self.generate_code_using::<Storage>();
        let dispatch = self.generate_code_using::<Dispatch>();
        let generate_abi = self.generate_code_using::<GenerateAbi>();
        let event_helpers = self.generate_code_using::<EventHelpers>();
        let event_structs = self.generate_code_using::<EventStructs>();
        let event_imports = self.generate_code_using::<EventImports>();
        let test_wrapper = self.generate_code_using::<TestWrapper>();
        let cross_calling = self.generate_code_using::<CrossCalling>();
        let non_ink_items = &self.contract.non_ink_items;

        quote! {
            mod #ident {
                use super::*;

                #env_types

                // Private struct and other type definitions.
                mod __ink_private {
                    use super::*;
                    #event_imports

                    #storage
                    #event_helpers
                    #dispatch
                    #generate_abi
                    #test_wrapper
                    #cross_calling
                }

                #[cfg(all(test, feature = "test-env"))]
                pub type #storage_ident = self::__ink_private::TestableStorageAndEnv;

                #[cfg(not(all(test, feature = "test-env")))]
                #conflic_depedency_cfg
                pub type #storage_ident = self::__ink_private::StorageAndEnv;

                #[cfg(feature = "ink-as-dependency")]
                pub type #storage_ident = self::__ink_private::StorageAsDependency;

                #event_structs

                #(
                    #non_ink_items
                )*
            }

            // Only re-export if we want to generate the ABI.
            // We should rethink this approach, it isn't a good
            // idea to generate code outside of the scope of the
            // given ink! module.
            #[cfg(feature = "ink-generate-abi")]
            #conflic_depedency_cfg
            pub use crate::#ident::#storage_ident;
        }
    }
}

impl GenerateCode for Contract {
    fn generate_code(&self) -> TokenStream2 {
        ContractModule::from(self).generate_code()
    }
}
