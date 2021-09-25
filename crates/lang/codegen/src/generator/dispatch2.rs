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
use ir::{
    Callable,
    HexLiteral as _,
};
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
        let contract_dispatchable_constructor_infos =
            self.generate_dispatchable_constructor_infos();
        let contract_dispatchable_messages_infos =
            self.generate_dispatchable_message_infos();
        quote! {
            #[cfg(not(test))]
            #cfg_not_as_dependency
            const _: () = {
                #amount_dispatchables
                #contract_dispatchable_messages
                #contract_dispatchable_constructors
                #contract_dispatchable_constructor_infos
                #contract_dispatchable_messages_infos
            };
        }
    }
}

impl Dispatch<'_> {
    /// Returns the number of dispatchable ink! constructors of the ink! smart contract.
    fn query_amount_constructors(&self) -> usize {
        self.contract
            .module()
            .impls()
            .map(|item_impl| item_impl.iter_constructors())
            .flatten()
            .count()
    }

    /// Returns the number of dispatchable ink! messages of the ink! smart contract.
    ///
    /// This includes inherent ink! messages as well as trait ink! messages.
    fn query_amount_messages(&self) -> usize {
        self.contract
            .module()
            .impls()
            .map(|item_impl| item_impl.iter_messages())
            .flatten()
            .count()
    }

    /// Generates code for the [`ink_lang::ContractDispatchables`] trait implementation.
    ///
    /// This trait implementation stores information of how many dispatchable
    /// ink! messages and ink! constructors there are for the ink! smart contract.
    fn generate_contract_amount_dispatchables_trait_impl(&self) -> TokenStream2 {
        let span = self.contract.module().storage().span();
        let storage_ident = self.contract.module().storage().ident();
        let count_messages = self.query_amount_messages();
        let count_constructors = self.query_amount_constructors();
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
                            <<::ink_lang::InkTraitDefinitionRegistry<<#storage_ident as ::ink_lang::ContractEnv>::Env>
                                as #trait_path>::__ink_TraitInfo
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
            .map(|item_impl| item_impl.iter_constructors())
            .flatten()
            .map(|constructor| {
                let span = constructor.span();
                let id = constructor
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

    /// Generate code for the [`ink_lang::DispatchableConstructorInfo`] trait implementations.
    ///
    /// These trait implementations store relevant dispatch information for every
    /// dispatchable ink! constructor of the ink! smart contract.
    fn generate_dispatchable_constructor_infos(&self) -> TokenStream2 {
        let span = self.contract.module().storage().span();
        let storage_ident = self.contract.module().storage().ident();
        let constructor_infos = self
            .contract
            .module()
            .impls()
            .map(|item_impl| item_impl.iter_constructors())
            .flatten()
            .map(|constructor| {
                let constructor_span = constructor.span();
                let constructor_ident = constructor.ident();
                let selector_id = constructor.composed_selector().into_be_u32().hex_padded_suffixed();
                let selector_bytes = constructor.composed_selector().hex_lits();
                let input_bindings = generator::input_bindings(constructor.inputs());
                let input_tuple_type = generator::input_types_tuple(constructor.inputs());
                let input_tuple_bindings = generator::input_bindings_tuple(constructor.inputs());
                quote_spanned!(constructor_span=>
                    impl ::ink_lang::DispatchableConstructorInfo<#selector_id> for #storage_ident {
                        type Input = #input_tuple_type;
                        type Storage = #storage_ident;

                        const CALLABLE: fn(Self::Input) -> Self::Storage = |#input_tuple_bindings| {
                            #storage_ident::#constructor_ident( #( #input_bindings ),* )
                        };
                        const SELECTOR: [::core::primitive::u8; 4usize] = [ #( #selector_bytes ),* ];
                        const LABEL: &'static ::core::primitive::str = ::core::stringify!(#constructor_ident);
                    }
                )
            });
        quote_spanned!(span=>
            #( #constructor_infos )*
        )
    }

    /// Generate code for the [`ink_lang::DispatchableConstructorInfo`] trait implementations.
    ///
    /// These trait implementations store relevant dispatch information for every
    /// dispatchable ink! constructor of the ink! smart contract.
    fn generate_dispatchable_message_infos(&self) -> TokenStream2 {
        let span = self.contract.module().storage().span();
        let storage_ident = self.contract.module().storage().ident();
        let inherent_message_infos = self
            .contract
            .module()
            .impls()
            .filter(|item_impl| item_impl.trait_path().is_none())
            .map(|item_impl| item_impl.iter_messages())
            .flatten()
            .map(|message| {
                let message_span = message.span();
                let message_ident = message.ident();
                let payable = message.is_payable();
                let mutates = message.receiver().is_ref_mut();
                let selector_id = message.composed_selector().into_be_u32().hex_padded_suffixed();
                let selector_bytes = message.composed_selector().hex_lits();
                let output_tuple_type = message
                    .output()
                    .map(quote::ToTokens::to_token_stream)
                    .unwrap_or_else(|| quote! { () });
                let input_bindings = generator::input_bindings(message.inputs());
                let input_tuple_type = generator::input_types_tuple(message.inputs());
                let input_tuple_bindings = generator::input_bindings_tuple(message.inputs());
                quote_spanned!(message_span=>
                    impl ::ink_lang::DispatchableMessageInfo<#selector_id> for #storage_ident {
                        type Input = #input_tuple_type;
                        type Output = #output_tuple_type;
                        type Storage = #storage_ident;

                        const CALLABLE: fn(&mut Self::Storage, Self::Input) -> Self::Output =
                            |storage, #input_tuple_bindings| {
                                #storage_ident::#message_ident( storage #( , #input_bindings )* )
                            };
                        const SELECTOR: [::core::primitive::u8; 4usize] = [ #( #selector_bytes ),* ];
                        const PAYABLE: ::core::primitive::bool = #payable;
                        const MUTATES: ::core::primitive::bool = #mutates;
                        const LABEL: &'static ::core::primitive::str = ::core::stringify!(#message_ident);
                    }
                )
            });
        let trait_message_infos = self
            .contract
            .module()
            .impls()
            .filter_map(|item_impl| {
                item_impl
                    .trait_path()
                    .map(|trait_path| {
                        let trait_ident = item_impl.trait_ident().expect(
                            "must have an ink! trait identifier if it is an ink! trait implementation"
                        );
                        iter::repeat((trait_ident, trait_path)).zip(item_impl.iter_messages())
                    })
            })
            .flatten()
            .map(|((trait_ident, trait_path), message)| {
                let message_span = message.span();
                let message_ident = message.ident();
                let mutates = message.receiver().is_ref_mut();
                let local_id = message.local_id().hex_padded_suffixed();
                let payable = quote! {{
                    <<::ink_lang::InkTraitDefinitionRegistry<<#storage_ident as ::ink_lang::ContractEnv>::Env>
                        as #trait_path>::__ink_TraitInfo
                        as ::ink_lang::TraitMessageInfo<#local_id>>::PAYABLE
                }};
                let selector = quote! {{
                    <<::ink_lang::InkTraitDefinitionRegistry<<#storage_ident as ::ink_lang::ContractEnv>::Env>
                        as #trait_path>::__ink_TraitInfo
                        as ::ink_lang::TraitMessageInfo<#local_id>>::SELECTOR
                }};
                let selector_id = quote! {{
                    ::core::primitive::u32::from_be_bytes(#selector)
                }};
                let output_tuple_type = message
                    .output()
                    .map(quote::ToTokens::to_token_stream)
                    .unwrap_or_else(|| quote! { () });
                let input_bindings = generator::input_bindings(message.inputs());
                let input_tuple_type = generator::input_types_tuple(message.inputs());
                let input_tuple_bindings = generator::input_bindings_tuple(message.inputs());
                let label = format!("{}::{}", trait_ident, message_ident);
                quote_spanned!(message_span=>
                    impl ::ink_lang::DispatchableMessageInfo<#selector_id> for #storage_ident {
                        type Input = #input_tuple_type;
                        type Output = #output_tuple_type;
                        type Storage = #storage_ident;

                        const CALLABLE: fn(&mut Self::Storage, Self::Input) -> Self::Output =
                            |storage, #input_tuple_bindings| {
                                #storage_ident::#message_ident( storage #( , #input_bindings )* )
                            };
                        const SELECTOR: [::core::primitive::u8; 4usize] = #selector;
                        const PAYABLE: ::core::primitive::bool = #payable;
                        const MUTATES: ::core::primitive::bool = #mutates;
                        const LABEL: &'static ::core::primitive::str = #label;
                    }
                )
            });
        quote_spanned!(span=>
            #( #inherent_message_infos )*
            #( #trait_message_infos )*
        )
    }
}
