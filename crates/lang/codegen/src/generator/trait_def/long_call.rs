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

//! Generates code to implement an ink! trait definition for the concretized
//! type using the more elaborate and customizable long-hand calling notation.

#![allow(dead_code)]

use super::TraitDefinition;
use heck::CamelCase as _;
use impl_serde::serialize as serde_hex;
use ir::TraitItemInputsIter;
use proc_macro2::TokenStream as TokenStream2;
use quote::{
    format_ident,
    quote,
    quote_spanned,
};

/// Errors which may occur when forwarding a call is not allowed.
///
/// We insert markers for these errors in the generated contract code.
/// This is necessary since we can't check these errors at compile time
/// of the contract.
/// `cargo-contract` checks the contract code for these error markers
/// when building a contract and fails if it finds markers.
#[derive(scale::Encode, scale::Decode)]
pub enum EnforcedErrors {
    /// The below error represents calling a `&mut self` message in a context that
    /// only allows for `&self` messages. This may happen under certain circumstances
    /// when ink! trait implementations are involved with long-hand calling notation.
    #[codec(index = 1)]
    CannotCallTraitMessage {
        /// The trait that defines the called message.
        trait_ident: String,
        /// The name of the called message.
        message_ident: String,
        /// The selector of the called message.
        message_selector: [u8; 4],
        /// Is `true` if the `self` receiver of the ink! message is `&mut self`.
        message_mut: bool,
    },
    /// The below error represents calling a constructor in a context that does
    /// not allow calling it. This may happen when the constructor defined in a
    /// trait is cross-called in another contract.
    /// This is not allowed since the contract to which a call is forwarded must
    /// already exist at the point when the call to it is made.
    #[codec(index = 2)]
    CannotCallTraitConstructor {
        /// The trait that defines the called constructor.
        trait_ident: String,
        /// The name of the called constructor.
        constructor_ident: String,
        /// The selector of the called constructor.
        constructor_selector: [u8; 4],
    },
}

impl EnforcedErrors {
    /// Create the identifier of an enforced ink! compilation error.
    fn enforce_error_ident(&self) -> syn::Ident {
        format_ident!(
            "__ink_enforce_error_{}",
            serde_hex::to_hex(&scale::Encode::encode(&self), false)
        )
    }
}

impl<'a> TraitDefinition<'a> {
    /// Returns the identifier for the generated call forwarder utility.
    fn call_forwarder_ident() -> syn::Ident {
        format_ident!("__ink_CallForwarder")
    }

    /// Returns the identifier for the generated `Out*` assoc. type.
    fn out_assoc_type_ident(method_ident: &syn::Ident) -> syn::Ident {
        format_ident!("{}Out", method_ident.to_string().to_camel_case())
    }

    /// Returns the sequence of input parameter bindings for the message.
    fn input_bindings(inputs: TraitItemInputsIter) -> Vec<syn::Ident> {
        inputs
            .enumerate()
            .map(|(n, _)| format_ident!("__ink_binding_{}", n))
            .collect::<Vec<_>>()
    }

    /// Returns the sequence of input types for the message.
    fn input_types<'b>(inputs: TraitItemInputsIter<'b>) -> Vec<&'b syn::Type> {
        inputs.map(|pat_type| &*pat_type.ty).collect::<Vec<_>>()
    }

    /// Create an enforced ink! message error.
    fn create_enforced_message_error(
        &self,
        message: &ir::InkTraitMessage,
        selector: ir::Selector,
    ) -> syn::Ident {
        let trait_ident = self.trait_def.ident();
        EnforcedErrors::CannotCallTraitMessage {
            trait_ident: trait_ident.to_string(),
            message_ident: message.ident().to_string(),
            message_selector: selector.as_bytes().to_owned(),
            message_mut: message.receiver().is_ref_mut(),
        }
        .enforce_error_ident()
    }

    /// Create an enforced ink! message error.
    fn create_enforced_constructor_error(
        &self,
        constructor: &ir::InkTraitConstructor,
        selector: ir::Selector,
    ) -> syn::Ident {
        let trait_ident = self.trait_def.ident();
        EnforcedErrors::CannotCallTraitConstructor {
            trait_ident: trait_ident.to_string(),
            constructor_ident: constructor.ident().to_string(),
            constructor_selector: selector.as_bytes().to_owned(),
        }
        .enforce_error_ident()
    }

    /// Generate code for cross-calling an invalid ink! trait message.
    ///
    /// Trying to call the generated message will always yield a link-time
    /// error which is caught by the `cargo-contract` CLI tool.
    ///
    /// # Note
    ///
    /// This is implemented for the call forwarder in case the mutability
    /// overlaps with the message's mutability.
    fn generate_ghost_message(
        &self,
        message: ir::InkTraitMessage,
        selector: ir::Selector,
    ) -> TokenStream2 {
        let span = message.span();
        let ident = message.ident();
        let output_ident = Self::out_assoc_type_ident(ident);
        let attrs = message.attrs();
        let mut_tok = message.mutates().then(|| quote! { mut });
        let input_bindings = Self::input_bindings(message.inputs());
        let input_types = Self::input_types(message.inputs());
        let enforced_error = self.create_enforced_message_error(&message, selector);
        let output_ty = message
            .output()
            .cloned()
            .unwrap_or_else(|| syn::parse_quote! { () });
        quote_spanned!(span=>
            type #output_ident = #output_ty;

            #( #attrs )*
            #[cold]
            #[doc(hidden)]
            fn #ident(
                & #mut_tok self,
                #( #input_bindings : #input_types ),*
            ) -> Self::#output_ident {
                extern {
                    fn #enforced_error() -> !;
                }
                unsafe { #enforced_error() }
            }
        )
    }

    /// Generate code for cross-calling an ink! trait message.
    ///
    /// # Note
    ///
    /// This is implemented for the call forwarder in case the mutability
    /// overlaps with the message's mutability.
    fn generate_proper_message(
        &self,
        message: ir::InkTraitMessage,
        selector: ir::Selector,
    ) -> TokenStream2 {
        let span = message.span();
        let ident = message.ident();
        let output_ident = Self::out_assoc_type_ident(ident);
        let attrs = message.attrs();
        let mut_tok = message.mutates().then(|| quote! { mut });
        let input_bindings = Self::input_bindings(message.inputs());
        let input_types = Self::input_types(message.inputs());
        let selector_bytes = selector.as_bytes();
        let arg_list = Self::generate_arg_list(input_types.iter().cloned());
        let output = message.output();
        let output_sig = output.map_or_else(
            || quote! { () },
            |output| quote! { ::ink_env::call::utils::ReturnType<#output> },
        );
        quote_spanned!(span=>
            #[allow(clippy::type_complexity)]
            type #output_ident = ::ink_env::call::CallBuilder<
                Environment,
                ::ink_env::call::utils::Set<AccountId>,
                ::ink_env::call::utils::Unset<u64>,
                ::ink_env::call::utils::Unset<Balance>,
                ::ink_env::call::utils::Set<::ink_env::call::ExecutionInput<#arg_list>>,
                ::ink_env::call::utils::Set<#output_sig>,
            >;

            #( #attrs )*
            #[inline]
            fn #ident(
                & #mut_tok #(, #input_bindings : #input_types )*
            ) -> Self::#output_ident {
                ::ink_env::call::build_call::<Environment>()
                    .callee(::ink_lang::ToAccountId::to_account_id(self.contract))
                    .exec_input(
                        ::ink_env::call::ExecutionInput::new(
                            ::ink_env::call::Selector::new([ #( #selector_bytes ),* ])
                        )
                        #(
                            .push_arg(#input_bindings)
                        )*
                    )
                    .returns::<#output_sig>()
            }
        )
    }

    /// Generates code for a single call forwarder trait message.
    ///
    /// The `mutable` parameter indicates whether only read-only (`false`) or
    /// write-only (`true`) messages shall be valid calls. For non valid messages
    /// an invalid implementation is provided so that actually calling those
    /// will result in a compiler or linker error.
    fn generate_proper_or_ghost_message(
        &self,
        mutable: bool,
        message: ir::InkTraitMessage,
        selector: ir::Selector,
    ) -> TokenStream2 {
        if mutable == message.receiver().is_ref_mut() {
            self.generate_proper_message(message, selector)
        } else {
            self.generate_ghost_message(message, selector)
        }
    }

    /// Generates code for a single call forwarder trait constructor.
    ///
    /// Note that constructors never need to be forwarded and that we only
    /// provide their implementations to satisfy the implementation block.
    /// We generally try to generate code in a way that actually calling
    /// those constructors will result in a compiler or linker error.
    fn generate_ghost_constructor(
        &self,
        constructor: ir::InkTraitConstructor,
        selector: ir::Selector,
    ) -> TokenStream2 {
        let span = constructor.span();
        let attrs = constructor.attrs();
        let ident = constructor.ident();
        let output_ident = format_ident!("{}Out", ident.to_string().to_camel_case());
        let enforced_error =
            self.create_enforced_constructor_error(&constructor, selector);
        let input_bindings = Self::input_bindings(constructor.inputs());
        let input_types = Self::input_types(constructor.inputs());
        quote_spanned!(span =>
            type #output_ident = ::ink_lang::NeverReturns;

            #( #attrs )*
            #[cold]
            #[doc(hidden)]
            fn #ident(
                #( #input_bindings : #input_types ),*
            ) -> Self::#output_ident {
                extern {
                    fn #enforced_error() -> !;
                }
                unsafe { #enforced_error() }
            }
        )
    }

    /// Generates code for a single call forwarder trait implementation block.
    ///
    /// The `mutable` parameter indicates whether only read-only (`false`) or
    /// write-only (`true`) messages and constructors are to be considered.
    fn generate_trait_impl_longhand(&self, mutable: bool) -> TokenStream2 {
        let span = self.trait_def.span();
        let attrs = self.trait_def.attrs();
        let forwarder_ident = Self::call_forwarder_ident();
        let verify_hash = self.trait_def.verify_hash();
        let checksum = u32::from_be_bytes([
            verify_hash[0],
            verify_hash[1],
            verify_hash[2],
            verify_hash[3],
        ]) as usize;
        let trait_ident = self.trait_def.ident();
        let self_ident = self.concretizer_ident();
        let mut_tok = mutable.then(|| quote! { mut });
        let messages = self.trait_def.iter_items().filter_map(|(item, selector)| {
            item.filter_map_message().map(|message| {
                self.generate_proper_or_ghost_message(mutable, message, selector)
            })
        });
        let constructors = self.trait_def.iter_items().filter_map(|(item, selector)| {
            item.filter_map_constructor()
                .map(|constructor| self.generate_ghost_constructor(constructor, selector))
        });
        quote_spanned!(span =>
            unsafe impl<'a> ::ink_lang::CheckedInkTrait<[(); #checksum]> for #forwarder_ident<&'a #mut_tok #self_ident> {}

            #( #attrs )*
            impl<'a> #trait_ident for #forwarder_ident<&'a #mut_tok #self_ident> {
                type __ink_Checksum = [(); #checksum];

                #( #constructors )*
                #( #messages )*
            }
        )
    }
}
