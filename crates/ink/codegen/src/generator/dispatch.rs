// Copyright (C) Use Ink (UK) Ltd.
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

use derive_more::From;
use ink_primitives::abi::Abi;
use ir::{
    Callable,
    CallableWithSelector,
    Constructor,
    HexLiteral as _,
    Message,
};
use itertools::Itertools;
use proc_macro2::{
    Ident,
    TokenStream as TokenStream2,
};
use quote::{
    format_ident,
    quote,
    quote_spanned,
};
use syn::spanned::Spanned as _;

use crate::{
    GenerateCode,
    generator,
    generator::sol,
};

/// A message to be dispatched.
/// Contains its callable and calculated unique id
pub struct MessageDispatchable<'a> {
    message: CallableWithSelector<'a, Message>,
    id: TokenStream2,
}

/// A constructor to be dispatched.
/// Contains its callable and calculated unique id
pub struct ConstructorDispatchable<'a> {
    constructor: CallableWithSelector<'a, Constructor>,
    id: TokenStream2,
    abi: Abi,
}

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
        let messages = self.compose_messages_with_ids();
        let constructors = self.compose_constructors_with_ids();

        let contract_dispatchable_constructor_infos =
            self.generate_dispatchable_constructor_infos();
        let contract_dispatchable_messages_infos =
            self.generate_dispatchable_message_infos();
        let constructor_decoder_type =
            self.generate_constructor_decoder_type(&constructors);
        let message_decoder_type = self.generate_message_decoder_type(&messages);
        let entry_points = self.generate_entry_points(&constructors, &messages);

        quote! {
            #contract_dispatchable_constructor_infos
            #contract_dispatchable_messages_infos
            #constructor_decoder_type
            #message_decoder_type

            #[cfg(not(any(test, feature = "std", feature = "ink-as-dependency")))]
            mod __do_not_access__ {
                use super::*;
                #entry_points
            }
        }
    }
}

// We arbitrarily use zero as the id for the Solidity ABI encoded constructor.
// SAFETY: ink! ABI encoded constructors compute their ids as the
// `u32` representation of their selector, so a collision is unlikely,
// and it would lead to compilation error anyway.
const SOL_CTOR_ID: u32 = 0;

impl Dispatch<'_> {
    /// Returns the index of the ink! message which has a wildcard selector, if existent.
    fn query_wildcard_message(&self) -> Option<usize> {
        self.contract
            .module()
            .impls()
            .flat_map(|item_impl| item_impl.iter_messages())
            .position(|item| item.has_wildcard_selector())
    }

    /// Returns the index of the ink! constructor which has a wildcard selector, if
    /// existent.
    #[cfg_attr(ink_abi = "sol", allow(dead_code))]
    fn query_wildcard_constructor(&self) -> Option<usize> {
        self.contract
            .module()
            .impls()
            .flat_map(|item_impl| item_impl.iter_constructors())
            .position(|item| item.has_wildcard_selector())
    }

    /// Returns the constructor to use for Solidity ABI encoded instantiation.
    fn constructor_sol(&self) -> Option<CallableWithSelector<'_, Constructor>> {
        self.contract
            .module()
            .impls()
            .flat_map(|item_impl| item_impl.iter_constructors())
            .find_or_first(|constructor| constructor.is_default())
    }

    /// Puts messages and their calculated selector ids in a single data structure
    ///
    /// See [`MessageDispatchable`]
    fn compose_messages_with_ids(&self) -> Vec<MessageDispatchable<'_>> {
        let storage_ident = self.contract.module().storage().ident();
        self.contract
            .module()
            .impls()
            .flat_map(|item_impl| {
                iter::repeat(item_impl.trait_path()).zip(item_impl.iter_messages())
            })
            .flat_map(|(trait_path, message)| {
                let mut message_dispatchables = Vec::new();
                for_each_abi!(@type |abi| {
                    match abi {
                        Abi::Ink => {
                            let span = message.span();
                            let id = if let Some(trait_path) = trait_path {
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
                                quote_spanned!(span=>
                                    #id
                                )
                            };
                            message_dispatchables.push(MessageDispatchable { message, id });
                        }
                        Abi::Sol => {
                            let id = sol::utils::selector_id(&message);
                            message_dispatchables.push(MessageDispatchable { message, id });
                        }
                    }
                });

                message_dispatchables
            })
            .collect::<Vec<_>>()
    }

    /// Puts constructors and their calculated selector ids in a single data structure
    ///
    /// See [`ConstructorDispatchable`]
    fn compose_constructors_with_ids(&self) -> Vec<ConstructorDispatchable<'_>> {
        let mut constructor_dispatchables = Vec::new();
        for_each_abi!(@type |abi| {
            match abi {
                Abi::Ink => {
                    constructor_dispatchables.extend(
                        self.contract
                        .module()
                        .impls()
                        .flat_map(|item_impl| item_impl.iter_constructors())
                        .map(|constructor| {
                            let id = constructor
                                .composed_selector()
                                .into_be_u32()
                                .hex_padded_suffixed();
                            ConstructorDispatchable {
                                constructor,
                                id: quote!( #id ),
                                abi: Abi::Ink
                            }
                        })
                    );
                }
                Abi::Sol => {
                    // Only one constructor is used for Solidity ABI encoding.
                    let constructor = self.constructor_sol()
                        .expect("Expected at least one constructor");
                    constructor_dispatchables.push(
                        ConstructorDispatchable {
                            constructor,
                            id: quote!( #SOL_CTOR_ID ),
                            abi: Abi::Sol
                        }
                    );
                }
            }
        });
        constructor_dispatchables
    }

    /// Generate code for the [`ink::DispatchableConstructorInfo`] trait implementations.
    ///
    /// These trait implementations store relevant dispatch information for every
    /// dispatchable ink! constructor of the ink! smart contract.
    fn generate_dispatchable_constructor_infos(&self) -> TokenStream2 {
        generate_abi_impls!(@type |abi| {
            match abi {
                Abi::Ink => {
                    let span = self.contract.module().storage().span();
                    let constructor_infos = self
                        .contract
                        .module()
                        .impls()
                        .flat_map(|item_impl| item_impl.iter_constructors())
                        .map(|constructor| self.generate_dispatchable_constructor_info(constructor, abi));
                    quote_spanned!(span=>
                        #( #constructor_infos )*
                    )
                }
                Abi::Sol => {
                    // Only one constructor is used for Solidity ABI encoding.
                    let constructor = self.constructor_sol()
                        .expect("Expected at least one constructor");
                    self.generate_dispatchable_constructor_info(constructor, abi)
                }
            }
        })
    }

    /// Generate code for the [`ink::DispatchableConstructorInfo`] trait implementation
    /// for a single dispatchable ink! constructor of the ink! smart contract.
    ///
    /// This trait implementation stores relevant dispatch information for the
    /// dispatchable ink! constructor for the specified ABI.
    ///
    /// # Note
    ///
    /// Only one constructor is used for Solidity ABI encoding,
    /// so generating Solidity ABI encoded constructor info for multiple constructors
    /// results in a compilation error.
    fn generate_dispatchable_constructor_info(
        &self,
        constructor: CallableWithSelector<Constructor>,
        abi: Abi,
    ) -> TokenStream2 {
        let storage_ident = self.contract.module().storage().ident();
        let span = constructor.span();
        let constructor_ident = constructor.ident();
        let payable = constructor.is_payable();
        let cfg_attrs = constructor.get_cfg_attrs(span);
        let output_type = constructor
            .output()
            .map(quote::ToTokens::to_token_stream)
            .unwrap_or_else(|| quote! { () });
        let input_bindings = generator::input_bindings(constructor.inputs());
        let input_tuple_type = generator::input_types_tuple(constructor.inputs());
        let input_tuple_bindings = generator::input_bindings_tuple(constructor.inputs());
        let constructor_return_type = quote_spanned!(span=>
            <::ink::reflect::ConstructorOutputValue<#output_type>
                as ::ink::reflect::ConstructorOutput<#storage_ident>>
        );

        #[cfg(feature = "std")]
        let return_type = quote! { () };
        #[cfg(not(feature = "std"))]
        let return_type = quote! { ! };

        let (constructor_id, selector_bytes, abi_ty, decode_trait, return_expr) =
            match abi {
                Abi::Ink => {
                    let id = constructor
                        .composed_selector()
                        .into_be_u32()
                        .hex_padded_suffixed();
                    let selector_bytes = constructor.composed_selector().hex_lits();
                    (
                        quote!( #id ),
                        quote! {
                            ::core::option::Option::Some([ #( #selector_bytes ),* ])
                        },
                        quote!(::ink::abi::Abi::Ink),
                        quote!(::ink::scale::Decode),
                        quote_spanned!(span =>
                            ::ink::env::return_value::<
                                ::ink::ConstructorResult<
                                    ::core::result::Result<(), &Self::Error>
                                >,
                            >(
                                flags,
                                // Currently no `LangError`s are raised at this level of the
                                // dispatch logic so `Ok` is always returned to the caller.
                                &::ink::ConstructorResult::Ok(output),
                            )
                        ),
                    )
                }
                Abi::Sol => {
                    // Only one constructor is used for Solidity ABI encoding.
                    // We always use the same selector id for Solidity constructors.
                    // Attempting to generate constructor info for multiple constructors
                    // will thus lead to a compilation error.
                    let decode_trait = if input_bindings.len() == 1 {
                        quote!(::ink::SolDecode)
                    } else {
                        quote!(::ink::sol::SolParamsDecode)
                    };
                    (
                        quote!( #SOL_CTOR_ID ),
                        quote!(::core::option::Option::None),
                        quote!(::ink::abi::Abi::Sol),
                        decode_trait,
                        quote_spanned!(span =>
                            ::ink::env::return_value_solidity::<::core::result::Result<(), &Self::Error>>(
                                flags,
                                &output,
                            )
                        ),
                    )
                }
            };
        quote_spanned!(span =>
            #( #cfg_attrs )*
            impl ::ink::reflect::DispatchableConstructorInfo<#constructor_id> for #storage_ident {
                type Input = #input_tuple_type;
                type Output = #output_type;
                type Storage = #storage_ident;
                type Error = #constructor_return_type::Error;
                const IS_RESULT: ::core::primitive::bool = #constructor_return_type::IS_RESULT;

                const CALLABLE: fn(Self::Input) -> Self::Output = |#input_tuple_bindings| {
                    #storage_ident::#constructor_ident(#( #input_bindings ),* )
                };
                const DECODE: fn(&mut &[::core::primitive::u8]) -> ::core::result::Result<Self::Input, ::ink::env::DispatchError> =
                    |input| {
                        <Self::Input as #decode_trait>::decode(input)
                            .map_err(|_| ::ink::env::DispatchError::InvalidParameters)
                    };
                const RETURN: fn(::ink::env::ReturnFlags, ::core::result::Result<(), &Self::Error>) -> #return_type =
                        |flags, output| {
                            #return_expr
                        };
                const PAYABLE: ::core::primitive::bool = #payable;
                const SELECTOR: ::core::option::Option<[::core::primitive::u8; 4usize]> = #selector_bytes;
                const LABEL: &'static ::core::primitive::str = ::core::stringify!(#constructor_ident);
                const ABI: ::ink::abi::Abi = #abi_ty;
            }
        )
    }

    /// Generate code for the [`ink::DispatchableMessageInfo`] trait implementations.
    ///
    /// These trait implementations store relevant dispatch information for every
    /// dispatchable ink! message of the ink! smart contract.
    fn generate_dispatchable_message_infos(&self) -> TokenStream2 {
        let span = self.contract.module().storage().span();

        let inherent_message_infos = self
            .contract
            .module()
            .impls()
            .filter(|item_impl| item_impl.trait_path().is_none())
            .flat_map(|item_impl| item_impl.iter_messages())
            .map(|message| self.dispatchable_inherent_message_infos(&message));
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
                self.dispatchable_trait_message_infos(&message, trait_ident, trait_path)
            });
        quote_spanned!(span=>
            #( #inherent_message_infos )*
            #( #trait_message_infos )*
        )
    }

    /// Generate code for the [`ink::DispatchableMessageInfo`] trait implementations.
    ///
    /// These trait implementations store relevant dispatch information for every
    /// inherent dispatchable ink! message of the ink! smart contract.
    fn dispatchable_inherent_message_infos(
        &self,
        message: &CallableWithSelector<Message>,
    ) -> TokenStream2 {
        let storage_ident = self.contract.module().storage().ident();

        let message_span = message.span();
        let message_ident = message.ident();
        let payable = message.is_payable();
        let mutates = message.receiver().is_ref_mut();

        let cfg_attrs = message.get_cfg_attrs(message_span);
        let output_tuple_type = message
            .output()
            .map(quote::ToTokens::to_token_stream)
            .unwrap_or_else(|| quote! { () });
        let input_bindings = generator::input_bindings(message.inputs());
        let input_tuple_type = generator::input_types_tuple(message.inputs());
        let input_tuple_bindings = generator::input_bindings_tuple(message.inputs());

        #[cfg(feature = "std")]
        let return_type = quote! { () };
        #[cfg(not(feature = "std"))]
        let return_type = quote! { ! };

        generate_abi_impls!(@type |abi| {
            let (selector_id, selector_bytes, abi_ty, decode_trait, return_expr) = match abi {
                Abi::Ink => {
                    let selector_id = message
                        .composed_selector()
                        .into_be_u32()
                        .hex_padded_suffixed();
                    let selector_bytes = message.composed_selector().hex_lits();
                    (
                        quote!(#selector_id),
                        quote!([ #( #selector_bytes ),* ]),
                        quote!(::ink::abi::Abi::Ink),
                        quote!(::ink::scale::Decode),
                        quote! {
                            ::ink::env::return_value::<::ink::MessageResult::<Self::Output>>(
                                flags,
                                // Currently no `LangError`s are raised at this level of the
                                // dispatch logic so `Ok` is always returned to the caller.
                                &::ink::MessageResult::Ok(output),
                            )
                        },
                    )
                }
                Abi::Sol => {
                    let decode_trait = if input_bindings.len() == 1 {
                        quote!(::ink::SolDecode)
                    } else {
                        quote!(::ink::sol::SolParamsDecode)
                    };
                    (
                        sol::utils::selector_id(message),
                        sol::utils::selector(message),
                        quote!(::ink::abi::Abi::Sol),
                        decode_trait,
                        quote! {
                            ::ink::env::return_value_solidity::<Self::Output>(
                                flags,
                                &output,
                            )
                        },
                    )
                }
            };
            quote_spanned!(message_span=>
                #( #cfg_attrs )*
                impl ::ink::reflect::DispatchableMessageInfo<#selector_id> for #storage_ident {
                    type Input = #input_tuple_type;
                    type Output = #output_tuple_type;
                    type Storage = #storage_ident;

                    const CALLABLE: fn(&mut Self::Storage, Self::Input) -> Self::Output =
                        |storage, #input_tuple_bindings| {
                            #storage_ident::#message_ident( storage #( , #input_bindings )* )
                        };
                    const DECODE: fn(&mut &[::core::primitive::u8]) -> ::core::result::Result<Self::Input, ::ink::env::DispatchError> =
                        |input| {
                            <Self::Input as #decode_trait>::decode(input)
                                .map_err(|_| ::ink::env::DispatchError::InvalidParameters)
                        };
                    const RETURN: fn(::ink::env::ReturnFlags, Self::Output) -> #return_type =
                        |flags, output| {
                            #return_expr
                        };
                    const SELECTOR: [::core::primitive::u8; 4usize] = #selector_bytes;
                    const PAYABLE: ::core::primitive::bool = #payable;
                    const MUTATES: ::core::primitive::bool = #mutates;
                    const LABEL: &'static ::core::primitive::str = ::core::stringify!(#message_ident);
                    const ABI: ::ink::abi::Abi = #abi_ty;
                }
            )
        })
    }

    /// Generate code for the [`ink::DispatchableMessageInfo`] trait implementations.
    ///
    /// These trait implementations store relevant dispatch information for every
    /// trait implementation dispatchable ink! message of the ink! smart contract.
    fn dispatchable_trait_message_infos(
        &self,
        message: &CallableWithSelector<Message>,
        trait_ident: &Ident,
        trait_path: &syn::Path,
    ) -> TokenStream2 {
        let storage_ident = self.contract.module().storage().ident();

        let message_span = message.span();
        let message_ident = message.ident();
        let mutates = message.receiver().is_ref_mut();
        let output_tuple_type = message
            .output()
            .map(quote::ToTokens::to_token_stream)
            .unwrap_or_else(|| quote! { () });
        let input_bindings = generator::input_bindings(message.inputs());
        let input_tuple_type = generator::input_types_tuple(message.inputs());
        let input_tuple_bindings = generator::input_bindings_tuple(message.inputs());
        let label = format!("{trait_ident}::{message_ident}");
        let cfg_attrs = message.get_cfg_attrs(message_span);

        #[cfg(feature = "std")]
        let return_type = quote! { () };
        #[cfg(not(feature = "std"))]
        let return_type = quote! { ! };

        generate_abi_impls!(@type |abi| {
            let (local_id, abi_ty, decode_trait, return_expr) = match abi {
                Abi::Ink => {
                    let local_id = message.local_id().hex_padded_suffixed();
                    (
                        quote!(#local_id),
                        quote!(::ink::abi::Abi::Ink),
                        quote!(::ink::scale::Decode),
                        quote! {
                            ::ink::env::return_value::<::ink::MessageResult::<Self::Output>>(
                                flags,
                                // Currently no `LangError`s are raised at this level of the
                                // dispatch logic so `Ok` is always returned to the caller.
                                &::ink::MessageResult::Ok(output),
                            )
                        },
                    )
                }
                Abi::Sol => {
                    let decode_trait = if input_bindings.len() == 1 {
                        quote!(::ink::SolDecode)
                    } else {
                        quote!(::ink::sol::SolParamsDecode)
                    };
                    (
                        sol::utils::selector_id(message),
                        quote!(::ink::abi::Abi::Sol),
                        decode_trait,
                        quote! {
                            ::ink::env::return_value_solidity::<Self::Output>(
                                flags,
                                &output,
                            )
                        },
                    )
                }
            };
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
            quote_spanned!(message_span=>
                #( #cfg_attrs )*
                impl ::ink::reflect::DispatchableMessageInfo<#selector_id> for #storage_ident {
                    type Input = #input_tuple_type;
                    type Output = #output_tuple_type;
                    type Storage = #storage_ident;

                    const CALLABLE: fn(&mut Self::Storage, Self::Input) -> Self::Output =
                        |storage, #input_tuple_bindings| {
                            <#storage_ident as #trait_path>::#message_ident( storage #( , #input_bindings )* )
                        };
                    const DECODE: fn(&mut &[::core::primitive::u8]) -> ::core::result::Result<Self::Input, ::ink::env::DispatchError> =
                        |input| {
                            <Self::Input as #decode_trait>::decode(input)
                                .map_err(|_| ::ink::env::DispatchError::InvalidParameters)
                        };
                    const RETURN: fn(::ink::env::ReturnFlags, Self::Output) -> #return_type =
                        |flags, output| {
                            #return_expr
                        };
                    const SELECTOR: [::core::primitive::u8; 4usize] = #selector;
                    const PAYABLE: ::core::primitive::bool = #payable;
                    const MUTATES: ::core::primitive::bool = #mutates;
                    const LABEL: &'static ::core::primitive::str = #label;
                    const ABI: ::ink::abi::Abi = #abi_ty;
                }
            )
        })
    }

    /// Generates code for the entry points of the root ink! smart contract.
    ///
    /// This generates the `deploy` and `call` functions with which the smart
    /// contract runtime mainly interacts with the ink! smart contract.
    fn generate_entry_points(
        &self,
        constructors: &[ConstructorDispatchable],
        messages: &[MessageDispatchable],
    ) -> TokenStream2 {
        let span = self.contract.module().storage().span();
        let storage_ident = self.contract.module().storage().ident();
        let any_constructor_accept_payment =
            self.any_constructor_accepts_payment(constructors);
        let any_message_accepts_payment = self.any_message_accepts_payment(messages);

        let fn_call: syn::ItemFn = syn::parse_quote! {
            #[cfg(target_arch = "riscv64")]
            #[::ink::polkavm_export(abi = ::ink::polkavm_derive::default_abi)]
            pub extern "C" fn call() {
                internal_call()
            }
        };
        let fn_deploy: syn::ItemFn = syn::parse_quote! {
            #[cfg(target_arch = "riscv64")]
            #[::ink::polkavm_export(abi = ::ink::polkavm_derive::default_abi)]
            pub extern "C" fn deploy() {
                internal_deploy()
            }
        };
        quote_spanned!(span=>
            #[allow(dead_code)] // clippy throws a false positive otherwise
            #[allow(clippy::nonminimal_bool)]
            #[cfg(target_arch = "riscv64")]
            fn internal_deploy() {
                if !#any_constructor_accept_payment {
                    ::ink::codegen::deny_payment()
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

                        // At this point we're unable to set the `Ok` variant to be the "real"
                        // constructor output since we were unable to figure out what the caller wanted
                        // to dispatch in the first place, so we set it to `()`.
                        //
                        // This is okay since we're going to only be encoding the `Err` variant
                        // into the output buffer anyway.
                        ::ink::env::return_value::<::ink::ConstructorResult<()>>(
                            ::ink::env::ReturnFlags::REVERT,
                            &error,
                        );
                    }
                };

                <<#storage_ident as ::ink::reflect::ContractConstructorDecoder>::Type
                    as ::ink::reflect::ExecuteDispatchable>::execute_dispatchable(dispatchable)
                .unwrap_or_else(|error| {
                    ::core::panic!("dispatching ink! constructor failed: {}", error)
                })
            }

            #[allow(dead_code)] // clippy throws a false positive otherwise
            #[allow(clippy::nonminimal_bool)]
            #[cfg(target_arch = "riscv64")]
            fn internal_call() {
                if !#any_message_accepts_payment {
                    ::ink::codegen::deny_payment()
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

                        // At this point we're unable to set the `Ok` variant to be the "real"
                        // message output since we were unable to figure out what the caller wanted
                        // to dispatch in the first place, so we set it to `()`.
                        //
                        // This is okay since we're going to only be encoding the `Err` variant
                        // into the output buffer anyway.
                        ::ink::env::return_value::<::ink::MessageResult<()>>(
                            ::ink::env::ReturnFlags::REVERT,
                            &error,
                        );
                    }
                };

                <<#storage_ident as ::ink::reflect::ContractMessageDecoder>::Type
                    as ::ink::reflect::ExecuteDispatchable>::execute_dispatchable(dispatchable)
                .unwrap_or_else(|error| {
                    ::core::panic!("dispatching ink! message failed: {}", error)
                })
            }

            #fn_call

            #fn_deploy
        )
    }

    /// Generates code for the ink! constructor decoder type of the ink! smart contract.
    ///
    /// This type can be used in order to decode the input bytes received by a call to
    /// `deploy` into one of the available dispatchable ink! constructors and their
    /// arguments.
    fn generate_constructor_decoder_type(
        &self,
        constructors: &[ConstructorDispatchable],
    ) -> TokenStream2 {
        /// Expands into the token sequence to represent the
        /// input type of the ink! constructor at the given index.
        fn expand_constructor_input(
            span: proc_macro2::Span,
            storage_ident: &syn::Ident,
            constructor_id: TokenStream2,
        ) -> TokenStream2 {
            quote_spanned!(span=>
                <#storage_ident as ::ink::reflect::DispatchableConstructorInfo< #constructor_id >>::Input
            )
        }

        /// Returns the n-th ink! constructor identifier for the decoder type.
        fn constructor_variant_ident(n: usize) -> syn::Ident {
            format_ident!("Constructor{}", n)
        }

        let span = self.contract.module().storage().span();
        let storage_ident = self.contract.module().storage().ident();
        let constructors_variants =
            constructors.iter().enumerate().map(|(index, item)| {
                let constructor_span = item.constructor.span();
                let constructor_ident = constructor_variant_ident(index);
                let cfg_attrs = item.constructor.get_cfg_attrs(constructor_span);
                let constructor_input = expand_constructor_input(
                    constructor_span,
                    storage_ident,
                    item.id.clone(),
                );
                quote_spanned!(constructor_span=>
                    #( #cfg_attrs )*
                    #constructor_ident(#constructor_input)
                )
            });

        // Returns dispatch decoding logic for Solidity ABI encoded constructor calls.
        let solidity_constructor_dispatch_decode = || {
            let solidity_constructor_info = constructors
                .iter()
                .enumerate()
                .find(|(_, item)| matches!(item.abi, Abi::Sol));
            match solidity_constructor_info {
                Some((index, item)) => {
                    let constructor_span = item.constructor.span();
                    let constructor_ident = constructor_variant_ident(index);
                    quote_spanned!(constructor_span=>
                        <#storage_ident as ::ink::reflect::DispatchableConstructorInfo< #SOL_CTOR_ID >>::DECODE(input)
                            .map(Self::#constructor_ident)
                    )
                }
                None => {
                    quote! {
                        ::core::result::Result::Err(::ink::env::DispatchError::UnknownSelector)
                    }
                }
            }
        };

        #[cfg(not(ink_abi = "sol"))]
        let decode_dispatch = {
            let constructor_selector =
                constructors.iter().enumerate().filter_map(|(index, item)| {
                    if matches!(item.abi, Abi::Sol) {
                        // The Solidity ABI encoded constructor doesn't have a selector.
                        return None;
                    }
                    let constructor_span = item.constructor.span();
                    let const_ident = format_ident!("CONSTRUCTOR_{}", index);
                    let id = item.id.clone();
                    let cfg_attrs = item.constructor.get_cfg_attrs(constructor_span);
                    Some(quote_spanned!(span=>
                        #( #cfg_attrs )*
                        const #const_ident: [::core::primitive::u8; 4usize] = <
                            #storage_ident as ::ink::reflect::DispatchableConstructorInfo< #id >
                        >::SELECTOR.expect("Expected a selector");
                    ))
                });

            let constructor_match = constructors.iter()
                .enumerate()
                .filter_map(|(index, item)| {
                    if matches!(item.abi, Abi::Sol) {
                        // The Solidity ABI encoded constructor doesn't have a selector to match against.
                        return None;
                    }
                    let constructor_span = item.constructor.span();
                    let constructor_ident = constructor_variant_ident(index);
                    let const_ident = format_ident!("CONSTRUCTOR_{}", index);
                    let id = item.id.clone();
                    let cfg_attrs = item.constructor.get_cfg_attrs(constructor_span);
                    Some(quote_spanned!(constructor_span=>
                        #( #cfg_attrs )*
                        #const_ident => {
                            ::core::result::Result::Ok(Self::#constructor_ident(
                                <#storage_ident as ::ink::reflect::DispatchableConstructorInfo< #id >>::DECODE(input)?
                            ))
                        }
                    ))
                });

            let wildcard_selector_constructor =
                self.query_wildcard_constructor().map(|wildcard_index| {
                    let item = &constructors[wildcard_index];
                    let constructor_span = item.constructor.span();
                    let constructor_ident = constructor_variant_ident(wildcard_index);
                    let constructor_input = expand_constructor_input(
                        constructor_span,
                        storage_ident,
                        item.id.clone(),
                    );
                    quote_spanned!(constructor_span=>
                        <#constructor_input as ::ink::scale::Decode>::decode(input)
                            .map(Self::#constructor_ident)
                            .map_err(|_| ::ink::env::DispatchError::InvalidParameters)
                    )
                });

            let (possible_full_input_ref, fallback_match) = if cfg!(ink_abi = "all") {
                let solidity_constructor = solidity_constructor_dispatch_decode();
                let try_wildcard_selector_constructor = wildcard_selector_constructor
                    .map(|wildcard_selector_constructor| {
                        quote_spanned!(span=>
                            let wildcard_result = #wildcard_selector_constructor;
                            if wildcard_result.is_ok() {
                                return wildcard_result;
                            }
                        )
                    });
                (
                    // Keeps a reference to the entire input slice for Solidity ABI
                    // decoding fallback.
                    quote_spanned!(span=>
                        let mut full_input = *input;
                    ),
                    quote_spanned!(span=>
                        {
                            // Try wildcard constructor (if any).
                            #try_wildcard_selector_constructor

                            // Fallback to Solidity constructor.
                            let input = &mut full_input;
                            #solidity_constructor
                        }
                    ),
                )
            } else {
                let fallback_match = wildcard_selector_constructor.unwrap_or_else(|| {
                    quote_spanned!(span =>
                        ::core::result::Result::Err(::ink::env::DispatchError::UnknownSelector)
                    )
                });
                (quote!(), fallback_match)
            };

            quote_spanned!(span =>
                #possible_full_input_ref
                #(
                    #constructor_selector
                )*
                match <[::core::primitive::u8; 4usize] as ::ink::scale::Decode>::decode(input)
                    .map_err(|_| ::ink::env::DispatchError::InvalidSelector)?
                {
                    #( #constructor_match , )*
                    _invalid => #fallback_match
                }
            )
        };
        #[cfg(ink_abi = "sol")]
        let decode_dispatch = {
            let solidity_constructor = solidity_constructor_dispatch_decode();
            quote_spanned!(span =>
                #solidity_constructor
            )
        };

        let constructor_execute = constructors.iter().enumerate().map(|(index, item)| {
            let constructor_span = item.constructor.span();
            let constructor_ident = constructor_variant_ident(index);
            let id = item.id.clone();
            let cfg_attrs = item.constructor.get_cfg_attrs(constructor_span);
            let constructor_callable = quote_spanned!(constructor_span=>
                <#storage_ident as ::ink::reflect::DispatchableConstructorInfo< #id >>::CALLABLE
            );
            let constructor_return = quote_spanned!(constructor_span=>
                    <#storage_ident as ::ink::reflect::DispatchableConstructorInfo< #id >>::RETURN
                );
            let constructor_output = quote_spanned!(constructor_span=>
                <#storage_ident as ::ink::reflect::DispatchableConstructorInfo< #id >>::Output
            );
            let deny_payment = quote_spanned!(constructor_span=>
                !<#storage_ident as ::ink::reflect::DispatchableConstructorInfo< #id >>::PAYABLE
            );
            let constructor_value = quote_spanned!(constructor_span=>
                <::ink::reflect::ConstructorOutputValue<#constructor_output>
                    as ::ink::reflect::ConstructorOutput::<#storage_ident>>
            );

            let constructor_accept_payment_assignment =
                self.any_constructor_accepts_payment(constructors);

            quote_spanned!(constructor_span=>
                #( #cfg_attrs )*
                Self::#constructor_ident(input) => {
                    if #constructor_accept_payment_assignment && #deny_payment {
                        ::ink::codegen::deny_payment()?;
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

                    // NOTE: we can't use an if/else expression here
                    // It fails inside quote_spanned! macro.
                    // See https://github.com/rust-lang/rust-clippy/issues/6249
                    let mut flag = ::ink::env::ReturnFlags::empty();
                    if output_result.is_err() {
                        flag = ::ink::env::ReturnFlags::REVERT;
                    }

                    // Constructors don't return arbitrary data,
                    // so the return type is represented as `Result<(), Error>`
                    // where `Error = ()` for infallible constructors.
                    // For Solidity ABI, `Error` is encoded as revert error data.
                    #constructor_return(flag, output_result.map(|_| ()));

                    #[cfg(feature = "std")]
                    return ::core::result::Result::Ok(());

                    #[cfg(not(feature = "std"))]
                    #[cfg_attr(not(feature = "std"), allow(unreachable_code))]
                    {
                        ::core::unreachable!("either `return_value` or the `return` before will already have returned");
                    }
                }
            )
        });
        quote_spanned!(span=>
            const _: () = {
                #[allow(non_camel_case_types)]
                pub enum __ink_ConstructorDecoder {
                    #( #constructors_variants ),*
                }

                impl ::ink::env::DecodeDispatch for __ink_ConstructorDecoder {
                    fn decode_dispatch(input: &mut &[::core::primitive::u8])
                        -> ::core::result::Result<Self, ::ink::env::DispatchError> {
                        #decode_dispatch
                    }
                }

                impl ::ink::reflect::ExecuteDispatchable for __ink_ConstructorDecoder {
                    #[allow(clippy::nonminimal_bool, dead_code)]
                    fn execute_dispatchable(self) -> ::core::result::Result<(), ::ink::env::DispatchError> {
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
    /// This type can be used in order to decode the input bytes received by a call to
    /// `call` into one of the available dispatchable ink! messages and their
    /// arguments.
    fn generate_message_decoder_type(
        &self,
        messages: &[MessageDispatchable],
    ) -> TokenStream2 {
        /// Expands into the token sequence to represent the
        /// input type of the ink! message at the given index.
        fn expand_message_input(
            span: proc_macro2::Span,
            storage_ident: &Ident,
            message_id: TokenStream2,
        ) -> TokenStream2 {
            quote_spanned!(span=>
                <#storage_ident as ::ink::reflect::DispatchableMessageInfo< #message_id >>::Input
            )
        }

        /// Returns the n-th ink! message identifier for the decoder type.
        fn message_variant_ident(n: usize) -> syn::Ident {
            format_ident!("Message{}", n)
        }

        let span = self.contract.module().storage().span();
        let storage_ident = self.contract.module().storage().ident();
        let message_variants = messages.iter().enumerate().map(|(index, item)| {
            let message_span = item.message.span();
            let message_ident = message_variant_ident(index);
            let cfg_attrs = item.message.get_cfg_attrs(message_span);
            let message_input =
                expand_message_input(message_span, storage_ident, item.id.clone());
            quote_spanned!(message_span=>
                #( #cfg_attrs )*
                #message_ident(#message_input)
            )
        });

        let message_selector = messages
            .iter()
            .enumerate()
            .map(|(index, item)|  {
                let message_span = item.message.span();
                let const_ident = format_ident!("MESSAGE_{}", index);
                let cfg_attrs = item.message.get_cfg_attrs(message_span);
                let id = item.id.clone();
                quote_spanned!(span=>
                    #( #cfg_attrs )*
                    const #const_ident: [::core::primitive::u8; 4usize] = <#storage_ident as ::ink::reflect::DispatchableMessageInfo< #id >>::SELECTOR;
                )
        });

        let message_match =  messages
            .iter()
            .enumerate()
            .map(|(index, item)| {
                let message_ident = message_variant_ident(index);
                let const_ident = format_ident!("MESSAGE_{}", index);
                let message_span = item.message.span();
                let cfg_attrs = item.message.get_cfg_attrs(message_span);
                let id = item.id.clone();
                quote_spanned!(message_span=>
                   #( #cfg_attrs )*
                    #const_ident => {
                        ::core::result::Result::Ok(Self::#message_ident(
                            <#storage_ident as ::ink::reflect::DispatchableMessageInfo< #id >>::DECODE(input)?
                        ))
                    }
                )
        });
        let possibly_wildcard_selector_message = match self.query_wildcard_message() {
            Some(wildcard_index) => {
                let item = messages
                    .get(wildcard_index)
                    .expect("unable to get wildcard index");
                let message_span = item.message.span();
                let message_ident = message_variant_ident(wildcard_index);
                let message_input =
                    expand_message_input(message_span, storage_ident, item.id.clone());
                quote! {
                    ::core::result::Result::Ok(Self::#message_ident(
                        <#message_input as ::ink::scale::Decode>::decode(input)
                            .map_err(|_| ::ink::env::DispatchError::InvalidParameters)?
                    ))
                }
            }
            None => {
                quote! {
                    ::core::result::Result::Err(::ink::env::DispatchError::UnknownSelector)
                }
            }
        };

        let message_execute = messages
            .iter()
            .enumerate()
            .map(|(index, item)| {
                let message_span = item.message.span();
                let message_ident = message_variant_ident(index);
                let id = item.id.clone();
                let cfg_attrs = item.message.get_cfg_attrs(message_span);
                let message_callable = quote_spanned!(message_span=>
                    <#storage_ident as ::ink::reflect::DispatchableMessageInfo< #id >>::CALLABLE
                );
                let message_return = quote_spanned!(message_span=>
                    <#storage_ident as ::ink::reflect::DispatchableMessageInfo< #id >>::RETURN
                );
                let message_output = quote_spanned!(message_span=>
                    <#storage_ident as ::ink::reflect::DispatchableMessageInfo< #id >>::Output
                );
                let deny_payment = quote_spanned!(message_span=>
                    !<#storage_ident as ::ink::reflect::DispatchableMessageInfo< #id >>::PAYABLE
                );
                let mutates_storage = quote_spanned!(message_span=>
                    <#storage_ident as ::ink::reflect::DispatchableMessageInfo< #id >>::MUTATES
                );

                let any_message_accepts_payment =
                    self.any_message_accepts_payment(messages);

                quote_spanned!(message_span=>
                    #( #cfg_attrs )*
                    Self::#message_ident(input) => {
                        if #any_message_accepts_payment && #deny_payment {
                            ::ink::codegen::deny_payment()?;
                        }

                        let result: #message_output = #message_callable(&mut contract, input);
                        let is_reverted = ::ink::is_result_type!(#message_output)
                            && ::ink::is_result_err!(result);

                        // NOTE: we can't use an if/else expression here
                        // It fails inside quote_spanned! macro.
                        // See https://github.com/rust-lang/rust-clippy/issues/6249
                        let mut flag = ::ink::env::ReturnFlags::REVERT;

                        // no need to push back results: transaction gets reverted anyway
                        if !is_reverted {
                            flag = ::ink::env::ReturnFlags::empty();
                            push_contract(contract, #mutates_storage);
                        }

                        #message_return(flag, result);

                        #[cfg(feature = "std")]
                        return ::core::result::Result::Ok(());

                        #[cfg(not(feature = "std"))]
                        #[cfg_attr(not(feature = "std"), allow(unreachable_code))]
                        {
                            ::core::unreachable!("either `return_value` or the `return` before will already have returned");
                        }
                    }
                )
        });

        quote_spanned!(span=>
            const _: () = {
                #[allow(non_camel_case_types)]
                pub enum __ink_MessageDecoder {
                    #( #message_variants ),*
                }

                impl ::ink::env::DecodeDispatch for __ink_MessageDecoder {
                    fn decode_dispatch(input: &mut &[::core::primitive::u8])
                        -> ::core::result::Result<Self, ::ink::env::DispatchError> {
                        #(
                            #message_selector
                        )*
                        match <[::core::primitive::u8; 4usize] as ::ink::scale::Decode>::decode(input)
                            .map_err(|_| ::ink::env::DispatchError::InvalidSelector)?
                        {
                            #( #message_match , )*
                            _invalid => #possibly_wildcard_selector_message
                        }
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
                    #[allow(clippy::nonminimal_bool, clippy::let_unit_value, dead_code)]
                    fn execute_dispatchable(
                        self
                    ) -> ::core::result::Result<(), ::ink::env::DispatchError> {
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
    /// Generates code in the form of variable assignments
    /// which can be conditionally omitted
    /// in which case the default assignment `let message_{id} = false` exists.
    ///
    /// This information can be used to speed-up dispatch since denying of payment
    /// can be generalized to work before dispatch happens if none of the ink! messages
    /// accept payment anyways.
    fn any_message_accepts_payment(
        &self,
        messages: &[MessageDispatchable],
    ) -> TokenStream2 {
        let span = self.contract.module().storage().span();
        let storage_ident = self.contract.module().storage().ident();
        let message_is_payable =  messages
            .iter()
            .enumerate()
            .map(|(index, item)| {
            let message_span = item.message.span();
            let cfg_attrs = item.message.get_cfg_attrs(message_span);
            let id = item.id.clone();
            let ident = quote::format_ident!("message_{}", index);
            quote_spanned!(message_span=>
                {
                    let #ident = false;
                    #( #cfg_attrs )*
                    let #ident = <#storage_ident as ::ink::reflect::DispatchableMessageInfo< #id >>::PAYABLE;
                    #ident
                }
            )
        });
        quote_spanned!(span=>
            {
                false #( || #message_is_payable )*
            }
        )
    }

    /// Generates code to express if any dispatchable ink! constructor accepts payment.
    ///
    /// Generates code in the form of variable assignments
    /// which can be conditionally omitted
    /// in which case the default assignment `let constructor_{id} = false` exists.
    ///
    /// This information can be used to speed-up dispatch since denying of payment
    /// can be generalized to work before dispatch happens if none of the ink!
    /// constructors accept payment anyways.
    fn any_constructor_accepts_payment(
        &self,
        constructors: &[ConstructorDispatchable],
    ) -> TokenStream2 {
        let span = self.contract.module().storage().span();
        let storage_ident = self.contract.module().storage().ident();
        let constructor_is_payable = constructors.iter().enumerate().map(|(index, item)| {
            let constructor_span = item.constructor.span();
            let cfg_attrs = item.constructor.get_cfg_attrs(constructor_span);
            let id = item.id.clone();
            let ident = quote::format_ident!("constructor_{}", index);
            quote_spanned!(constructor_span=>
                {
                    let #ident = false;
                    #( #cfg_attrs )*
                    let #ident = <#storage_ident as ::ink::reflect::DispatchableConstructorInfo< #id >>::PAYABLE;
                    #ident
                }
            )
        });
        quote_spanned!(span=>
            {
                false #( || #constructor_is_payable )*
            }
        )
    }
}
