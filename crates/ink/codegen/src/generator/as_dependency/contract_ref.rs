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
use ir::{
    Callable,
    IsDocAttribute as _,
};
use itertools::Itertools;
use proc_macro2::TokenStream as TokenStream2;
use quote::{
    format_ident,
    quote,
    quote_spanned,
};
use syn::spanned::Spanned as _;

use crate::{
    GenerateCode,
    generator,
};

/// Generates code for the contract reference of the ink! smart contract.
///
/// The call builder is the entity that builds up calls for calling of other
/// smart contract on-chain in a type safe way.
/// It implements all ink! traits that the associated ink! smart contract implements
/// so that their underlying implementation directly calls the respective ink!
/// trait implementation on-chain.
///
/// The ink! call builder of a smart contract is directly used by the storage
/// type of the smart contract itself as well by other entities that use the
/// smart contract via long-hand calling notation to incrementally build up calls.
#[derive(From)]
pub struct ContractRef<'a> {
    contract: &'a ir::Contract,
}

impl GenerateCode for ContractRef<'_> {
    fn generate_code(&self) -> TokenStream2 {
        let contract_ref = self.generate_struct();
        let contract_ref_trait_impls = self.generate_contract_trait_impls();
        let contract_ref_inherent_impls = self.generate_contract_inherent_impls();
        let call_builder_trait_impl = self.generate_call_builder_trait_impl();
        let auxiliary_trait_impls = self.generate_auxiliary_trait_impls();
        quote! {
            #contract_ref
            #contract_ref_trait_impls
            #contract_ref_inherent_impls
            #call_builder_trait_impl
            #auxiliary_trait_impls
        }
    }
}

impl ContractRef<'_> {
    /// Generates the identifier of the contract reference struct.
    fn generate_contract_ref_base_ident(&self) -> syn::Ident {
        format_ident!("{}Ref", self.contract.module().storage().ident())
    }

    /// Generates the identifier of the contract reference struct.
    fn generate_contract_ref_ident(&self) -> syn::Ident {
        format_ident!("{}For", self.generate_contract_ref_base_ident())
    }

    /// Generates the code for the struct representing the contract reference.
    ///
    /// The generated struct is the type onto which everything is implemented.
    /// It is also the type that is going to be used by other smart contract
    /// dynamically depending on the smart contract. It mirrors the smart contract
    /// API but is just a typed thin-wrapper around an `AccountId`.
    fn generate_struct(&self) -> TokenStream2 {
        let span = self.contract.module().storage().span();
        let doc_attrs = self
            .contract
            .module()
            .storage()
            .attrs()
            .iter()
            .filter(|&x| syn::Attribute::is_doc_attribute(x))
            .cloned();
        let storage_ident = self.contract.module().storage().ident();
        let ref_ident = self.generate_contract_ref_ident();
        let abi = default_abi!();
        let ref_ident_default_abi = self.generate_contract_ref_base_ident();
        let ref_ident_abi_aliases = generate_abi_impls!(@type |abi| {
            let (abi_ty, suffix) = match abi {
                Abi::Ink => (quote!(::ink::abi::Ink), "Ink"),
                Abi::Sol => (quote!(::ink::abi::Sol), "Sol"),
            };
            let ref_ident_abi_alias = format_ident!("{ref_ident_default_abi}{suffix}");
            quote! {
                #[allow(dead_code)]
                pub type #ref_ident_abi_alias = #ref_ident::<#abi_ty>;
            }
        });
        let sol_codec = if cfg!(any(ink_abi = "sol", ink_abi = "all")) {
            // These manual implementations are a bit more efficient than the derived
            // equivalents.
            quote_spanned!(span=>
                impl<Abi> ::ink::SolDecode for #ref_ident<Abi> {
                    type SolType = ::ink::Address;

                    fn from_sol_type(value: Self::SolType) -> ::core::result::Result<Self, ::ink::sol::Error> {
                        Ok(Self {
                            inner: <<#storage_ident
                                as ::ink::codegen::ContractCallBuilder>::Type<Abi>
                                as ::ink::env::call::FromAddr>::from_addr(value),
                            _marker: ::core::marker::PhantomData,
                        })
                    }
                }

                impl<'a, Abi> ::ink::SolEncode<'a> for #ref_ident<Abi> {
                    type SolType = &'a ::ink::Address;

                    fn to_sol_type(&'a self) -> Self::SolType {
                        self.as_ref()
                    }
                }
            )
        } else {
            quote!()
        };
        quote_spanned!(span=>
            #[derive(
                ::core::fmt::Debug,
                ::core::hash::Hash,
                ::core::cmp::PartialEq,
                ::core::cmp::Eq,
                ::core::clone::Clone,
            )]
            #[::ink::scale_derive(Encode, Decode)]
            #( #doc_attrs )*
            pub struct #ref_ident<Abi = #abi> {
                inner: <#storage_ident as ::ink::codegen::ContractCallBuilder>::Type<Abi>,
                _marker: core::marker::PhantomData<Abi>,
            }

            // Default type alias (i.e. `ContractRef` for a contract named `Contract`).
            #[allow(dead_code)]
            pub type #ref_ident_default_abi = #ref_ident::<#abi>;
            // ABI specific type aliases (i.e. `ContractRefInk` and `ContractRefSol`) as appropriate.
            #ref_ident_abi_aliases

            const _: () = {
                impl ::ink::env::ContractReference for #storage_ident {
                    type Type = #ref_ident;
                }

                impl<Abi> ::ink::env::ContractReverseReference for #ref_ident<Abi> {
                    type Type = #storage_ident;
                }

                impl<Abi> ::ink::env::call::ConstructorReturnType<#ref_ident, Abi> for #storage_ident
                where
                    (): ink::env::call::utils::DecodeConstructorError<Abi>
                {
                    type Output = #ref_ident;
                    type Error = ();

                    fn ok(value: #ref_ident) -> Self::Output {
                        value
                    }
                }

                impl<E, Abi> ::ink::env::call::ConstructorReturnType<#ref_ident, Abi>
                    for ::core::result::Result<#storage_ident, E>
                where
                    E: ink::env::call::utils::DecodeConstructorError<Abi>
                {
                    const IS_RESULT: bool = true;

                    type Output = ::core::result::Result<#ref_ident, E>;
                    type Error = E;

                    fn ok(value: #ref_ident) -> Self::Output {
                        ::core::result::Result::Ok(value)
                    }

                    fn err(err: Self::Error) -> ::core::option::Option<Self::Output> {
                        ::core::option::Option::Some(::core::result::Result::Err(err))
                    }
                }

                impl<Abi> ::ink::env::ContractEnv for #ref_ident<Abi> {
                    type Env = <#storage_ident as ::ink::env::ContractEnv>::Env;
                }

                #[cfg(feature = "std")]
                // We require this manual implementation since the derive produces incorrect trait bounds.
                impl<Abi> ::ink::storage::traits::StorageLayout for #ref_ident<Abi> {
                    fn layout(
                        __key: &::ink::primitives::Key,
                    ) -> ::ink::metadata::layout::Layout {
                        ::ink::metadata::layout::Layout::Struct(
                            ::ink::metadata::layout::StructLayout::new(
                                ::core::stringify!(#ref_ident),
                                [
                                    ::ink::metadata::layout::FieldLayout::new(
                                        "inner",
                                        <<#storage_ident as ::ink::codegen::ContractCallBuilder>::Type<Abi>
                                            as ::ink::storage::traits::StorageLayout>::layout(__key)
                                    )
                                ]
                            )
                        )
                    }
                }

                #[cfg(feature = "std")]
                // We require this manual implementation since the derive produces incorrect trait bounds.
                impl<Abi> ::ink::scale_info::TypeInfo for #ref_ident<Abi>
                where
                    ::ink::Address: ::ink::scale_info::TypeInfo + 'static,
                {
                    type Identity = <
                        <#storage_ident as ::ink::codegen::ContractCallBuilder>::Type<Abi> as ::ink::scale_info::TypeInfo
                    >::Identity;

                    fn type_info() -> ::ink::scale_info::Type {
                        <
                            <#storage_ident as ::ink::codegen::ContractCallBuilder>::Type<Abi> as ::ink::scale_info::TypeInfo
                        >::type_info()
                    }
                }

                #sol_codec
            };
        )
    }

    /// Generates some ink! specific auxiliary trait implementations for the
    /// smart contract reference type.
    ///
    /// These are required to properly interoperate with the contract reference.
    fn generate_auxiliary_trait_impls(&self) -> TokenStream2 {
        let span = self.contract.module().storage().span();
        let storage_ident = self.contract.module().storage().ident();
        let ref_ident = self.generate_contract_ref_ident();
        quote_spanned!(span=>
            impl<Abi> ::ink::env::call::FromAddr for #ref_ident<Abi> {
                #[inline]
                fn from_addr(addr: ::ink::Address) -> Self {
                    Self {
                        inner: <<#storage_ident
                            as ::ink::codegen::ContractCallBuilder>::Type<Abi>
                            as ::ink::env::call::FromAddr>::from_addr(addr),
                        _marker: ::core::default::Default::default(),
                    }
                }
            }

            impl<Abi> ::ink::ToAddr for #ref_ident<Abi> {
                #[inline]
                fn to_addr(&self) -> ::ink::Address {
                    <<#storage_ident as ::ink::codegen::ContractCallBuilder>::Type<Abi>
                        as ::ink::ToAddr>::to_addr(&self.inner)
                }
            }

            impl<Abi> ::core::convert::AsRef<::ink::Address> for #ref_ident<Abi> {
                fn as_ref(&self) -> &::ink::Address {
                    <_ as ::core::convert::AsRef<::ink::Address>>::as_ref(&self.inner)
                }
            }

            impl<Abi> ::core::convert::AsMut<::ink::Address> for #ref_ident<Abi> {
                fn as_mut(&mut self) -> &mut ::ink::Address {
                    <_ as ::core::convert::AsMut<::ink::Address>>::as_mut(&mut self.inner)
                }
            }
        )
    }

    /// Generates the `CallBuilder` trait implementation for the contract reference.
    ///
    /// This creates the bridge between the ink! smart contract type and the
    /// associated call builder.
    fn generate_call_builder_trait_impl(&self) -> TokenStream2 {
        let span = self.contract.module().storage().span();
        let ref_ident = self.generate_contract_ref_ident();
        let storage_ident = self.contract.module().storage().ident();
        quote_spanned!(span=>
            const _: () = {
                impl<Abi> ::ink::codegen::TraitCallBuilder for #ref_ident<Abi> {
                    type Builder = <#storage_ident as ::ink::codegen::ContractCallBuilder>::Type<Abi>;

                    #[inline]
                    fn call(&self) -> &Self::Builder {
                        &self.inner
                    }

                    #[inline]
                    fn call_mut(&mut self) -> &mut Self::Builder {
                        &mut self.inner
                    }
                }
            };
        )
    }

    /// Generates the code for all ink! trait implementations of the contract itself.
    ///
    /// # Note
    ///
    /// The generated implementations must live outside of an artificial `const` block
    /// in order to properly show their documentation using `rustdoc`.
    fn generate_contract_trait_impls(&self) -> TokenStream2 {
        self.contract
            .module()
            .impls()
            .filter_map(|impl_block| {
                // We are only interested in ink! trait implementation block.
                impl_block.trait_path().map(|trait_path| {
                    self.generate_contract_trait_impl(trait_path, impl_block)
                })
            })
            .collect()
    }

    /// Generates the code for a single ink! trait implementation of the contract itself.
    ///
    /// The generated implementation mainly forwards the calls to the previously generated
    /// associated call builder that implements each respective ink! trait.
    fn generate_contract_trait_impl(
        &self,
        trait_path: &syn::Path,
        impl_block: &ir::ItemImpl,
    ) -> TokenStream2 {
        let span = impl_block.span();
        let attrs = impl_block.attrs();
        let forwarder_ident = self.generate_contract_ref_ident();
        generate_abi_impls!(@tokens |abi: TokenStream2| {
            let messages = self.generate_contract_trait_impl_messages(trait_path, impl_block, abi.clone());
            quote_spanned!(span=>
                #( #attrs )*
                impl #trait_path for #forwarder_ident<#abi> {
                    type __ink_TraitInfo = <::ink::reflect::TraitDefinitionRegistry<Environment>
                        as #trait_path>::__ink_TraitInfo;

                    #messages
                }
            )
        })
    }

    /// Generates the code for all messages of a single ink! trait implementation of
    /// the ink! smart contract.
    fn generate_contract_trait_impl_messages(
        &self,
        trait_path: &syn::Path,
        impl_block: &ir::ItemImpl,
        abi: TokenStream2,
    ) -> TokenStream2 {
        impl_block
            .iter_messages()
            .map(|message| {
                self.generate_contract_trait_impl_for_message(
                    trait_path,
                    message,
                    abi.clone(),
                )
            })
            .collect()
    }

    /// Generates the code for a single message of a single ink! trait implementation
    /// that is implemented by the ink! smart contract.
    fn generate_contract_trait_impl_for_message(
        &self,
        trait_path: &syn::Path,
        message: ir::CallableWithSelector<ir::Message>,
        abi: TokenStream2,
    ) -> TokenStream2 {
        use ir::Callable as _;
        let span = message.span();
        let trait_info_id = generator::generate_reference_to_trait_info(span, trait_path);
        let message_ident = message.ident();
        let output_ident = generator::output_ident(message_ident);
        let call_operator = match message.receiver() {
            ir::Receiver::Ref => quote! { call },
            ir::Receiver::RefMut => quote! { call_mut },
        };
        let forward_operator = match message.receiver() {
            ir::Receiver::Ref => quote! { forward },
            ir::Receiver::RefMut => quote! { forward_mut },
        };
        let mut_token = message.receiver().is_ref_mut().then(|| quote! { mut });
        let input_idents = generator::input_message_idents(message.inputs());
        let input_types = message.inputs().map(|input| &input.ty).collect::<Vec<_>>();
        let cfg_attrs = message.get_cfg_attrs(span);
        quote_spanned!(span=>
            #( #cfg_attrs )*
            type #output_ident =
                <<Self::__ink_TraitInfo as ::ink::codegen::TraitCallForwarder>::Forwarder<#abi> as #trait_path>::#output_ident;

            #[inline]
            #( #cfg_attrs )*
            fn #message_ident(
                & #mut_token self
                #( , #input_idents : #input_types )*
            ) -> Self::#output_ident {
                <_ as #trait_path>::#message_ident(
                    <_ as ::ink::codegen::TraitCallForwarderFor<{#trait_info_id}>>::#forward_operator(
                        <Self as ::ink::codegen::TraitCallBuilder>::#call_operator(self),
                    )
                    #( , #input_idents )*
                )
            }
        )
    }

    /// Generates the code for all ink! inherent implementations of the contract itself.
    ///
    /// # Note
    ///
    /// The generated implementations must live outside of an artificial `const` block
    /// in order to properly show their documentation using `rustdoc`.
    fn generate_contract_inherent_impls(&self) -> TokenStream2 {
        generate_abi_impls!(@type |abi| {
            let impls = self.contract.module().impls()
                .filter(|impl_block| {
                    // We are only interested in ink! trait implementation block.
                    impl_block.trait_path().is_none()
                })
                .map(|impl_block| self.generate_contract_inherent_impl(impl_block, abi));
            let impl_sol_constructor = match abi {
                Abi::Ink => quote!(),
                Abi::Sol => {
                    // Only one constructor is used for Solidity ABI encoding.
                    let constructor = self.contract.module().impls()
                        .flat_map(|item_impl| item_impl.iter_constructors())
                        .find_or_first(|constructor| {
                            constructor.is_default()
                        })
                        .expect("Expected at least one constructor");
                    let ctor = self.generate_contract_inherent_impl_for_constructor(constructor, abi);
                    let span = ctor.span();
                    let forwarder_ident = self.generate_contract_ref_ident();
                    quote_spanned!(span=>
                        impl #forwarder_ident<::ink::abi::Sol> {
                            #ctor
                        }
                    )
                }
            };
            let span = self.contract.module().span();
            quote_spanned!(span=>
                #impl_sol_constructor
                #( #impls )*
            )
        })
    }

    /// Generates the code for a single ink! inherent implementation of the contract
    /// itself.
    ///
    /// # Note
    ///
    /// This produces the short-hand calling notation for the inherent contract
    /// implementation. The generated code simply forwards its calling logic to the
    /// associated call builder.
    fn generate_contract_inherent_impl(
        &self,
        impl_block: &ir::ItemImpl,
        abi: Abi,
    ) -> TokenStream2 {
        let span = impl_block.span();
        let attrs = impl_block.attrs();
        let forwarder_ident = self.generate_contract_ref_ident();
        let messages = impl_block
            .iter_messages()
            .map(|message| self.generate_contract_inherent_impl_for_message(message));
        let messages = quote! {
            #( #messages )*
        };
        let (abi_ty, constructors) = match abi {
            Abi::Ink => {
                let constructors = impl_block.iter_constructors().map(|constructor| {
                    self.generate_contract_inherent_impl_for_constructor(constructor, abi)
                });
                (
                    quote!(::ink::abi::Ink),
                    quote! {
                        #( #constructors )*
                    },
                )
            }
            Abi::Sol => {
                (
                    quote!(::ink::abi::Sol),
                    // The constructor implementation for Solidity ABI encoding is
                    // handled in `generate_contract_inherent_impls`.
                    quote!(),
                )
            }
        };
        quote_spanned!(span=>
            #( #attrs )*
            impl #forwarder_ident<#abi_ty> {
                #constructors
                #messages
            }
        )
    }

    /// Generates the code for a single ink! inherent message of the contract itself.
    ///
    /// # Note
    ///
    /// This produces the short-hand calling notation for the inherent contract message.
    /// The generated code simply forwards its calling logic to the associated call
    /// builder.
    fn generate_contract_inherent_impl_for_message(
        &self,
        message: ir::CallableWithSelector<ir::Message>,
    ) -> TokenStream2 {
        use ir::Callable as _;
        let span = message.span();
        let attrs = self
            .contract
            .config()
            .whitelisted_attributes()
            .filter_attr(message.attrs().to_vec());
        let storage_ident = self.contract.module().storage().ident();
        let message_ident = message.ident();
        let try_message_ident = message.try_ident();
        let call_operator = match message.receiver() {
            ir::Receiver::Ref => quote! { call },
            ir::Receiver::RefMut => quote! { call_mut },
        };
        let mut_token = message.receiver().is_ref_mut().then(|| quote! { mut });
        let input_idents = generator::input_message_idents(message.inputs());
        let input_types = message.inputs().map(|input| &input.ty).collect::<Vec<_>>();
        let output_type = message.output().map(|ty| quote! { -> #ty });
        let wrapped_output_type = message.wrapped_output();
        quote_spanned!(span=>
            #( #attrs )*
            #[inline]
            pub fn #message_ident(
                & #mut_token self
                #( , #input_idents : #input_types )*
            ) #output_type {
                self.#try_message_ident( #( #input_idents, )* )
                    .unwrap_or_else(|error| ::core::panic!(
                        "encountered error while calling {}::{}: {:?}",
                        ::core::stringify!(#storage_ident),
                        ::core::stringify!(#message_ident),
                        error,
                    ))
            }

            #( #attrs )*
            #[inline]
            pub fn #try_message_ident(
                & #mut_token self
                #( , #input_idents : #input_types )*
            ) -> #wrapped_output_type {
                <Self as ::ink::codegen::TraitCallBuilder>::#call_operator(self)
                    .#message_ident( #( #input_idents ),* )
                    .try_invoke()
                    .unwrap_or_else(|error| ::core::panic!(
                        "encountered error while calling {}::{}: {:?}",
                        ::core::stringify!(#storage_ident),
                        ::core::stringify!(#message_ident),
                        error,
                    ))
            }
        )
    }

    /// Generates the code for a single ink! inherent constructor of the contract itself.
    ///
    /// # Note
    ///
    /// Unlike with ink! messages this does not forward to the call builder since
    /// constructor calls in ink! do not have a short-hand notation and therefore this
    /// implements the long-hand calling notation code directly.
    fn generate_contract_inherent_impl_for_constructor(
        &self,
        constructor: ir::CallableWithSelector<ir::Constructor>,
        abi: Abi,
    ) -> TokenStream2 {
        let span = constructor.span();
        let attrs = self
            .contract
            .config()
            .whitelisted_attributes()
            .filter_attr(constructor.attrs().to_vec());
        let constructor_ident = constructor.ident();
        let input_bindings = generator::input_bindings(constructor.inputs());
        let input_types = generator::input_types(constructor.inputs());
        let ret_type = constructor
            .output()
            .map(quote::ToTokens::to_token_stream)
            .unwrap_or_else(|| quote::quote! { Self });

        let (abi_ty, exec_input_init, build_create_fn) = match abi {
            Abi::Ink => {
                let selector_bytes = constructor.composed_selector().hex_lits();
                (
                    quote!(::ink::abi::Ink),
                    quote! {
                        ::ink::env::call::ExecutionInput::new(
                            ::ink::env::call::Selector::new([ #( #selector_bytes ),* ])
                        )
                    },
                    quote!(build_create_ink),
                )
            }
            Abi::Sol => {
                (
                    quote!(::ink::abi::Sol),
                    quote! {
                        ::ink::env::call::ExecutionInput::no_selector()
                    },
                    quote!(build_create_sol),
                )
            }
        };
        let arg_list = generator::generate_argument_list(
            input_types.iter().cloned(),
            abi_ty.clone(),
        );

        quote_spanned!(span =>
            #( #attrs )*
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn #constructor_ident(
                #( #input_bindings : #input_types ),*
            ) -> ::ink::env::call::CreateBuilder<
                Environment,
                Self,
                ::ink::env::call::utils::Set<::ink::env::call::LimitParamsV2 >,
                ::ink::env::call::utils::Set<::ink::env::call::ExecutionInput<#arg_list, #abi_ty>>,
                ::ink::env::call::utils::Set<::ink::env::call::utils::ReturnType<#ret_type>>,
                #abi_ty
            > {
                ::ink::env::call::#build_create_fn::<Self>()
                    .exec_input(
                        #exec_input_init
                        #(
                            .push_arg(#input_bindings)
                        )*
                    )
                    .returns::<#ret_type>()
            }
        )
    }
}
