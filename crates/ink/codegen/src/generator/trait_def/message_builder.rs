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

use derive_more::From;
use ink_primitives::abi::Abi;
use proc_macro2::{
    Span,
    TokenStream as TokenStream2,
};
use quote::{
    quote,
    quote_spanned,
};

use super::TraitDefinition;
use crate::{
    generator,
    generator::sol,
    traits::GenerateCode,
};

impl TraitDefinition<'_> {
    /// Generates code for the global trait call builder for an ink! trait.
    ///
    /// # Note
    ///
    /// - The generated call builder type implements the ink! trait definition and allows
    ///   to build up contract calls that allow for customization by the user to provide
    ///   gas limit, endowment etc.
    /// - The call builder is used directly by the generated call forwarder. There exists
    ///   one global call forwarder and call builder pair for every ink! trait definition.
    pub fn generate_message_builder(&self) -> TokenStream2 {
        MessageBuilder::from(*self).generate_code()
    }

    /// The identifier of the ink! trait message builder.
    pub fn message_builder_ident(&self) -> syn::Ident {
        self.append_trait_suffix(MessageBuilder::SUFFIX)
    }
}

/// Generates code for the global ink! trait call builder.
#[derive(From)]
struct MessageBuilder<'a> {
    trait_def: TraitDefinition<'a>,
}

impl GenerateCode for MessageBuilder<'_> {
    fn generate_code(&self) -> TokenStream2 {
        let struct_definition = self.generate_struct_definition();
        let auxiliary_trait_impls = self.generate_auxiliary_trait_impls();
        let ink_trait_impl = self.generate_ink_trait_impl();
        quote! {
            #struct_definition
            #auxiliary_trait_impls
            #ink_trait_impl
        }
    }
}

impl MessageBuilder<'_> {
    /// The name suffix for ink! trait message builder.
    const SUFFIX: &'static str = "TraitMessageBuilder";

    /// Returns the span of the ink! trait definition.
    fn span(&self) -> Span {
        self.trait_def.span()
    }

    /// Generates the struct type definition for the account wrapper type.
    ///
    /// This type is going to implement the trait so that invoking its trait
    /// methods will return an `Execution` instance populated with the message selector,
    /// arguments and return type, which can then be executed via the `exec` method.
    fn generate_struct_definition(&self) -> TokenStream2 {
        let span = self.span();
        let message_builder_ident = self.trait_def.message_builder_ident();
        quote_spanned!(span =>
            /// The global call builder type for all trait implementers.
            ///
            /// All calls to types (contracts) implementing the trait will be built by this type.
            #[doc(hidden)]
            #[allow(non_camel_case_types)]
            #[::ink::scale_derive(Encode, Decode, TypeInfo)]
            pub struct #message_builder_ident<E, Abi> {
                _marker: ::core::marker::PhantomData<fn() -> (E, Abi)>,
            }
        )
    }

    /// Generates trait implementations for auxiliary traits for the message builder.
    ///
    /// # Note
    ///
    /// Auxiliary traits currently include:
    ///
    /// - `Default`: To allow initializing a contract message builder.
    fn generate_auxiliary_trait_impls(&self) -> TokenStream2 {
        let span = self.span();
        let message_builder_ident = self.trait_def.message_builder_ident();
        quote_spanned!(span=>
            impl<E, Abi> ::core::default::Default for #message_builder_ident<E, Abi>
            where
                E: ::ink::env::Environment,
            {
                fn default() -> Self {
                    Self {
                        _marker: ::core::default::Default::default()
                    }
                }
            }

            impl<E, Abi> ::ink::env::ContractEnv for #message_builder_ident<E, Abi>
            where
                E: ::ink::env::Environment,
            {
                type Env = E;
            }
        )
    }

    /// Generates the implementation of the associated ink! trait definition.
    ///
    /// # Note
    ///
    /// The implemented messages create an instance of the `Execution` type which
    /// encapsulates the execution input (the selector and arguments) and the output
    /// type of the message.
    fn generate_ink_trait_impl(&self) -> TokenStream2 {
        let span = self.trait_def.span();
        let trait_ident = self.trait_def.trait_def.item().ident();
        let trait_info_ident = self.trait_def.trait_info_ident();
        let message_builder_ident = self.trait_def.message_builder_ident();
        generate_abi_impls!(@type |abi| {
            let abi_ty = match abi {
                Abi::Ink => quote!(::ink::abi::Ink),
                Abi::Sol => quote!(::ink::abi::Sol),
            };
            let message_impls = self.generate_ink_trait_impl_messages(abi);
            quote_spanned!(span=>
                impl<E> #trait_ident for #message_builder_ident<E, #abi_ty>
                where
                    E: ::ink::env::Environment,
                {
                    #[allow(non_camel_case_types)]
                    type __ink_TraitInfo = #trait_info_ident<E>;

                    #message_impls
                }
            )
        })
    }

    /// Generate the code for all ink! trait messages implemented by the trait call
    /// builder.
    fn generate_ink_trait_impl_messages(&self, abi: Abi) -> TokenStream2 {
        let messages = self.trait_def.trait_def.item().iter_items().filter_map(
            |(item, selector)| {
                item.filter_map_message().map(|message| {
                    let (selector_bytes, abi_ty) = match abi {
                        Abi::Ink => {
                            let selector_bytes = selector.hex_lits();
                            (quote!([ #( #selector_bytes ),* ]), quote!(::ink::abi::Ink))
                        }
                        Abi::Sol => {
                            let message_ident = message.ident();
                            let signature = sol::utils::call_signature(
                                message_ident.to_string(),
                                message.inputs(),
                            );
                            (
                                quote!(::ink::codegen::sol::selector_bytes(#signature)),
                                quote!(::ink::abi::Sol),
                            )
                        }
                    };
                    self.generate_ink_trait_impl_for_message(
                        &message,
                        selector_bytes,
                        abi_ty,
                    )
                })
            },
        );
        quote! {
            #( #messages )*
        }
    }

    /// Generate the code for a single ink! trait message implemented by the trait call
    /// builder.
    fn generate_ink_trait_impl_for_message(
        &self,
        message: &ir::InkTraitMessage,
        selector_bytes: TokenStream2,
        abi: TokenStream2,
    ) -> TokenStream2 {
        let span = message.span();
        let message_ident = message.ident();
        let attrs = self
            .trait_def
            .trait_def
            .config()
            .whitelisted_attributes()
            .filter_attr(message.attrs());
        let output_ident = generator::output_ident(message_ident);
        let output = message.output();
        let output_type =
            output.map_or_else(|| quote! { () }, |output| quote! { #output });
        let input_bindings = generator::input_bindings(message.inputs());
        let input_types = generator::input_types(message.inputs());
        let mut_tok = message.mutates().then(|| quote! { mut });
        let arg_list =
            generator::generate_argument_list(input_types.iter().cloned(), abi.clone());
        let cfg_attrs = message.get_cfg_attrs(span);
        quote_spanned!(span =>
            #( #cfg_attrs )*
            type #output_ident = ::ink::env::call::Execution<
                #arg_list,
                #output_type,
                #abi
            >;

            #( #attrs )*
            #[inline]
            fn #message_ident(
                & #mut_tok self
                #( , #input_bindings : #input_types )*
            ) -> Self::#output_ident {
                ::ink::env::call::Execution::new(
                    ::ink::env::call::ExecutionInput::new(
                        ::ink::env::call::Selector::new(#selector_bytes)
                    )
                    #(
                        .push_arg(#input_bindings)
                    )*
                )
            }
        )
    }
}
