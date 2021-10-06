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

use crate::GenerateCode;
use derive_more::From;
use heck::CamelCase as _;
use ir::{
    Callable as _,
    HexLiteral,
};
use proc_macro2::TokenStream as TokenStream2;
use quote::{
    format_ident,
    quote,
    quote_spanned,
    ToTokens,
};
use syn::spanned::Spanned as _;

/// Generates code for all ink! implementation blocks.
#[derive(From)]
pub struct ItemImpls<'a> {
    contract: &'a ir::Contract,
}
impl_as_ref_for_generator!(ItemImpls);

impl GenerateCode for ItemImpls<'_> {
    fn generate_code(&self) -> TokenStream2 {
        let item_impls = self
            .contract
            .module()
            .impls()
            .map(|item_impl| self.generate_item_impl(item_impl));
        let inout_guards = self.generate_input_output_guards();
        let trait_message_property_guards = self.generate_trait_message_property_guards();
        let use_emit_event =
            self.contract.module().events().next().is_some().then(|| {
                // Required to make `self.env().emit_event(..)` syntax available.
                quote! { use ::ink_lang::codegen::EmitEvent as _; }
            });
        quote! {
            const _: () = {
                // Required to make `self.env()` and `Self::env()` syntax available.
                use ::ink_lang::codegen::{Env as _, StaticEnv as _};
                #use_emit_event

                #( #item_impls )*
                #inout_guards
                #trait_message_property_guards
            };
        }
    }
}

impl ItemImpls<'_> {
    /// Generates code to guard annotated ink! trait message properties.
    ///
    /// These guarded properties include `selector` and `payable`.
    /// If an ink! trait message is annotated with `#[ink(payable)]`
    /// or `#[ink(selector = ..)]` then code is generated to guard that
    /// the given argument to `payable` or `selector` is equal to
    /// what the associated ink! trait definition defines for the same
    /// ink! message.
    fn generate_trait_message_property_guards(&self) -> TokenStream2 {
        let storage_span = self.contract.module().storage().span();
        let storage_ident = self.contract.module().storage().ident();
        let trait_message_guards = self
            .contract
            .module()
            .impls()
            .filter_map(|item_impl| item_impl.trait_path().map(|trait_path| {
                iter::repeat(trait_path).zip(item_impl.iter_messages())
            }))
            .flatten()
            .map(|(trait_path, message)| {
                let message_span = message.span();
                let message_local_id = message.local_id().hex_padded_suffixed();
                let message_guard_payable = message.is_payable().then(|| {
                    quote_spanned!(message_span=>
                        const _: ::ink_lang::codegen::TraitMessagePayable<{
                            <<::ink_lang::reflect::TraitDefinitionRegistry<<#storage_ident as ::ink_lang::reflect::ContractEnv>::Env>
                                as #trait_path>::__ink_TraitInfo
                                as ::ink_lang::reflect::TraitMessageInfo<#message_local_id>>::PAYABLE
                        }> = ::ink_lang::codegen::TraitMessagePayable::<true>;
                    )
                });
                let message_guard_selector = message.user_provided_selector().map(|selector| {
                    let given_selector = selector.into_be_u32().hex_padded_suffixed();
                    quote_spanned!(message_span=>
                        const _: ::ink_lang::codegen::TraitMessageSelector<{
                            ::core::primitive::u32::from_be_bytes(
                                <<::ink_lang::reflect::TraitDefinitionRegistry<<#storage_ident as ::ink_lang::reflect::ContractEnv>::Env>
                                    as #trait_path>::__ink_TraitInfo
                                    as ::ink_lang::reflect::TraitMessageInfo<#message_local_id>>::SELECTOR
                            )
                        }> = ::ink_lang::codegen::TraitMessageSelector::<#given_selector>;
                    )
                });
                quote_spanned!(message_span=>
                    #message_guard_payable
                    #message_guard_selector
                )
            });
        quote_spanned!(storage_span=>
            #( #trait_message_guards )*
        )
    }

    /// Generates code to assert that ink! input and output types meet certain properties.
    fn generate_input_output_guards(&self) -> TokenStream2 {
        let storage_span = self.contract.module().storage().span();
        let constructor_input_guards = self
            .contract
            .module()
            .impls()
            .map(|item_impl| item_impl.iter_constructors())
            .flatten()
            .map(|constructor| {
                let constructor_span = constructor.span();
                let constructor_inputs = constructor.inputs().map(|input| {
                    let span = input.span();
                    let input_type = &*input.ty;
                    quote_spanned!(span=>
                        let _: () = ::ink_lang::codegen::utils::identity_type::<
                            ::ink_lang::codegen::DispatchInput<#input_type>
                        >();
                    )
                });
                quote_spanned!(constructor_span=>
                    #( #constructor_inputs )*
                )
            });
        let message_inout_guards = self
            .contract
            .module()
            .impls()
            .map(|item_impl| item_impl.iter_messages())
            .flatten()
            .map(|message| {
                let message_span = message.span();
                let message_inputs = message.inputs().map(|input| {
                    let span = input.span();
                    let input_type = &*input.ty;
                    quote_spanned!(span=>
                        let _: () = ::ink_lang::codegen::utils::identity_type::<
                            ::ink_lang::codegen::DispatchInput<#input_type>
                        >();
                    )
                });
                let message_output = message.output().map(|output_type| {
                    let span = output_type.span();
                    quote_spanned!(span=>
                        let _: () = ::ink_lang::codegen::utils::identity_type::<
                            ::ink_lang::codegen::DispatchOutput<#output_type>
                        >();
                    )
                });
                quote_spanned!(message_span=>
                    #( #message_inputs )*
                    #message_output
                )
            });
        quote_spanned!(storage_span=>
            const _: () = {
                #( #constructor_input_guards )*
                #( #message_inout_guards )*
            };
        )
    }

    /// Generates the code for the given ink! message within a trait implementation block.
    fn generate_trait_message(message: &ir::Message) -> TokenStream2 {
        let span = message.span();
        let attrs = message.attrs();
        let vis = message.visibility();
        let receiver = message.receiver();
        let ident = message.ident();
        let output_ident = format_ident!("{}Output", ident.to_string().to_camel_case());
        let inputs = message.inputs();
        let output = message
            .output()
            .cloned()
            .unwrap_or_else(|| syn::parse_quote! { () });
        let statements = message.statements();
        quote_spanned!(span =>
            type #output_ident = #output;

            #( #attrs )*
            #vis fn #ident(#receiver #( , #inputs )* ) -> Self::#output_ident {
                #( #statements )*
            }
        )
    }

    fn generate_trait_item_impl(item_impl: &ir::ItemImpl) -> TokenStream2 {
        assert!(item_impl.trait_path().is_some());
        let span = item_impl.span();
        let attrs = item_impl.attrs();
        let messages = item_impl
            .iter_messages()
            .map(|cws| Self::generate_trait_message(cws.callable()));
        let trait_path = item_impl
            .trait_path()
            .expect("encountered missing trait path for trait impl block");
        let self_type = item_impl.self_type();
        quote_spanned!(span =>
            #( #attrs )*
            impl #trait_path for #self_type {
                type __ink_TraitInfo = <::ink_lang::reflect::TraitDefinitionRegistry<Environment>
                    as #trait_path>::__ink_TraitInfo;

                #( #messages )*
            }
        )
    }

    /// Generates the code for the given ink! constructor within an inherent implementation block.
    fn generate_inherent_constructor(constructor: &ir::Constructor) -> TokenStream2 {
        let span = constructor.span();
        let attrs = constructor.attrs();
        let vis = constructor.visibility();
        let ident = constructor.ident();
        let inputs = constructor.inputs();
        let statements = constructor.statements();
        quote_spanned!(span =>
            #( #attrs )*
            #vis fn #ident( #( #inputs ),* ) -> Self {
                #( #statements )*
            }
        )
    }

    /// Generates the code for the given ink! message within an inherent implementation block.
    fn generate_inherent_message(message: &ir::Message) -> TokenStream2 {
        let span = message.span();
        let attrs = message.attrs();
        let vis = message.visibility();
        let receiver = message.receiver();
        let ident = message.ident();
        let inputs = message.inputs();
        let output_arrow = message.output().map(|_| quote! { -> });
        let output = message.output();
        let statements = message.statements();
        quote_spanned!(span =>
            #( #attrs )*
            #vis fn #ident(#receiver #( , #inputs )* ) #output_arrow #output {
                #( #statements )*
            }
        )
    }

    fn generate_inherent_item_impl(item_impl: &ir::ItemImpl) -> TokenStream2 {
        assert!(item_impl.trait_path().is_none());
        let span = item_impl.span();
        let attrs = item_impl.attrs();
        let messages = item_impl
            .iter_messages()
            .map(|cws| Self::generate_inherent_message(cws.callable()));
        let constructors = item_impl
            .iter_constructors()
            .map(|cws| Self::generate_inherent_constructor(cws.callable()));
        let other_items = item_impl
            .items()
            .iter()
            .filter_map(ir::ImplItem::filter_map_other_item)
            .map(ToTokens::to_token_stream);
        let self_type = item_impl.self_type();
        quote_spanned!(span =>
            #( #attrs )*
            impl #self_type {
                #( #constructors )*
                #( #messages )*
                #( #other_items )*
            }
        )
    }

    /// Generates code to guard against ink! implementations that have not been implemented
    /// for the ink! storage struct.
    fn generate_item_impl_self_ty_guard(&self, item_impl: &ir::ItemImpl) -> TokenStream2 {
        let self_ty = item_impl.self_type();
        let span = self_ty.span();
        let storage_ident = self.contract.module().storage().ident();
        quote_spanned!(span =>
            const _: ::ink_lang::codegen::utils::IsSameType<#storage_ident> =
                ::ink_lang::codegen::utils::IsSameType::<#self_ty>::new();
        )
    }

    /// Generates code for the given ink! implementation block.
    fn generate_item_impl(&self, item_impl: &ir::ItemImpl) -> TokenStream2 {
        let self_ty_guard = self.generate_item_impl_self_ty_guard(item_impl);
        let impl_block = match item_impl.trait_path() {
            Some(_) => Self::generate_trait_item_impl(item_impl),
            None => Self::generate_inherent_item_impl(item_impl),
        };
        quote! {
            #self_ty_guard
            #impl_block
        }
    }
}
