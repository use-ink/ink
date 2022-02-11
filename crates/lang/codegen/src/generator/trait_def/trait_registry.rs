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

//! The global registry with which it is possible to refer back to the global
//! trait call builder and call forwarder types using only the trait identifier.
//!
//! This works by making the global trait registry type defined in the `ink_lang`
//! crate implement each and every ink! trait definition and defining associated
//! types for the trait's respective call builder and call forwarder.

use super::TraitDefinition;
use crate::{
    generator,
    traits::GenerateCode,
};
use derive_more::From;
use proc_macro2::{
    Span,
    TokenStream as TokenStream2,
};
use quote::{
    quote,
    quote_spanned,
};
use syn::spanned::Spanned;

impl<'a> TraitDefinition<'a> {
    /// Generates the code for the global trait registry implementation.
    ///
    /// This also generates the code for the global trait info object which
    /// implements some `ink_lang` traits to provide common information about
    /// the ink! trait definition such as its unique identifier.
    pub fn generate_trait_registry_impl(&self) -> TokenStream2 {
        TraitRegistry::from(*self).generate_code()
    }

    /// Returns the identifier for the ink! trait definition info object.
    pub fn trait_info_ident(&self) -> syn::Ident {
        self.append_trait_suffix("TraitInfo")
    }
}

/// Generates code for the global ink! trait registry implementation.
#[derive(From)]
struct TraitRegistry<'a> {
    trait_def: TraitDefinition<'a>,
}

impl GenerateCode for TraitRegistry<'_> {
    fn generate_code(&self) -> TokenStream2 {
        let registry_impl = self.generate_registry_impl();
        let trait_info = self.generate_trait_info_object();
        quote! {
            #registry_impl
            #trait_info
        }
    }
}

impl TraitRegistry<'_> {
    /// Returns the span of the ink! trait definition.
    fn span(&self) -> Span {
        self.trait_def.span()
    }

    /// Returns the identifier of the ink! trait definition.
    fn trait_ident(&self) -> &syn::Ident {
        self.trait_def.trait_def.item().ident()
    }

    /// Generates the global trait registry implementation for the ink! trait.
    ///
    /// This makes it possible to refer back to the global call forwarder and
    /// call builder specific to this ink! trait from anywhere with just the Rust
    /// trait identifier which allows for type safe access.
    ///
    /// # Note
    ///
    /// Through this implementation we register the previously defined ink! trait
    /// call forwarder and call builder types as such for the ink! trait.
    ///
    /// This is done by the fact that ink! implements all ink! traits by the
    /// [`ink_lang::TraitDefinitionRegistry`] type and uses the `__ink_ConcreteImplementer`
    /// associated type to refer back to the actual call forwarder and call builder types.
    fn generate_registry_impl(&self) -> TokenStream2 {
        let span = self.span();
        let trait_ident = self.trait_ident();
        let trait_info_ident = self.trait_def.trait_info_ident();
        let message_impls = self.generate_ink_trait_impl_messages();
        quote_spanned!(span=>
            /// The blanket implementation of the forwarder to do cross-contract
            /// calls without customization. It only fires the call via a SEAL host function
            /// and checks that dispatch result is not an error.
            ///
            /// # Note
            ///
            /// That implementation is used by builder generated in the body of the contract
            /// and by reference to contract (a.k.a `ContractRef` in case of `Contract`).
            impl<T> #trait_ident for T
            where
                T: ::ink_lang::reflect::CallBuilder,
            {
                #[doc(hidden)]
                #[allow(non_camel_case_types)]
                type __ink_TraitInfo = #trait_info_ident<<Self as ::ink_lang::reflect::ContractEnv>::Env>;

                #message_impls
            }
        )
    }

    /// Phantom type that implements the following traits for every ink! trait:
    ///
    /// - `ink_lang::TraitCallForwarder`
    ///
    /// It is mainly used to access global information about the ink! trait.
    fn generate_trait_info_object(&self) -> TokenStream2 {
        let span = self.span();
        let trait_ident = self.trait_ident();
        let trait_info_ident = self.trait_def.trait_info_ident();
        let trait_call_forwarder = self.trait_def.call_forwarder_ident();
        let trait_message_info = self.generate_info_for_trait_messages();
        quote_spanned!(span =>
            #[doc(hidden)]
            #[allow(non_camel_case_types)]
            pub struct #trait_info_ident<E> {
                marker: ::core::marker::PhantomData<fn() -> E>,
            }

            #trait_message_info

            impl<E> ::ink_lang::reflect::TraitModulePath for #trait_info_ident<E>
            where
                E: ::ink_env::Environment,
            {
                const PATH: &'static ::core::primitive::str = ::core::module_path!();

                const NAME: &'static ::core::primitive::str = ::core::stringify!(#trait_ident);
            }

            impl<E> ::ink_lang::codegen::TraitCallForwarder for #trait_info_ident<E>
            where
                E: ::ink_env::Environment,
            {
                type Forwarder = #trait_call_forwarder<E>;
            }
        )
    }

    /// Generates the [`::ink_lang::reflect::TraitMessageInfo`] implementations for all
    /// ink! messages defined by the ink! trait definition.
    fn generate_info_for_trait_messages(&self) -> TokenStream2 {
        let span = self.span();
        let message_impls = self.trait_def.trait_def.item().iter_items().filter_map(
            |(trait_item, selector)| {
                trait_item.filter_map_message().map(|message| {
                    self.generate_info_for_trait_for_message(&message, selector)
                })
            },
        );
        quote_spanned!(span=>
            #( #message_impls )*
        )
    }

    /// Generates the [`::ink_lang::reflect::TraitMessageInfo`] implementation for a single
    /// ink! message defined by the ink! trait definition.
    fn generate_info_for_trait_for_message(
        &self,
        message: &ir::InkTraitMessage,
        selector: ir::Selector,
    ) -> TokenStream2 {
        let span = message.span();
        let trait_info_ident = self.trait_def.trait_info_ident();
        let local_id = message.local_id();
        let selector_bytes = selector.hex_lits();
        let is_payable = message.ink_attrs().is_payable();
        quote_spanned!(span=>
            impl<E> ::ink_lang::reflect::TraitMessageInfo<#local_id> for #trait_info_ident<E> {
                const PAYABLE: ::core::primitive::bool = #is_payable;

                const SELECTOR: [::core::primitive::u8; 4usize] = [ #( #selector_bytes ),* ];
            }
        )
    }

    /// Generate the code for all ink! trait messages implemented by the trait.
    fn generate_ink_trait_impl_messages(&self) -> TokenStream2 {
        let messages =
            self.trait_def
                .trait_def
                .item()
                .iter_items()
                .filter_map(|(item, _)| {
                    item.filter_map_message()
                        .map(|message| self.generate_ink_trait_impl_for_message(&message))
                });
        quote! {
            #( #messages )*
        }
    }

    /// Generate the code for a single ink! trait message implemented by the trait.
    fn generate_ink_trait_impl_for_message(
        &self,
        message: &ir::InkTraitMessage,
    ) -> TokenStream2 {
        let span = message.span();
        let trait_ident = self.trait_ident();
        let forwarder_ident = self.trait_def.call_forwarder_ident();
        let message_ident = message.ident();
        let attrs = message.attrs();
        let output_ident = generator::output_ident(message_ident);
        let output_type = message
            .output()
            .cloned()
            .unwrap_or_else(|| syn::parse_quote!(()));

        let input_bindings = message.inputs().map(|input| &input.pat).collect::<Vec<_>>();
        let input_types = message.inputs().map(|input| &input.ty).collect::<Vec<_>>();

        let inout_guards = Self::generate_inout_guards_for_message(message);

        let mut_tok = message.mutates().then(|| quote! { mut });
        let panic_str = format!(
            "encountered error while calling <{} as {}>::{}",
            forwarder_ident, trait_ident, message_ident,
        );
        let builder_ident = self.trait_def.call_builder_ident();
        let env_type = quote! { <Self as ::ink_lang::reflect::ContractEnv>::Env };
        quote_spanned!(span =>
            type #output_ident = #output_type;

            #( #attrs )*
            #[inline]
            fn #message_ident(
                & #mut_tok self
                #( , #input_bindings : #input_types )*
            ) -> Self::#output_ident {
                #inout_guards
                <#builder_ident<#env_type> as #trait_ident>::#message_ident(
                    & #mut_tok <#builder_ident<#env_type> as ::ink_env::call::FromAccountId<#env_type>>::from_account_id(
                        <Self as ::ink_lang::ToAccountId<#env_type>>::to_account_id(self)
                    )
                    #(
                        , #input_bindings
                    )*
                )
                    .fire()
                    .unwrap_or_else(|err| ::core::panic!("{}: {:?}", #panic_str, err))
            }
        )
    }

    /// Generates code to assert that ink! input and output types meet certain properties.
    fn generate_inout_guards_for_message(message: &ir::InkTraitMessage) -> TokenStream2 {
        let message_span = message.span();
        let message_inputs = message.inputs().map(|input| {
            let input_span = input.span();
            let input_type = &*input.ty;
            quote_spanned!(input_span=>
                let _: () = ::ink_lang::codegen::utils::consume_type::<
                    ::ink_lang::codegen::DispatchInput<#input_type>
                >();
            )
        });
        let message_output = message.output().map(|output_type| {
            let output_span = output_type.span();
            quote_spanned!(output_span=>
                let _: () = ::ink_lang::codegen::utils::consume_type::<
                    ::ink_lang::codegen::DispatchOutput<#output_type>
                >();
            )
        });
        quote_spanned!(message_span=>
            #( #message_inputs )*
            #message_output
        )
    }
}
