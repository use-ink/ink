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
    pub fn generate_call_builder(&self) -> TokenStream2 {
        CallBuilder::from(*self).generate_code()
    }

    /// The identifier of the ink! trait call builder.
    pub fn call_builder_ident(&self) -> syn::Ident {
        self.append_trait_suffix(CallBuilder::SUFFIX)
    }
}

/// Generates code for the global ink! trait call builder.
#[derive(From)]
struct CallBuilder<'a> {
    trait_def: TraitDefinition<'a>,
}

impl GenerateCode for CallBuilder<'_> {
    fn generate_code(&self) -> TokenStream2 {
        let struct_definition = self.generate_struct_definition();
        let storage_layout_impl = self.generate_storage_layout_impl();
        let auxiliary_trait_impls = self.generate_auxiliary_trait_impls();
        let to_from_addr_impls = self.generate_to_from_addr_impls();
        let message_builder_trait_impl = self.generate_message_builder_trait_impl();
        let ink_trait_impl = self.generate_ink_trait_impl();
        quote! {
            #struct_definition
            #storage_layout_impl
            #auxiliary_trait_impls
            #to_from_addr_impls
            #message_builder_trait_impl
            #ink_trait_impl
        }
    }
}

impl CallBuilder<'_> {
    /// The name suffix for ink! trait call builder.
    const SUFFIX: &'static str = "TraitCallBuilder";

    /// Returns the span of the ink! trait definition.
    fn span(&self) -> Span {
        self.trait_def.span()
    }

    /// Returns the identifier of the ink! trait call builder.
    fn ident(&self) -> syn::Ident {
        self.trait_def.call_builder_ident()
    }

    /// Generates the struct type definition for the account wrapper type.
    ///
    /// This type is going to implement the trait so that invoking its trait
    /// methods will perform contract calls via contract's pallet contract
    /// execution abstraction.
    ///
    /// # Note
    ///
    /// Unlike the layout specific traits it is possible to derive the SCALE
    /// `Encode` and `Decode` traits since they generate trait bounds per field
    /// instead of per generic parameter which is exactly what we need here.
    /// However, it should be noted that this is not Rust default behavior.
    fn generate_struct_definition(&self) -> TokenStream2 {
        let span = self.span();
        let call_builder_ident = self.ident();
        quote_spanned!(span =>
            /// The global call builder type for all trait implementers.
            ///
            /// All calls to types (contracts) implementing the trait will be built by this type.
            #[doc(hidden)]
            #[allow(non_camel_case_types)]
            #[::ink::scale_derive(Encode, Decode)]
            #[repr(transparent)]
            pub struct #call_builder_ident<E, Abi>
            where
                E: ::ink::env::Environment,
            {
                addr: ::ink::Address,
                _marker: ::core::marker::PhantomData<fn() -> (E, Abi)>,
            }
        )
    }

    /// Generates the `StorageLayout` trait implementation for the account wrapper.
    ///
    /// # Note
    ///
    /// Due to the generic parameter `E` and Rust's default rules for derive generated
    /// trait bounds it is not recommended to derive the `StorageLayout` trait
    /// implementation.
    fn generate_storage_layout_impl(&self) -> TokenStream2 {
        let span = self.span();
        let call_builder_ident = self.ident();
        quote_spanned!(span=>
            #[cfg(feature = "std")]
            impl<E, Abi> ::ink::storage::traits::StorageLayout
                for #call_builder_ident<E, Abi>
            where
                E: ::ink::env::Environment,
                ::ink::Address: ::ink::storage::traits::StorageLayout,
            {
                fn layout(
                    __key: &::ink::primitives::Key,
                ) -> ::ink::metadata::layout::Layout {
                    ::ink::metadata::layout::Layout::Struct(
                        ::ink::metadata::layout::StructLayout::new(
                            ::core::stringify!(#call_builder_ident),
                            [
                                ::ink::metadata::layout::FieldLayout::new(
                                    "addr",
                                    <::ink::Address
                                        as ::ink::storage::traits::StorageLayout>::layout(__key)
                                )
                            ]
                        )
                    )
                }
            }
        )
    }

    /// Generates trait implementations for auxiliary traits for the account wrapper.
    ///
    /// # Note
    ///
    /// Auxiliary traits currently include:
    ///
    /// - `Clone`: To allow cloning contract references in the long run.
    /// - `Debug`: To better debug internal contract state.
    fn generate_auxiliary_trait_impls(&self) -> TokenStream2 {
        let span = self.span();
        let call_builder_ident = self.ident();
        quote_spanned!(span=>
            /// We require this manual implementation since the derive produces incorrect trait bounds.
            impl<E, Abi> ::core::clone::Clone for #call_builder_ident<E, Abi>
            where
                E: ::ink::env::Environment,
                ::ink::Address: ::core::clone::Clone,
            {
                #[inline]
                fn clone(&self) -> Self {
                    Self {
                        addr: ::core::clone::Clone::clone(&self.addr),
                        _marker: self._marker,
                    }
                }
            }

            /// We require this manual implementation since the derive produces incorrect trait bounds.
            impl<E, Abi> ::core::fmt::Debug for #call_builder_ident<E, Abi>
            where
                E: ::ink::env::Environment,
                ::ink::Address: ::core::fmt::Debug,
            {
                fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                    f.debug_struct(::core::stringify!(#call_builder_ident))
                        .field("addr", &self.addr)
                        .finish()
                }
            }

            #[cfg(feature = "std")]
            // todo
            /// We require this manual implementation since the derive produces incorrect trait bounds.
            impl<E, Abi> ::ink::scale_info::TypeInfo for #call_builder_ident<E, Abi>
            where
                E: ::ink::env::Environment,
                ::ink::Address: ::ink::scale_info::TypeInfo + 'static,
            {
                type Identity = ::ink::Address;

                fn type_info() -> ::ink::scale_info::Type {
                    <::ink::Address as ::ink::scale_info::TypeInfo>::type_info()
                }
            }
        )
    }

    /// Generate trait implementations for `FromAccountId` and `ToAccountId` for the
    /// account wrapper.
    ///
    /// # Note
    ///
    /// This allows user code to conveniently transform from and to `AccountId` when
    /// interacting with typed contracts.
    fn generate_to_from_addr_impls(&self) -> TokenStream2 {
        let span = self.span();
        let call_builder_ident = self.ident();
        quote_spanned!(span=>
            impl<E, Abi> ::ink::env::call::FromAddr
                for #call_builder_ident<E, Abi>
            where
                E: ::ink::env::Environment,
            {
                #[inline]
                fn from_addr(addr: ::ink::Address) -> Self {
                    Self {
                        addr,
                        _marker: ::core::default::Default::default(),
                    }
                }
            }

            impl<E, Abi> ::core::convert::From<::ink::Address> for #call_builder_ident<E, Abi>
            where
                E: ::ink::env::Environment,
                ::ink::Address: ::ink::env::AccountIdGuard,
            {
                fn from(value: ::ink::Address) -> Self {
                    <Self as ::ink::env::call::FromAddr>::from_addr(value)
                }
            }

            impl<E, Abi> ::ink::ToAddr for #call_builder_ident<E, Abi>
            where
                E: ::ink::env::Environment,
            {
                #[inline]
                fn to_addr(&self) -> ::ink::Address {
                    <::ink::Address as ::core::clone::Clone>::clone(&self.addr)
                }
            }

            impl<E, Abi> ::core::convert::AsRef<::ink::Address> for #call_builder_ident<E, Abi>
            where
                E: ::ink::env::Environment,
            {
                fn as_ref(&self) -> &::ink::Address {
                    &self.addr
                }
            }

            impl<E, Abi> ::core::convert::AsMut<::ink::Address> for #call_builder_ident<E, Abi>
            where
                E: ::ink::env::Environment,
            {
                fn as_mut(&mut self) -> &mut ::ink::Address {
                    &mut self.addr
                }
            }

            impl<E, Abi> ::ink::env::ContractEnv for #call_builder_ident<E, Abi>
            where
                E: ::ink::env::Environment,
            {
                type Env = E;
            }
        )
    }

    /// Generate the trait implementation for `MessageBuilder` for the ink! trait call
    /// builder.
    ///
    /// # Note
    ///
    /// Through the implementation of this trait it is possible to refer to the
    /// ink! trait message builder that is associated to this ink! trait call builder.
    fn generate_message_builder_trait_impl(&self) -> TokenStream2 {
        let span = self.trait_def.span();
        let call_builder_ident = self.ident();
        let message_builder_ident = self.trait_def.message_builder_ident();
        quote_spanned!(span=>
            /// This trait allows to bridge from the call builder to message builder.
            impl<E, TypeAbi> ::ink::codegen::TraitMessageBuilder for #call_builder_ident<E, TypeAbi>
            where
                E: ::ink::env::Environment
            {
                type MessageBuilder<TraitAbi> = #message_builder_ident<E, TypeAbi>;
            }
        )
    }

    /// Generates the implementation of the associated ink! trait definition.
    ///
    /// # Note
    ///
    /// The implemented messages call the SEAL host runtime in order to dispatch
    /// the respective ink! trait message calls of the called smart contract
    /// instance.
    /// The way these messages are built-up allows the caller to customize message
    /// parameters such as gas limit and transferred value.
    fn generate_ink_trait_impl(&self) -> TokenStream2 {
        let span = self.trait_def.span();
        let trait_ident = self.trait_def.trait_def.item().ident();
        let trait_info_ident = self.trait_def.trait_info_ident();
        let builder_ident = self.ident();
        generate_abi_impls!(@tokens |abi: TokenStream2| {
            let message_impls = self.generate_ink_trait_impl_messages(abi.clone());
            quote_spanned!(span=>
                impl<E> #trait_ident for #builder_ident<E, #abi>
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
    fn generate_ink_trait_impl_messages(&self, abi: TokenStream2) -> TokenStream2 {
        let messages = self.trait_def.trait_def.item().iter_items().filter_map(
            |(item, _selector)| {
                item.filter_map_message().map(|message| {
                    self.generate_ink_trait_impl_for_message(&message, abi.clone())
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
        abi: TokenStream2,
    ) -> TokenStream2 {
        let span = message.span();
        let trait_ident = self.trait_def.trait_def.item().ident();
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
            #[allow(clippy::type_complexity)]
            #( #cfg_attrs )*
            type #output_ident = ::ink::env::call::CallBuilder<
                Self::Env,
                ::ink::env::call::utils::Set< ::ink::env::call::Call >,
                ::ink::env::call::utils::Set< ::ink::env::call::ExecutionInput<#arg_list, #abi> >,
                ::ink::env::call::utils::Set< ::ink::env::call::utils::ReturnType<#output_type> >,
            >;

            #( #attrs )*
            #[inline]
            fn #message_ident(
                & #mut_tok self
                #( , #input_bindings : #input_types )*
            ) -> Self::#output_ident {
                <::ink::env::call::CallBuilder<
                    Self::Env,
                    ::ink::env::call::utils::Unset< ::ink::env::call::Call >,
                    ::ink::env::call::utils::Set< ::ink::env::call::ExecutionInput<#arg_list, #abi> >,
                    ::ink::env::call::utils::Set< ::ink::env::call::utils::ReturnType<#output_type> >,
                > as ::core::convert::From::<_>>::from(
                    <<Self as ::ink::codegen::TraitMessageBuilder>::MessageBuilder<#abi> as #trait_ident>
                        ::#message_ident(
                            & #mut_tok <<Self
                                as ::ink::codegen::TraitMessageBuilder>::MessageBuilder<#abi>
                                as ::core::default::Default>::default()
                            #(
                                , #input_bindings
                            )*
                        )
                )
                    .call(::ink::ToAddr::to_addr(self))
            }
        )
    }
}
