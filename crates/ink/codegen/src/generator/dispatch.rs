// Copyright 2018-2022 Parity Technologies (UK) Ltd.
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
};
use derive_more::From;
use ir::{
    Callable,
    HexLiteral as _,
};
use proc_macro2::TokenStream as TokenStream2;
use quote::{
    format_ident,
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
        let mut constructor_spans = Vec::new();
        let mut message_spans = Vec::new();

        let amount_dispatchables =
            self.generate_contract_amount_dispatchables_trait_impl();
        let contract_dispatchable_messages =
            self.generate_contract_dispatchable_messages_trait_impl(&mut message_spans);
        let contract_dispatchable_constructors = self
            .generate_contract_dispatchable_constructors_trait_impl(
                &mut constructor_spans,
            );
        let contract_dispatchable_constructor_infos =
            self.generate_dispatchable_constructor_infos();
        let contract_dispatchable_messages_infos =
            self.generate_dispatchable_message_infos();
        let constructor_decoder_type =
            self.generate_constructor_decoder_type(&constructor_spans);
        let message_decoder_type = self.generate_message_decoder_type(&message_spans);
        let entry_points = self.generate_entry_points(&constructor_spans, &message_spans);
        quote! {
            #amount_dispatchables
            #contract_dispatchable_messages
            #contract_dispatchable_constructors
            #contract_dispatchable_constructor_infos
            #contract_dispatchable_messages_infos
            #constructor_decoder_type
            #message_decoder_type
            #entry_points
        }
    }
}

impl Dispatch<'_> {
    /// Returns the number of dispatchable ink! constructors of the ink! smart contract.
    fn query_amount_constructors(&self) -> usize {
        self.contract
            .module()
            .impls()
            .flat_map(|item_impl| item_impl.iter_constructors())
            .count()
    }

    /// Returns the number of dispatchable ink! messages of the ink! smart contract.
    ///
    /// This includes inherent ink! messages as well as trait ink! messages.
    fn query_amount_messages(&self) -> usize {
        self.contract
            .module()
            .impls()
            .flat_map(|item_impl| item_impl.iter_messages())
            .count()
    }

    /// Returns the index of the ink! message which has a wildcard selector, if existent.
    fn query_wildcard_message(&self) -> Option<usize> {
        self.contract
            .module()
            .impls()
            .flat_map(|item_impl| item_impl.iter_messages())
            .position(|item| item.has_wildcard_selector())
    }

    /// Returns the index of the ink! constructor which has a wildcard selector, if existent.
    fn query_wildcard_constructor(&self) -> Option<usize> {
        self.contract
            .module()
            .impls()
            .flat_map(|item_impl| item_impl.iter_constructors())
            .position(|item| item.has_wildcard_selector())
    }

    /// Generates code for the [`ink::ContractDispatchables`] trait implementation.
    ///
    /// This trait implementation stores information of how many dispatchable
    /// ink! messages and ink! constructors there are for the ink! smart contract.
    fn generate_contract_amount_dispatchables_trait_impl(&self) -> TokenStream2 {
        let span = self.contract.module().storage().span();
        let storage_ident = self.contract.module().storage().ident();
        let count_messages = self.query_amount_messages();
        let count_constructors = self.query_amount_constructors();
        quote_spanned!(span=>
            impl ::ink::reflect::ContractAmountDispatchables for #storage_ident {
                const MESSAGES: ::core::primitive::usize = #count_messages;
                const CONSTRUCTORS: ::core::primitive::usize = #count_constructors;
            }
        )
    }

    /// Generates code for the [`ink::ContractDispatchableMessages`] trait implementation.
    ///
    /// This trait implementation stores the selector ID of each dispatchable
    /// ink! messages of the ink! smart contract.
    fn generate_contract_dispatchable_messages_trait_impl(
        &self,
        message_spans: &mut Vec<proc_macro2::Span>,
    ) -> TokenStream2 {
        let span = self.contract.module().storage().span();
        let storage_ident = self.contract.module().storage().ident();
        let message_ids = self
            .contract
            .module()
            .impls()
            .flat_map(|item_impl| {
                iter::repeat(item_impl.trait_path()).zip(item_impl.iter_messages())
            })
            .map(|(trait_path, message)| {
                let span = message.span();
                message_spans.push(span);

                if let Some(trait_path) = trait_path {
                    let local_id = message.local_id().hex_padded_suffixed();
                    quote_spanned!(span=>
                        {
                            ::core::primitive::u32::from_be_bytes(
                                <<::ink::reflect::TraitDefinitionRegistry<<#storage_ident as ::ink::env::ContractEnv>::Env>
                                    as #trait_path>::__ink_TraitInfo
                                    as ::ink::reflect::TraitMessageInfo<#local_id>>::SELECTOR
                            )
                        }
                    )
                } else {
                    let id = message
                        .composed_selector()
                        .into_be_u32()
                        .hex_padded_suffixed();
                    quote_spanned!(span=> #id)
                }
            })
            .collect::<Vec<_>>();
        quote_spanned!(span=>
            impl ::ink::reflect::ContractDispatchableMessages<{
                <#storage_ident as ::ink::reflect::ContractAmountDispatchables>::MESSAGES
            }> for #storage_ident {
                const IDS: [
                    ::core::primitive::u32;
                    <#storage_ident as ::ink::reflect::ContractAmountDispatchables>::MESSAGES
                ] = [
                    #( #message_ids , )*
                ];
            }
        )
    }

    /// Generates code for the [`ink::ContractDispatchableConstructors`] trait implementation.
    ///
    /// This trait implementation stores the selector ID of each dispatchable
    /// ink! constructor of the ink! smart contract.
    fn generate_contract_dispatchable_constructors_trait_impl(
        &self,
        constructor_spans: &mut Vec<proc_macro2::Span>,
    ) -> TokenStream2 {
        let span = self.contract.module().storage().span();
        let storage_ident = self.contract.module().storage().ident();
        let constructor_ids = self
            .contract
            .module()
            .impls()
            .flat_map(|item_impl| item_impl.iter_constructors())
            .map(|constructor| {
                let span = constructor.span();
                constructor_spans.push(span);
                let id = constructor
                    .composed_selector()
                    .into_be_u32()
                    .hex_padded_suffixed();
                quote_spanned!(span=> #id)
            });
        quote_spanned!(span=>
            impl ::ink::reflect::ContractDispatchableConstructors<{
                <#storage_ident as ::ink::reflect::ContractAmountDispatchables>::CONSTRUCTORS
            }> for #storage_ident {
                const IDS: [
                    ::core::primitive::u32;
                    <#storage_ident as ::ink::reflect::ContractAmountDispatchables>::CONSTRUCTORS
                ] = [
                    #( #constructor_ids ),*
                ];
            }
        )
    }

    /// Generate code for the [`ink::DispatchableConstructorInfo`] trait implementations.
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
            .flat_map(|item_impl| item_impl.iter_constructors())
            .map(|constructor| {
                let constructor_span = constructor.span();
                let constructor_ident = constructor.ident();
                let payable = constructor.is_payable();
                let selector_id = constructor.composed_selector().into_be_u32().hex_padded_suffixed();
                let selector_bytes = constructor.composed_selector().hex_lits();
                let output_type = constructor.output().map(quote::ToTokens::to_token_stream)
                    .unwrap_or_else(|| quote! { () });
                let input_bindings = generator::input_bindings(constructor.inputs());
                let input_tuple_type = generator::input_types_tuple(constructor.inputs());
                let input_tuple_bindings = generator::input_bindings_tuple(constructor.inputs());
                let constructor_return_type = quote_spanned!(constructor_span=>
                    <::ink::reflect::ConstructorOutputValue<#output_type>
                        as ::ink::reflect::ConstructorOutput<#storage_ident>>
                );
                quote_spanned!(constructor_span=>
                    impl ::ink::reflect::DispatchableConstructorInfo<#selector_id> for #storage_ident {
                        type Input = #input_tuple_type;
                        type Output = #output_type;
                        type Storage = #storage_ident;
                        type Error = #constructor_return_type::Error;
                        const IS_RESULT: ::core::primitive::bool = #constructor_return_type::IS_RESULT;

                        const CALLABLE: fn(Self::Input) -> Self::Output = |#input_tuple_bindings| {
                            #storage_ident::#constructor_ident(#( #input_bindings ),* )
                        };
                        const PAYABLE: ::core::primitive::bool = #payable;
                        const SELECTOR: [::core::primitive::u8; 4usize] = [ #( #selector_bytes ),* ];
                        const LABEL: &'static ::core::primitive::str = ::core::stringify!(#constructor_ident);
                    }
                )
            });
        quote_spanned!(span=>
            #( #constructor_infos )*
        )
    }

    /// Generate code for the [`ink::DispatchableMessageInfo`] trait implementations.
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
            .flat_map(|item_impl| item_impl.iter_messages())
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
                    impl ::ink::reflect::DispatchableMessageInfo<#selector_id> for #storage_ident {
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
                    <<::ink::reflect::TraitDefinitionRegistry<<#storage_ident as ::ink::env::ContractEnv>::Env>
                        as #trait_path>::__ink_TraitInfo
                        as ::ink::reflect::TraitMessageInfo<#local_id>>::PAYABLE
                }};
                let selector = quote! {{
                    <<::ink::reflect::TraitDefinitionRegistry<<#storage_ident as ::ink::env::ContractEnv>::Env>
                        as #trait_path>::__ink_TraitInfo
                        as ::ink::reflect::TraitMessageInfo<#local_id>>::SELECTOR
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
                let label = format!("{trait_ident}::{message_ident}");
                quote_spanned!(message_span=>
                    impl ::ink::reflect::DispatchableMessageInfo<#selector_id> for #storage_ident {
                        type Input = #input_tuple_type;
                        type Output = #output_tuple_type;
                        type Storage = #storage_ident;

                        const CALLABLE: fn(&mut Self::Storage, Self::Input) -> Self::Output =
                            |storage, #input_tuple_bindings| {
                                <#storage_ident as #trait_path>::#message_ident( storage #( , #input_bindings )* )
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

    /// Generates code for the entry points of the root ink! smart contract.
    ///
    /// This generates the `deploy` and `call` functions with which the smart
    /// contract runtime mainly interacts with the ink! smart contract.
    fn generate_entry_points(
        &self,
        constructor_spans: &[proc_macro2::Span],
        message_spans: &[proc_macro2::Span],
    ) -> TokenStream2 {
        let span = self.contract.module().storage().span();
        let storage_ident = self.contract.module().storage().ident();
        let any_constructor_accept_payment =
            self.any_constructor_accepts_payment_expr(constructor_spans);
        let any_message_accept_payment =
            self.any_message_accepts_payment_expr(message_spans);
        quote_spanned!(span=>
            #[cfg(not(test))]
            #[cfg(not(feature = "ink-as-dependency"))]
            const _: () = {
                #[no_mangle]
                fn deploy() {
                    <#storage_ident as ::ink::env::contract::Entrypoint>::deploy()
                }

                #[no_mangle]
                fn call() {
                    <#storage_ident as ::ink::env::contract::Entrypoint>::call()
                }
            };

            impl ::ink::env::contract::Entrypoint for #storage_ident {
                #[allow(clippy::nonminimal_bool)]
                fn deploy() {
                    if !#any_constructor_accept_payment {
                        ::ink::codegen::deny_payment::<<#storage_ident as ::ink::env::ContractEnv>::Env>()
                            .unwrap_or_else(|error| ::core::panic!("{}", error))
                    }

                    let dispatchable = match ::ink::env::decode_input::<
                        <#storage_ident as ::ink::reflect::ContractConstructorDecoder>::Type,
                    >() {
                        ::core::result::Result::Ok(decoded_dispatchable) => {
                            decoded_dispatchable
                        }
                        ::core::result::Result::Err(_decoding_error) => {
                            let error = ::ink::ConstructorResult::Err(::ink::LangError::CouldNotReadInput);

                            // At this point we're unable to set the `Ok` variant to be the any "real"
                            // constructor output since we were unable to figure out what the caller wanted
                            // to dispatch in the first place, so we set it to `()`.
                            //
                            // This is okay since we're going to only be encoding the `Err` variant
                            // into the output buffer anyways.
                            ::ink::env::return_value::<::ink::ConstructorResult<()>>(
                                ::ink::env::ReturnFlags::new_with_reverted(true),
                                &error,
                            );
                            ::core::panic!("execute_constructor reverted");
                        }
                    };

                    <<#storage_ident as ::ink::reflect::ContractConstructorDecoder>::Type
                        as ::ink::reflect::ExecuteDispatchable>::execute_dispatchable(dispatchable)
                    .unwrap_or_else(|error| {
                        ::core::panic!("dispatching ink! message failed: {}", error)
                    })
                }

                #[allow(clippy::nonminimal_bool)]
                fn call() {
                    if !#any_message_accept_payment {
                        ::ink::codegen::deny_payment::<<#storage_ident as ::ink::env::ContractEnv>::Env>()
                            .unwrap_or_else(|error| ::core::panic!("{}", error))
                    }

                    let dispatchable = match ::ink::env::decode_input::<
                        <#storage_ident as ::ink::reflect::ContractMessageDecoder>::Type,
                    >() {
                        ::core::result::Result::Ok(decoded_dispatchable) => {
                            decoded_dispatchable
                        }
                        ::core::result::Result::Err(_decoding_error) => {
                            let error = ::ink::MessageResult::Err(::ink::LangError::CouldNotReadInput);

                            // At this point we're unable to set the `Ok` variant to be the any "real"
                            // message output since we were unable to figure out what the caller wanted
                            // to dispatch in the first place, so we set it to `()`.
                            //
                            // This is okay since we're going to only be encoding the `Err` variant
                            // into the output buffer anyways.
                            ::ink::env::return_value::<::ink::MessageResult<()>>(
                                ::ink::env::ReturnFlags::new_with_reverted(true),
                                &error,
                            );
                            ::core::panic!("execute_message reverted");
                        }
                    };

                    <<#storage_ident as ::ink::reflect::ContractMessageDecoder>::Type
                        as ::ink::reflect::ExecuteDispatchable>::execute_dispatchable(dispatchable)
                    .unwrap_or_else(|error| {
                        ::core::panic!("dispatching ink! message failed: {}", error)
                    })
                }
            }
        )
    }

    /// Generates code for the ink! constructor decoder type of the ink! smart contract.
    ///
    /// This type can be used in order to decode the input bytes received by a call to `deploy`
    /// into one of the available dispatchable ink! constructors and their arguments.
    fn generate_constructor_decoder_type(
        &self,
        constructor_spans: &[proc_macro2::Span],
    ) -> TokenStream2 {
        assert_eq!(constructor_spans.len(), self.query_amount_constructors());

        /// Expands into the token sequence to represent the
        /// input type of the ink! constructor at the given index.
        fn expand_constructor_input(
            span: proc_macro2::Span,
            storage_ident: &syn::Ident,
            constructor_index: usize,
        ) -> TokenStream2 {
            quote_spanned!(span=>
                <#storage_ident as ::ink::reflect::DispatchableConstructorInfo<{
                    <#storage_ident as ::ink::reflect::ContractDispatchableConstructors<{
                        <#storage_ident as ::ink::reflect::ContractAmountDispatchables>::CONSTRUCTORS
                    }>>::IDS[#constructor_index]
                }>>::Input
            )
        }

        /// Returns the n-th ink! constructor identifier for the decoder type.
        fn constructor_variant_ident(n: usize) -> syn::Ident {
            quote::format_ident!("Constructor{}", n)
        }

        let span = self.contract.module().storage().span();
        let storage_ident = self.contract.module().storage().ident();
        let count_constructors = self.query_amount_constructors();
        let constructors_variants = (0..count_constructors).map(|index| {
            let constructor_span = constructor_spans[index];
            let constructor_ident = constructor_variant_ident(index);
            let constructor_input =
                expand_constructor_input(constructor_span, storage_ident, index);
            quote_spanned!(constructor_span=>
                #constructor_ident(#constructor_input)
            )
        });

        let constructor_selector = (0..count_constructors).map(|index| {
            let const_ident = format_ident!("CONSTRUCTOR_{}", index);
            quote_spanned!(span=>
                const #const_ident: [::core::primitive::u8; 4usize] = <#storage_ident as ::ink::reflect::DispatchableConstructorInfo<{
                    <#storage_ident as ::ink::reflect::ContractDispatchableConstructors<{
                        <#storage_ident as ::ink::reflect::ContractAmountDispatchables>::CONSTRUCTORS
                    }>>::IDS[#index]
                }>>::SELECTOR;
            )
        });

        let constructor_match = (0..count_constructors).map(|index| {
            let constructor_span = constructor_spans[index];
            let constructor_ident = constructor_variant_ident(index);
            let const_ident = format_ident!("CONSTRUCTOR_{}", index);
            let constructor_input = expand_constructor_input(constructor_span, storage_ident, index);
            quote_spanned!(constructor_span=>
                #const_ident => {
                    ::core::result::Result::Ok(Self::#constructor_ident(
                        <#constructor_input as ::scale::Decode>::decode(input)
                            .map_err(|_| ::ink::reflect::DispatchError::InvalidParameters)?
                    ))
                }
            )
        });
        let possibly_wildcard_selector_constructor = match self
            .query_wildcard_constructor()
        {
            Some(wildcard_index) => {
                let constructor_span = constructor_spans[wildcard_index];
                let constructor_ident = constructor_variant_ident(wildcard_index);
                let constructor_input = expand_constructor_input(
                    constructor_span,
                    storage_ident,
                    wildcard_index,
                );
                quote! {
                    ::core::result::Result::Ok(Self::#constructor_ident(
                        <#constructor_input as ::scale::Decode>::decode(input)
                            .map_err(|_| ::ink::reflect::DispatchError::InvalidParameters)?
                    ))
                }
            }
            None => {
                quote! {
                    ::core::result::Result::Err(::ink::reflect::DispatchError::UnknownSelector)
                }
            }
        };
        let any_constructor_accept_payment =
            self.any_constructor_accepts_payment_expr(constructor_spans);

        let constructor_execute = (0..count_constructors).map(|index| {
            let constructor_span = constructor_spans[index];
            let constructor_ident = constructor_variant_ident(index);
            let constructor_callable = quote_spanned!(constructor_span=>
                <#storage_ident as ::ink::reflect::DispatchableConstructorInfo<{
                    <#storage_ident as ::ink::reflect::ContractDispatchableConstructors<{
                        <#storage_ident as ::ink::reflect::ContractAmountDispatchables>::CONSTRUCTORS
                    }>>::IDS[#index]
                }>>::CALLABLE
            );
            let constructor_output = quote_spanned!(constructor_span=>
                <#storage_ident as ::ink::reflect::DispatchableConstructorInfo<{
                    <#storage_ident as ::ink::reflect::ContractDispatchableConstructors<{
                        <#storage_ident as ::ink::reflect::ContractAmountDispatchables>::CONSTRUCTORS
                    }>>::IDS[#index]
                }>>::Output
            );
            let deny_payment = quote_spanned!(constructor_span=>
                !<#storage_ident as ::ink::reflect::DispatchableConstructorInfo<{
                    <#storage_ident as ::ink::reflect::ContractDispatchableConstructors<{
                        <#storage_ident as ::ink::reflect::ContractAmountDispatchables>::CONSTRUCTORS
                    }>>::IDS[#index]
                }>>::PAYABLE
            );
            let constructor_value = quote_spanned!(constructor_span=>
                <::ink::reflect::ConstructorOutputValue<#constructor_output>
                    as ::ink::reflect::ConstructorOutput::<#storage_ident>>
            );

            quote_spanned!(constructor_span=>
                Self::#constructor_ident(input) => {
                    if #any_constructor_accept_payment && #deny_payment {
                        ::ink::codegen::deny_payment::<
                            <#storage_ident as ::ink::env::ContractEnv>::Env>()?;
                    }

                    let result: #constructor_output = #constructor_callable(input);
                    let output_value = ::ink::reflect::ConstructorOutputValue::new(result);
                    let output_result = #constructor_value::as_result(&output_value);

                    if let ::core::result::Result::Ok(contract) = output_result.as_ref() {
                        ::ink::env::set_contract_storage::<::ink::primitives::Key, #storage_ident>(
                            &<#storage_ident as ::ink::storage::traits::StorageKey>::KEY,
                            contract,
                        );
                    }

                    ::core::result::Result::Ok(::ink::env::return_value::<
                        ::ink::ConstructorResult<
                            ::core::result::Result<(), &#constructor_value::Error>
                        >,
                    >(
                        ::ink::env::ReturnFlags::new_with_reverted(output_result.is_err()),
                        // Currently no `LangError`s are raised at this level of the
                        // dispatch logic so `Ok` is always returned to the caller.
                        &::ink::ConstructorResult::Ok(output_result.map(|_| ())),
                    ))
                }
            )
        });
        quote_spanned!(span=>
            const _: () = {
                #[allow(non_camel_case_types)]
                pub enum __ink_ConstructorDecoder {
                    #( #constructors_variants ),*
                }

                impl ::ink::reflect::DecodeDispatch for __ink_ConstructorDecoder {
                    fn decode_dispatch<I>(input: &mut I)
                        -> ::core::result::Result<Self, ::ink::reflect::DispatchError>
                    where
                        I: ::scale::Input,
                    {
                        #(
                            #constructor_selector
                        )*
                        match <[::core::primitive::u8; 4usize] as ::scale::Decode>::decode(input)
                            .map_err(|_| ::ink::reflect::DispatchError::InvalidSelector)?
                        {
                            #( #constructor_match , )*
                            _invalid => #possibly_wildcard_selector_constructor
                        }
                    }
                }

                impl ::scale::Decode for __ink_ConstructorDecoder {
                    fn decode<I>(input: &mut I) -> ::core::result::Result<Self, ::scale::Error>
                    where
                        I: ::scale::Input,
                    {
                        <Self as ::ink::reflect::DecodeDispatch>::decode_dispatch(input)
                            .map_err(::core::convert::Into::into)
                    }
                }

                impl ::ink::reflect::ExecuteDispatchable for __ink_ConstructorDecoder {
                    #[allow(clippy::nonminimal_bool)]
                    fn execute_dispatchable(self) -> ::core::result::Result<(), ::ink::reflect::DispatchError> {
                        match self {
                            #( #constructor_execute ),*
                        }
                    }
                }

                impl ::ink::reflect::ContractConstructorDecoder for #storage_ident {
                    type Type = __ink_ConstructorDecoder;
                }
            };
        )
    }

    /// Generates code for the ink! message decoder type of the ink! smart contract.
    ///
    /// This type can be used in order to decode the input bytes received by a call to `call`
    /// into one of the available dispatchable ink! messages and their arguments.
    fn generate_message_decoder_type(
        &self,
        message_spans: &[proc_macro2::Span],
    ) -> TokenStream2 {
        assert_eq!(message_spans.len(), self.query_amount_messages());

        /// Expands into the token sequence to represent the
        /// input type of the ink! message at the given index.
        fn expand_message_input(
            span: proc_macro2::Span,
            storage_ident: &syn::Ident,
            message_index: usize,
        ) -> TokenStream2 {
            quote_spanned!(span=>
                <#storage_ident as ::ink::reflect::DispatchableMessageInfo<{
                    <#storage_ident as ::ink::reflect::ContractDispatchableMessages<{
                        <#storage_ident as ::ink::reflect::ContractAmountDispatchables>::MESSAGES
                    }>>::IDS[#message_index]
                }>>::Input
            )
        }

        /// Returns the n-th ink! message identifier for the decoder type.
        fn message_variant_ident(n: usize) -> syn::Ident {
            quote::format_ident!("Message{}", n)
        }

        let span = self.contract.module().storage().span();
        let storage_ident = self.contract.module().storage().ident();
        let count_messages = self.query_amount_messages();
        let message_variants = (0..count_messages).map(|index| {
            let message_span = message_spans[index];
            let message_ident = message_variant_ident(index);
            let message_input = expand_message_input(message_span, storage_ident, index);
            quote_spanned!(message_span=>
                #message_ident(#message_input)
            )
        });

        let message_selector = (0..count_messages).map(|index| {
            let const_ident = format_ident!("MESSAGE_{}", index);
            quote_spanned!(span=>
                const #const_ident: [::core::primitive::u8; 4usize] = <#storage_ident as ::ink::reflect::DispatchableMessageInfo<{
                    <#storage_ident as ::ink::reflect::ContractDispatchableMessages<{
                        <#storage_ident as ::ink::reflect::ContractAmountDispatchables>::MESSAGES
                    }>>::IDS[#index]
                }>>::SELECTOR;
            )
        });

        let message_match = (0..count_messages).map(|index| {
            let message_span = message_spans[index];
            let message_ident = message_variant_ident(index);
            let const_ident = format_ident!("MESSAGE_{}", index);
            let message_input = expand_message_input(message_span, storage_ident, index);
            quote_spanned!(message_span=>
                #const_ident => {
                    ::core::result::Result::Ok(Self::#message_ident(
                        <#message_input as ::scale::Decode>::decode(input)
                            .map_err(|_| ::ink::reflect::DispatchError::InvalidParameters)?
                    ))
                }
            )
        });
        let possibly_wildcard_selector_message = match self.query_wildcard_message() {
            Some(wildcard_index) => {
                let message_span = message_spans[wildcard_index];
                let message_ident = message_variant_ident(wildcard_index);
                let message_input =
                    expand_message_input(message_span, storage_ident, wildcard_index);
                quote! {
                    ::core::result::Result::Ok(Self::#message_ident(
                        <#message_input as ::scale::Decode>::decode(input)
                            .map_err(|_| ::ink::reflect::DispatchError::InvalidParameters)?
                    ))
                }
            }
            None => {
                quote! {
                    ::core::result::Result::Err(::ink::reflect::DispatchError::UnknownSelector)
                }
            }
        };
        let any_message_accept_payment =
            self.any_message_accepts_payment_expr(message_spans);

        let message_execute = (0..count_messages).map(|index| {
            let message_span = message_spans[index];
            let message_ident = message_variant_ident(index);
            let message_callable = quote_spanned!(message_span=>
                <#storage_ident as ::ink::reflect::DispatchableMessageInfo<{
                    <#storage_ident as ::ink::reflect::ContractDispatchableMessages<{
                        <#storage_ident as ::ink::reflect::ContractAmountDispatchables>::MESSAGES
                    }>>::IDS[#index]
                }>>::CALLABLE
            );
            let message_output = quote_spanned!(message_span=>
                <#storage_ident as ::ink::reflect::DispatchableMessageInfo<{
                    <#storage_ident as ::ink::reflect::ContractDispatchableMessages<{
                        <#storage_ident as ::ink::reflect::ContractAmountDispatchables>::MESSAGES
                    }>>::IDS[#index]
                }>>::Output
            );
            let deny_payment = quote_spanned!(message_span=>
                !<#storage_ident as ::ink::reflect::DispatchableMessageInfo<{
                    <#storage_ident as ::ink::reflect::ContractDispatchableMessages<{
                        <#storage_ident as ::ink::reflect::ContractAmountDispatchables>::MESSAGES
                    }>>::IDS[#index]
                }>>::PAYABLE
            );
            let mutates_storage = quote_spanned!(message_span=>
                <#storage_ident as ::ink::reflect::DispatchableMessageInfo<{
                    <#storage_ident as ::ink::reflect::ContractDispatchableMessages<{
                        <#storage_ident as ::ink::reflect::ContractAmountDispatchables>::MESSAGES
                    }>>::IDS[#index]
                }>>::MUTATES
            );

            quote_spanned!(message_span=>
                Self::#message_ident(input) => {
                    if #any_message_accept_payment && #deny_payment {
                        ::ink::codegen::deny_payment::<
                            <#storage_ident as ::ink::env::ContractEnv>::Env>()?;
                    }

                    let result: #message_output = #message_callable(&mut contract, input);
                    let is_reverted = ::ink::is_result_type!(#message_output)
                        && ::ink::is_result_err!(result);

                    // no need to push back results: transaction gets reverted anyways
                    if !is_reverted {
                        push_contract(contract, #mutates_storage);
                    }

                    ::core::result::Result::Ok(::ink::env::return_value::<::ink::MessageResult::<#message_output>>(
                        ::ink::env::ReturnFlags::new_with_reverted(is_reverted),
                        // Currently no `LangError`s are raised at this level of the
                        // dispatch logic so `Ok` is always returned to the caller.
                        &::ink::MessageResult::Ok(result),
                    ))
                }
            )
        });

        quote_spanned!(span=>
            const _: () = {
                #[allow(non_camel_case_types)]
                pub enum __ink_MessageDecoder {
                    #( #message_variants ),*
                }

                impl ::ink::reflect::DecodeDispatch for __ink_MessageDecoder {
                    fn decode_dispatch<I>(input: &mut I)
                        -> ::core::result::Result<Self, ::ink::reflect::DispatchError>
                    where
                        I: ::scale::Input,
                    {
                        #(
                            #message_selector
                        )*
                        match <[::core::primitive::u8; 4usize] as ::scale::Decode>::decode(input)
                            .map_err(|_| ::ink::reflect::DispatchError::InvalidSelector)?
                        {
                            #( #message_match , )*
                            _invalid => #possibly_wildcard_selector_message
                        }
                    }
                }

                impl ::scale::Decode for __ink_MessageDecoder {
                    fn decode<I>(input: &mut I) -> ::core::result::Result<Self, ::scale::Error>
                    where
                        I: ::scale::Input,
                    {
                        <Self as ::ink::reflect::DecodeDispatch>::decode_dispatch(input)
                            .map_err(::core::convert::Into::into)
                    }
                }

                fn push_contract(contract: ::core::mem::ManuallyDrop<#storage_ident>, mutates: bool) {
                    if mutates {
                        ::ink::env::set_contract_storage::<::ink::primitives::Key, #storage_ident>(
                            &<#storage_ident as ::ink::storage::traits::StorageKey>::KEY,
                            &contract,
                        );
                    }
                }

                impl ::ink::reflect::ExecuteDispatchable for __ink_MessageDecoder {
                    #[allow(clippy::nonminimal_bool, clippy::let_unit_value)]
                    fn execute_dispatchable(
                        self
                    ) -> ::core::result::Result<(), ::ink::reflect::DispatchError> {
                        let key = <#storage_ident as ::ink::storage::traits::StorageKey>::KEY;
                        let mut contract: ::core::mem::ManuallyDrop<#storage_ident> =
                            ::core::mem::ManuallyDrop::new(
                                match ::ink::env::get_contract_storage(&key) {
                                    ::core::result::Result::Ok(::core::option::Option::Some(value)) => value,
                                    ::core::result::Result::Ok(::core::option::Option::None) => {
                                        ::core::panic!("storage entry was empty")
                                    },
                                    ::core::result::Result::Err(_) => {
                                        ::core::panic!("could not properly decode storage entry")
                                    },
                                }
                            );

                        match self {
                            #( #message_execute ),*
                        }
                    }
                }

                impl ::ink::reflect::ContractMessageDecoder for #storage_ident {
                    type Type = __ink_MessageDecoder;
                }
            };
        )
    }

    /// Generates code to express if any dispatchable ink! message accepts payment.
    ///
    /// This information can be used to speed-up dispatch since denying of payment
    /// can be generalized to work before dispatch happens if none of the ink! messages
    /// accept payment anyways.
    fn any_message_accepts_payment_expr(
        &self,
        message_spans: &[proc_macro2::Span],
    ) -> TokenStream2 {
        assert_eq!(message_spans.len(), self.query_amount_messages());

        let span = self.contract.module().storage().span();
        let storage_ident = self.contract.module().storage().ident();
        let count_messages = self.query_amount_messages();
        let message_is_payable = (0..count_messages).map(|index| {
            let message_span = message_spans[index];
            quote_spanned!(message_span=>
                <#storage_ident as ::ink::reflect::DispatchableMessageInfo<{
                    <#storage_ident as ::ink::reflect::ContractDispatchableMessages<{
                        <#storage_ident as ::ink::reflect::ContractAmountDispatchables>::MESSAGES
                    }>>::IDS[#index]
                }>>::PAYABLE
            )
        });
        quote_spanned!(span=>
            { false #( || #message_is_payable )* }
        )
    }

    /// Generates code to express if any dispatchable ink! constructor accepts payment.
    ///
    /// This information can be used to speed-up dispatch since denying of payment
    /// can be generalized to work before dispatch happens if none of the ink! constructors
    /// accept payment anyways.
    fn any_constructor_accepts_payment_expr(
        &self,
        constructor_spans: &[proc_macro2::Span],
    ) -> TokenStream2 {
        assert_eq!(constructor_spans.len(), self.query_amount_constructors());

        let span = self.contract.module().storage().span();
        let storage_ident = self.contract.module().storage().ident();
        let count_constructors = self.query_amount_constructors();
        let constructor_is_payable = (0..count_constructors).map(|index| {
            let constructor_span = constructor_spans[index];
            quote_spanned!(constructor_span=>
                <#storage_ident as ::ink::reflect::DispatchableConstructorInfo<{
                    <#storage_ident as ::ink::reflect::ContractDispatchableConstructors<{
                        <#storage_ident as ::ink::reflect::ContractAmountDispatchables>::CONSTRUCTORS
                    }>>::IDS[#index]
                }>>::PAYABLE
            )
        });
        quote_spanned!(span=>
            { false #( || #constructor_is_payable )* }
        )
    }
}
