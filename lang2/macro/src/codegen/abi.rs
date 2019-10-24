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
    ir::Contract,
};
use derive_more::From;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

/// Generates code to generate the metadata of the contract.
#[derive(From)]
pub struct GenerateAbi<'a> {
    /// The contract to generate code for.
    contract: &'a Contract,
}

impl GenerateCode for GenerateAbi<'_> {
    fn generate_code(&self) -> TokenStream2 {
        let contract = self.generate_contract();
        let layout = self.generate_layout();

        quote! {
            #[cfg(feature = "ink-generate-abi")]
            const _: () = {
                impl ink_lang2::GenerateAbi for Flipper {
                    fn generate_abi() -> ink_abi::InkProject {
                        let contract: ink_abi::ContractSpec = {
                            #contract
                        };
                        let layout: ink_abi::StorageLayout = {
                            #layout
                        };
                        ink_abi::InkProject::new(layout, contract)
                    }
                }
            };
        }
    }
}

impl GenerateAbi<'_> {
    fn generate_constructors(&self) -> impl Iterator<Item = TokenStream2> {
        vec![].into_iter()
    }

    fn generate_messages(&self) -> impl Iterator<Item = TokenStream2> {
        vec![].into_iter()
    }

    fn generate_events(&self) -> impl Iterator<Item = TokenStream2> {
        vec![].into_iter()
    }

    fn generate_docs(&self) -> impl Iterator<Item = TokenStream2> {
        vec![].into_iter()
    }

    fn generate_contract(&self) -> TokenStream2 {
        let contract_ident_lit = self.contract.ident.to_string();

        let constructors = self.generate_constructors();
        let messages = self.generate_messages();
        let events = self.generate_events();
        let docs = self.generate_docs();

        quote! {
            ink_abi::ContractSpec::new(#contract_ident_lit)
                .constructors(vec![
                    #(#constructors ,)*
                ])
                .messages(vec![
                    #(#messages ,)*
                ])
                .events(vec![
                    #(#events ,)*
                ])
                .docs(vec![
                    #(#docs ,)*
                ])
                .done()
        }
    }

    fn generate_layout(&self) -> TokenStream2 {
        let contract_ident = &self.contract.storage.ident;
        quote! {
            unsafe {
                use ink_abi::HasLayout as _;
                use ink_core::storage::alloc::AllocateUsing as _;
                // We can use `ManuallyDrop` here and don't care for
                // unfreed memory since this function will generally be
                // called from within the `.ink` tool `abi-gen` and process
                // will end shortly after generating the ABI, so the
                // operating system will perform the cleanup immediately
                // for us.
                //
                // # Note
                //
                // This is not an optimization but to prevent panicking
                // because of a potential use of a dynamic environment
                // that uses storage data structures internally
                // that are going to panic upon `Drop` if not initialized
                // beforehand which would normally happen for contract
                // execution.
                core::mem::ManuallyDrop::new(
                    #contract_ident::allocate_using(&mut ink_core::storage::alloc::BumpAlloc::from_raw_parts(
                        ink_core::storage::Key([0x0; 32]),
                    ))
                )
                .layout()
            }
        }
    }
}
