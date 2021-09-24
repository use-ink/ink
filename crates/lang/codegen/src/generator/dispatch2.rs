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

use core::iter;

use crate::{
    generator,
    GenerateCode,
    GenerateCodeUsing as _,
};
use derive_more::From;
use ir::HexLiteral as _;
use proc_macro2::TokenStream as TokenStream2;
use quote::{
    quote,
    quote_spanned,
};
use syn::spanned::Spanned as _;

/// Generates code for the message and constructor dispatcher.
///
/// This code efficiently selects the dispatched ink! constructor or message
/// by inspecting the first four bytes (selector) of the given input bytes.
///
/// As this happens on every contract execution this code must be highly optimized.
/// For that purpose a so-called dispatch enum is being generated that has a
/// specialized `scale::Decode` implementation taking the first four bytes of
/// the input stream in order to identify the enum variant that it is going to
/// produce out of the rest of the input buffer.
///
/// The rest of the input buffer is then automatically decoded directly into the
/// expected input types of the respective ink! constructor or message.
#[derive(From)]
pub struct Dispatch<'a> {
    contract: &'a ir::Contract,
}
impl_as_ref_for_generator!(Dispatch);

impl GenerateCode for Dispatch<'_> {
    fn generate_code(&self) -> TokenStream2 {
        let cfg_not_as_dependency =
            self.generate_code_using::<generator::NotAsDependencyCfg>();
        let amount_dispatchables =
            self.generate_contract_amount_dispatchables_trait_impl();
        let contract_dispatchable_messages =
            self.generate_contract_dispatchable_messages_trait_impl();
        let contract_dispatchable_constructors =
            self.generate_contract_dispatchable_constructors_trait_impl();
        // let entry_points = self.generate_entry_points();
        // let dispatch_using_mode = self.generate_dispatch_using_mode();
        // let dispatch_trait_impl_namespaces = self.generate_trait_impl_namespaces();
        // let dispatch_trait_impls = self.generate_dispatch_trait_impls();
        // let message_dispatch_enum = self.generate_message_dispatch_enum();
        // let constructor_dispatch_enum = self.generate_constructor_dispatch_enum();
        quote! {
            // We do not generate contract dispatch code while the contract
            // is being tested or the contract is a dependency of another
            // since both resulting compilations do not require dispatching.
            #[cfg(not(test))]
            #cfg_not_as_dependency
            const _: () = {
                #amount_dispatchables
                #contract_dispatchable_messages
                #contract_dispatchable_constructors
                // #entry_points
                // #dispatch_using_mode
                // #dispatch_trait_impl_namespaces
                // #dispatch_trait_impls
                // #message_dispatch_enum
                // #constructor_dispatch_enum
            };
        }
    }
}

impl Dispatch<'_> {
    /// Generates code for the [`ink_lang::ContractDispatchables`] trait implementation.
    ///
    /// This trait implementation stores information of how many dispatchable
    /// ink! messages and ink! constructors there are for the ink! smart contract.
    fn generate_contract_amount_dispatchables_trait_impl(&self) -> TokenStream2 {
        let span = self.contract.module().storage().span();
        let storage_ident = self.contract.module().storage().ident();
        let count_messages = self
            .contract
            .module()
            .impls()
            .map(|item_impl| item_impl.iter_messages())
            .flatten()
            .count();
        let count_constructors = self
            .contract
            .module()
            .impls()
            .map(|item_impl| item_impl.iter_constructors())
            .flatten()
            .count();
        quote_spanned!(span=>
            impl ::ink_lang::ContractAmountDispatchables for #storage_ident {
                const MESSAGES: ::core::primitive::usize = #count_messages;
                const CONSTRUCTORS: ::core::primitive::usize = #count_constructors;
            }
        )
    }

    /// Generates code for the [`ink_lang::ContractDispatchableMessages`] trait implementation.
    ///
    /// This trait implementation stores the selector ID of each dispatchable
    /// ink! messages of the ink! smart contract.
    fn generate_contract_dispatchable_messages_trait_impl(&self) -> TokenStream2 {
        let span = self.contract.module().storage().span();
        let storage_ident = self.contract.module().storage().ident();
        let inherent_ids = self
            .contract
            .module()
            .impls()
            .filter(|item_impl| item_impl.trait_path().is_none())
            .map(|item_impl| item_impl.iter_messages())
            .flatten()
            .map(|message| {
                let span = message.span();
                let id = message
                    .composed_selector()
                    .into_be_u32()
                    .hex_padded_suffixed();
                quote_spanned!(span=> #id)
            });
        let trait_ids = self
            .contract
            .module()
            .impls()
            .filter_map(|item_impl| {
                item_impl
                    .trait_path()
                    .map(|trait_path| {
                        iter::repeat(trait_path).zip(item_impl.iter_messages())
                    })
            })
            .flatten()
            .map(|(trait_path, message)| {
                let local_id = message.local_id().hex_padded_suffixed();
                let span = message.span();
                quote_spanned!(span=>
                    {
                        ::core::primitive::u32::from_be_bytes(
                            <<::ink_lang::TraitDefinitionRegistry as #trait_path>::__ink_TraitInfo
                                as ::ink_lang::TraitMessageInfo<#local_id>>::SELECTOR
                        )
                    }
                )
            });
        quote_spanned!(span=>
            impl ::ink_lang::ContractDispatchableMessages<{
                <#storage_ident as ::ink_lang::ContractAmountDispatchables>::MESSAGES
            }> for #storage_ident {
                const IDS: [
                    ::core::primitive::u32;
                    <#storage_ident as ::ink_lang::ContractAmountDispatchables>::MESSAGES
                ] = [
                    #( #inherent_ids , )*
                    #( #trait_ids ),*
                ];
            }
        )
    }

    /// Generates code for the [`ink_lang::ContractDispatchableConstructors`] trait implementation.
    ///
    /// This trait implementation stores the selector ID of each dispatchable
    /// ink! constructor of the ink! smart contract.
    fn generate_contract_dispatchable_constructors_trait_impl(&self) -> TokenStream2 {
        let span = self.contract.module().storage().span();
        let storage_ident = self.contract.module().storage().ident();
        let constructor_ids = self
            .contract
            .module()
            .impls()
            .filter(|item_impl| item_impl.trait_path().is_none())
            .map(|item_impl| item_impl.iter_constructors())
            .flatten()
            .map(|message| {
                let span = message.span();
                let id = message
                    .composed_selector()
                    .into_be_u32()
                    .hex_padded_suffixed();
                quote_spanned!(span=> #id)
            });
        quote_spanned!(span=>
            impl ::ink_lang::ContractDispatchableConstructors<{
                <#storage_ident as ::ink_lang::ContractAmountDispatchables>::CONSTRUCTORS
            }> for #storage_ident {
                const IDS: [
                    ::core::primitive::u32;
                    <#storage_ident as ::ink_lang::ContractAmountDispatchables>::CONSTRUCTORS
                ] = [
                    #( #constructor_ids ),*
                ];
            }
        )
    }
}
