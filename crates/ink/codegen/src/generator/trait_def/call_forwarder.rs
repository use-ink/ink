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
    /// Generates code for the global trait call forwarder for an ink! trait.
    ///
    /// # Note
    ///
    /// - The generated call forwarder type implements the ink! trait definition and
    ///   allows to build up contract calls that allow for customization by the user to
    ///   provide gas limit, endowment etc.
    /// - The call forwarder is associated to the call builder for the same ink! trait
    ///   definition and handles all ink! trait calls into another contract instance
    ///   on-chain. For constructing custom calls it forwards to the call builder.
    pub fn generate_call_forwarder(&self) -> TokenStream2 {
        CallForwarder::from(*self).generate_code()
    }

    /// The identifier of the ink! trait call forwarder.
    pub fn call_forwarder_ident(&self) -> syn::Ident {
        self.append_trait_suffix(CallForwarder::SUFFIX)
    }
}

/// Generates code for the global ink! trait call forwarder.
#[derive(From)]
struct CallForwarder<'a> {
    trait_def: TraitDefinition<'a>,
}

impl GenerateCode for CallForwarder<'_> {
    fn generate_code(&self) -> TokenStream2 {
        let struct_definition = self.generate_struct_definition();
        let storage_layout_impl = self.generate_storage_layout_impl();
        let auxiliary_trait_impls = self.generate_auxiliary_trait_impls();
        let to_from_addr_impls = self.generate_to_from_addr_impls();
        let call_builder_impl = self.generate_call_builder_trait_impl();
        let ink_trait_impl = self.generate_ink_trait_impl();
        quote! {
            #struct_definition
            #storage_layout_impl
            #auxiliary_trait_impls
            #to_from_addr_impls
            #call_builder_impl
            #ink_trait_impl
        }
    }
}

impl CallForwarder<'_> {
    /// The name suffix for ink! trait call forwarder.
    const SUFFIX: &'static str = "TraitCallForwarder";

    /// Returns the span of the ink! trait definition.
    fn span(&self) -> Span {
        self.trait_def.span()
    }

    /// Returns the identifier of the ink! trait call forwarder.
    fn ident(&self) -> syn::Ident {
        self.trait_def.call_forwarder_ident()
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
        let call_forwarder_ident = self.ident();
        quote_spanned!(span =>
            /// The global call forwarder for the ink! trait definition.
            ///
            /// All cross-contract calls to contracts implementing the associated ink! trait
            /// will be handled by this type.
            #[doc(hidden)]
            #[allow(non_camel_case_types)]
            #[::ink::scale_derive(Encode, Decode)]
            #[repr(transparent)]
            pub struct #call_forwarder_ident<E, Abi>
            where
                E: ::ink::env::Environment,
            {
                builder: <Self as ::ink::codegen::TraitCallBuilder>::Builder,
                _marker: ::core::marker::PhantomData<fn() -> Abi>,
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
        let call_forwarder_ident = self.ident();
        quote_spanned!(span=>
            #[cfg(feature = "std")]
            impl<E, Abi> ::ink::storage::traits::StorageLayout
                for #call_forwarder_ident<E, Abi>
            where
                E: ::ink::env::Environment,
                ::ink::Address: ::ink::storage::traits::StorageLayout,
            {
                fn layout(
                    __key: &::ink::primitives::Key,
                ) -> ::ink::metadata::layout::Layout {
                    <<Self as ::ink::codegen::TraitCallBuilder>::Builder
                        as ::ink::storage::traits::StorageLayout>::layout(__key)
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
        let call_forwarder_ident = self.ident();
        quote_spanned!(span=>
            /// We require this manual implementation since the derive produces incorrect trait bounds.
            impl<E, Abi> ::core::clone::Clone for #call_forwarder_ident<E, Abi>
            where
                E: ::ink::env::Environment,
                ::ink::Address: ::core::clone::Clone,
            {
                #[inline]
                fn clone(&self) -> Self {
                    Self {
                        builder: <<Self as ::ink::codegen::TraitCallBuilder>::Builder
                            as ::core::clone::Clone>::clone(&self.builder),
                        _marker: self._marker,
                    }
                }
            }

            /// We require this manual implementation since the derive produces incorrect trait bounds.
            impl<E, Abi> ::core::fmt::Debug for #call_forwarder_ident<E, Abi>
            where
                E: ::ink::env::Environment,
                ::ink::Address: ::core::fmt::Debug,
            {
                fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                    f.debug_struct(::core::stringify!(#call_forwarder_ident))
                        .field("addr", &self.builder.addr)
                        .finish()
                }
            }

            #[cfg(feature = "std")]
            /// We require this manual implementation since the derive produces incorrect trait bounds.
            impl<E, Abi> ::ink::scale_info::TypeInfo for #call_forwarder_ident<E, Abi>
            where
                E: ::ink::env::Environment,
                ::ink::Address: ::ink::scale_info::TypeInfo + 'static,
            {
                type Identity = <
                    <Self as ::ink::codegen::TraitCallBuilder>::Builder as ::ink::scale_info::TypeInfo
                >::Identity;

                fn type_info() -> ::ink::scale_info::Type {
                    <
                        <Self as ::ink::codegen::TraitCallBuilder>::Builder as ::ink::scale_info::TypeInfo
                    >::type_info()
                }
            }
        )
    }

    /// Generate trait impls for `FromAccountId` and `ToAccountId` for the account
    /// wrapper.
    ///
    /// # Note
    ///
    /// This allows user code to conveniently transform from and to `AccountId` when
    /// interacting with typed contracts.
    fn generate_to_from_addr_impls(&self) -> TokenStream2 {
        let span = self.span();
        let call_forwarder_ident = self.ident();
        quote_spanned!(span=>
            impl<E, Abi> ::ink::env::call::FromAddr
                for #call_forwarder_ident<E, Abi>
            where
                E: ::ink::env::Environment,
            {
                #[inline]
                fn from_addr(addr: ::ink::Address) -> Self {
                    Self {
                        builder: <<Self as ::ink::codegen::TraitCallBuilder>::Builder
                            as ::ink::env::call::FromAddr>::from_addr(addr),
                        _marker: ::core::default::Default::default(),
                    }
                }
            }

            impl<E, Abi> ::core::convert::From<::ink::Address> for #call_forwarder_ident<E, Abi>
            where
                E: ::ink::env::Environment,
            {
                fn from(addr: ::ink::Address) -> Self {
                    <Self as ::ink::env::call::FromAddr>::from_addr(addr)
                }
            }

            impl<E, Abi> ::ink::ToAddr for #call_forwarder_ident<E, Abi>
            where
                E: ::ink::env::Environment,
            {
                #[inline]
                fn to_addr(&self) -> ::ink::Address {
                    <<Self as ::ink::codegen::TraitCallBuilder>::Builder
                        as ::ink::ToAddr>::to_addr(&self.builder)
                }
            }

            impl<E, Abi> ::core::convert::AsRef<::ink::Address> for #call_forwarder_ident<E, Abi>
            where
                E: ::ink::env::Environment,
            {
                fn as_ref(&self) -> &::ink::Address {
                    <_ as ::core::convert::AsRef<::ink::Address>>::as_ref(&self.builder)
                }
            }

            impl<E, Abi> ::core::convert::AsMut<::ink::Address> for #call_forwarder_ident<E, Abi>
            where
                E: ::ink::env::Environment,
            {
                fn as_mut(&mut self) -> &mut ::ink::Address {
                    <_ as ::core::convert::AsMut<::ink::Address>>::as_mut(&mut self.builder)
                }
            }

            impl<E, Abi> ::ink::env::ContractEnv for #call_forwarder_ident<E, Abi>
            where
                E: ::ink::env::Environment,
            {
                type Env = E;
            }
        )
    }

    /// Generate the trait implementation for `CallBuilder` for the ink! trait call
    /// forwarder.
    ///
    /// # Note
    ///
    /// Through the implementation of this trait it is possible to refer to the
    /// ink! trait call builder that is associated to this ink! trait call forwarder.
    fn generate_call_builder_trait_impl(&self) -> TokenStream2 {
        let span = self.trait_def.span();
        let call_forwarder_ident = self.ident();
        let call_builder_ident = self.trait_def.call_builder_ident();
        quote_spanned!(span=>
            /// This trait allows to bridge from call forwarder to call builder.
            ///
            /// Also this explains why we designed the generated code so that we have
            /// both types and why the forwarder is a thin-wrapper around the builder
            /// as this allows to perform this operation safely.
            impl<E, Abi> ::ink::codegen::TraitCallBuilder for #call_forwarder_ident<E, Abi>
            where
                E: ::ink::env::Environment,
            {
                type Builder = #call_builder_ident<E, Abi>;

                #[inline]
                fn call(&self) -> &<Self as ::ink::codegen::TraitCallBuilder>::Builder {
                    &self.builder
                }

                #[inline]
                fn call_mut(&mut self) -> &mut <Self as ::ink::codegen::TraitCallBuilder>::Builder {
                    &mut self.builder
                }
            }
        )
    }

    /// Generates the implementation of the associated ink! trait definition.
    ///
    /// # Note
    ///
    /// The implementation mainly forwards to the associated ink! call builder
    /// of the same ink! trait definition.
    fn generate_ink_trait_impl(&self) -> TokenStream2 {
        let span = self.trait_def.span();
        let trait_ident = self.trait_def.trait_def.item().ident();
        let trait_info_ident = self.trait_def.trait_info_ident();
        let forwarder_ident = self.ident();
        let message_impls = self.generate_ink_trait_impl_messages();
        generate_abi_impls!(@tokens |abi| quote_spanned!(span=>
            impl<E> #trait_ident for #forwarder_ident<E, #abi>
            where
                E: ::ink::env::Environment,
            {
                #[allow(non_camel_case_types)]
                type __ink_TraitInfo = #trait_info_ident<E>;

                #message_impls
            }
        ))
    }

    /// Generate the code for all ink! trait messages implemented by the trait call
    /// forwarder.
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

    /// Generate the code for a single ink! trait message implemented by the trait call
    /// forwarder.
    fn generate_ink_trait_impl_for_message(
        &self,
        message: &ir::InkTraitMessage,
    ) -> TokenStream2 {
        let span = message.span();
        let trait_ident = self.trait_def.trait_def.item().ident();
        let forwarder_ident = self.ident();
        let message_ident = message.ident();
        let attrs = self
            .trait_def
            .trait_def
            .config()
            .whitelisted_attributes()
            .filter_attr(message.attrs());
        let output_ident = generator::output_ident(message_ident);
        let output_type = message
            .output()
            .cloned()
            .unwrap_or_else(|| syn::parse_quote!(()));
        let input_bindings = message.inputs().map(|input| &input.pat).collect::<Vec<_>>();
        let input_types = message.inputs().map(|input| &input.ty).collect::<Vec<_>>();
        let call_op = match message.receiver() {
            ir::Receiver::Ref => quote! { call },
            ir::Receiver::RefMut => quote! { call_mut },
        };
        let mut_tok = message.mutates().then(|| quote! { mut });
        let panic_str = format!(
            "encountered error while calling <{forwarder_ident} as {trait_ident}>::{message_ident}",
        );
        let cfg_attrs = message.get_cfg_attrs(span);
        quote_spanned!(span =>
            #( #cfg_attrs )*
            type #output_ident = #output_type;

            #( #attrs )*
            #[inline]
            fn #message_ident(
                & #mut_tok self
                #( , #input_bindings : #input_types )*
            ) -> Self::#output_ident {
                <<Self as ::ink::codegen::TraitCallBuilder>::Builder as #trait_ident>::#message_ident(
                    <Self as ::ink::codegen::TraitCallBuilder>::#call_op(self)
                    #(
                        , #input_bindings
                    )*
                )
                    .try_invoke()
                    .unwrap_or_else(|env_err| ::core::panic!("{}: {:?}", #panic_str, env_err))
                    .unwrap_or_else(|lang_err| ::core::panic!("{}: {:?}", #panic_str, lang_err))
            }
        )
    }
}
