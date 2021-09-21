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

//! The global registry with which it is possible to refer back to the global
//! trait call builder and call forwarder types using only the trait identifier.
//!
//! This works by making the global trait registry type defined in the `ink_lang`
//! crate implement each and every ink! trait definition and defining associated
//! types for the trait's respective call builder and call forwarder.

use super::TraitDefinition;
use crate::{
    generator::{self,},
    traits::GenerateCode,
    EnforcedErrors,
};
use derive_more::From;
use ir::HexLiteral;
use proc_macro2::{
    Span,
    TokenStream as TokenStream2,
};
use quote::{
    format_ident,
    quote,
    quote_spanned,
};
use syn::parse_quote;

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
    /// [`ink_lang::TraitCallForwarderRegistry`] type and uses the `__ink_ConcreteImplementer`
    /// associated type to refer back to the actual call forwarder and call builder types.
    fn generate_registry_impl(&self) -> TokenStream2 {
        let span = self.span();
        let name = self.trait_ident();
        let trait_info_ident = self.trait_def.trait_info_ident();
        let messages = self.generate_registry_messages();
        quote_spanned!(span=>
            impl<E> #name for ::ink_lang::TraitCallForwarderRegistry<E>
            where
                E: ::ink_env::Environment,
            {
                /// Holds general and global information about the trait.
                #[doc(hidden)]
                #[allow(non_camel_case_types)]
                type __ink_TraitInfo = #trait_info_ident<E>;

                #messages
            }
        )
    }

    /// Generate the code for all ink! trait messages implemented by the trait registry.
    fn generate_registry_messages(&self) -> TokenStream2 {
        let messages = self.trait_def.trait_def.item().iter_items().filter_map(
            |(item, selector)| {
                item.filter_map_message()
                    .map(|message| self.generate_registry_for_message(&message, selector))
            },
        );
        quote! {
            #( #messages )*
        }
    }

    /// Generate the code for a single ink! trait message implemented by the trait registry.
    ///
    /// Generally the implementation of any ink! trait of the ink! trait registry
    fn generate_registry_for_message(
        &self,
        message: &ir::InkTraitMessage,
        selector: ir::Selector,
    ) -> TokenStream2 {
        let span = message.span();
        let ident = message.ident();
        let attrs = message.attrs();
        let output_ident = generator::output_ident(message.ident());
        let output_type = message
            .output()
            .cloned()
            .unwrap_or_else(|| parse_quote! { () });
        let mut_token = message.receiver().is_ref_mut().then(|| quote! { mut });
        let (input_bindings, input_types) =
            Self::input_bindings_and_types(message.inputs());
        let linker_error_ident = EnforcedErrors::cannot_call_trait_message(
            self.trait_ident(),
            message.ident(),
            selector,
            message.mutates(),
        );
        quote_spanned!(span =>
            type #output_ident = #output_type;

            #( #attrs )*
            #[cold]
            #[doc(hidden)]
            fn #ident(
                & #mut_token self
                #( , #input_bindings : #input_types )*
            ) -> Self::#output_ident {
                /// We enforce linking errors in case this is ever actually called.
                /// These linker errors are properly resolved by the cargo-contract tool.
                extern "C" {
                    fn #linker_error_ident() -> !;
                }
                unsafe { #linker_error_ident() }
            }
        )
    }

    /// Returns a pair of input bindings `__ink_bindings_N` and types.
    fn input_bindings_and_types(
        inputs: ir::InputsIter,
    ) -> (Vec<syn::Ident>, Vec<&syn::Type>) {
        inputs
            .enumerate()
            .map(|(n, pat_type)| {
                let binding = format_ident!("__ink_binding_{}", n);
                let ty = &*pat_type.ty;
                (binding, ty)
            })
            .unzip()
    }

    /// Phantom type that implements the following traits for every ink! trait:
    ///
    /// - `ink_lang::TraitImplementer` (unsafe implementation)
    /// - `ink_lang::TraitUniqueId`
    /// - `ink_lang::TraitCallForwarder`
    ///
    /// It is mainly used to access global information about the ink! trait.
    fn generate_trait_info_object(&self) -> TokenStream2 {
        let span = self.span();
        let trait_ident = self.trait_ident();
        let unique_id = self.trait_def.trait_def.id().hex_padded_suffixed();
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

            unsafe impl<E> ::ink_lang::TraitImplementer<#unique_id>
                for #trait_info_ident<E>
            where
                E: ::ink_env::Environment,
            {
            }

            impl<E> ::ink_lang::TraitUniqueId for #trait_info_ident<E>
            where
                E: ::ink_env::Environment,
            {
                const ID: ::core::primitive::u32 = #unique_id;
            }

            impl<E> ::ink_lang::TraitModulePath for #trait_info_ident<E>
            where
                E: ::ink_env::Environment,
            {
                const PATH: &'static ::core::primitive::str = ::core::module_path!();

                const NAME: &'static ::core::primitive::str = ::core::stringify!(#trait_ident);
            }

            impl<E> ::ink_lang::TraitCallForwarder for #trait_info_ident<E>
            where
                E: ::ink_env::Environment,
            {
                type Forwarder = #trait_call_forwarder<E>;
            }
        )
    }

    /// Generates the [`::ink_lang::TraitMessageInfo`] implementations for all
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

    /// Generates the [`::ink_lang::TraitMessageInfo`] implementation for a single
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
            impl<E> ::ink_lang::TraitMessageInfo<#local_id> for #trait_info_ident<E> {
                const PAYABLE: ::core::primitive::bool = #is_payable;

                const SELECTOR: [::core::primitive::u8; 4usize] = [ #( #selector_bytes ),* ];
            }
        )
    }
}
