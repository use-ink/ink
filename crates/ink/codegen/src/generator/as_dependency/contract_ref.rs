// Copyright (C) Parity Technologies (UK) Ltd.
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

use crate::{
    generator,
    GenerateCode,
};
use derive_more::From;
use ir::{
    Callable,
    IsDocAttribute as _,
};
use proc_macro2::TokenStream as TokenStream2;
use quote::{
    quote,
    quote_spanned,
};
use syn::spanned::Spanned as _;

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
    fn generate_contract_ref_ident(&self) -> syn::Ident {
        quote::format_ident!("{}Ref", self.contract.module().storage().ident())
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
            .cloned()
            .filter(syn::Attribute::is_doc_attribute);
        let storage_ident = self.contract.module().storage().ident();
        let ref_ident = self.generate_contract_ref_ident();
        quote_spanned!(span=>
            #[cfg_attr(feature = "std", derive(
                ::ink::storage::traits::StorageLayout,
            ))]
            #[derive(
                ::core::fmt::Debug,
                ::core::hash::Hash,
                ::core::cmp::PartialEq,
                ::core::cmp::Eq,
                ::core::clone::Clone,
            )]
            #[::ink::scale_derive(Encode, Decode, TypeInfo)]
            #( #doc_attrs )*
            pub struct #ref_ident {
                inner: <#storage_ident as ::ink::codegen::ContractCallBuilder>::Type,
            }

            const _: () = {
                impl ::ink::env::ContractReference for #storage_ident {
                    type Type = #ref_ident;
                }

                impl ::ink::env::ContractReverseReference for #ref_ident {
                    type Type = #storage_ident;
                }

                impl ::ink::env::call::ConstructorReturnType<#ref_ident> for #storage_ident {
                    type Output = #ref_ident;
                    type Error = ();

                    fn ok(value: #ref_ident) -> Self::Output {
                        value
                    }
                }

                impl<E> ::ink::env::call::ConstructorReturnType<#ref_ident>
                    for ::core::result::Result<#storage_ident, E>
                where
                    E: ::ink::scale::Decode
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

                impl ::ink::env::ContractEnv for #ref_ident {
                    type Env = <#storage_ident as ::ink::env::ContractEnv>::Env;
                }
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
            impl ::ink::env::call::FromAccountId<Environment> for #ref_ident {
                #[inline]
                fn from_account_id(account_id: AccountId) -> Self {
                    Self { inner: <<#storage_ident
                        as ::ink::codegen::ContractCallBuilder>::Type
                        as ::ink::env::call::FromAccountId<Environment>>::from_account_id(account_id)
                    }
                }
            }

            impl ::ink::ToAccountId<Environment> for #ref_ident {
                #[inline]
                fn to_account_id(&self) -> AccountId {
                    <<#storage_ident as ::ink::codegen::ContractCallBuilder>::Type
                        as ::ink::ToAccountId<Environment>>::to_account_id(&self.inner)
                }
            }

            impl ::core::convert::AsRef<AccountId> for #ref_ident {
                fn as_ref(&self) -> &AccountId {
                    <_ as ::core::convert::AsRef<AccountId>>::as_ref(&self.inner)
                }
            }

            impl ::core::convert::AsMut<AccountId> for #ref_ident {
                fn as_mut(&mut self) -> &mut AccountId {
                    <_ as ::core::convert::AsMut<AccountId>>::as_mut(&mut self.inner)
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
                impl ::ink::codegen::TraitCallBuilder for #ref_ident {
                    type Builder = <#storage_ident as ::ink::codegen::ContractCallBuilder>::Type;

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
        let messages = self.generate_contract_trait_impl_messages(trait_path, impl_block);
        quote_spanned!(span=>
            #( #attrs )*
            impl #trait_path for #forwarder_ident {
                type __ink_TraitInfo = <::ink::reflect::TraitDefinitionRegistry<Environment>
                    as #trait_path>::__ink_TraitInfo;

                #messages
            }
        )
    }

    /// Generates the code for all messages of a single ink! trait implementation of
    /// the ink! smart contract.
    fn generate_contract_trait_impl_messages(
        &self,
        trait_path: &syn::Path,
        impl_block: &ir::ItemImpl,
    ) -> TokenStream2 {
        impl_block
            .iter_messages()
            .map(|message| {
                self.generate_contract_trait_impl_for_message(trait_path, message)
            })
            .collect()
    }

    /// Generates the code for a single message of a single ink! trait implementation
    /// that is implemented by the ink! smart contract.
    fn generate_contract_trait_impl_for_message(
        &self,
        trait_path: &syn::Path,
        message: ir::CallableWithSelector<ir::Message>,
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
        let input_bindings = message.inputs().map(|input| &input.pat).collect::<Vec<_>>();
        let input_types = message.inputs().map(|input| &input.ty).collect::<Vec<_>>();
        let cfg_attrs = message.get_cfg_attrs(span);
        quote_spanned!(span=>
            #( #cfg_attrs )*
            type #output_ident =
                <<Self::__ink_TraitInfo as ::ink::codegen::TraitCallForwarder>::Forwarder as #trait_path>::#output_ident;

            #[inline]
            #( #cfg_attrs )*
            fn #message_ident(
                & #mut_token self
                #( , #input_bindings : #input_types )*
            ) -> Self::#output_ident {
                <_ as #trait_path>::#message_ident(
                    <_ as ::ink::codegen::TraitCallForwarderFor<{#trait_info_id}>>::#forward_operator(
                        <Self as ::ink::codegen::TraitCallBuilder>::#call_operator(self),
                    )
                    #( , #input_bindings )*
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
        self.contract
            .module()
            .impls()
            .filter(|impl_block| {
                // We are only interested in ink! trait implementation block.
                impl_block.trait_path().is_none()
            })
            .map(|impl_block| self.generate_contract_inherent_impl(impl_block))
            .collect()
    }

    /// Generates the code for a single ink! inherent implementation of the contract
    /// itself.
    ///
    /// # Note
    ///
    /// This produces the short-hand calling notation for the inherent contract
    /// implementation. The generated code simply forwards its calling logic to the
    /// associated call builder.
    fn generate_contract_inherent_impl(&self, impl_block: &ir::ItemImpl) -> TokenStream2 {
        let span = impl_block.span();
        let attrs = impl_block.attrs();
        let forwarder_ident = self.generate_contract_ref_ident();
        let messages = impl_block
            .iter_messages()
            .map(|message| self.generate_contract_inherent_impl_for_message(message));
        let constructors = impl_block.iter_constructors().map(|constructor| {
            self.generate_contract_inherent_impl_for_constructor(constructor)
        });
        quote_spanned!(span=>
            #( #attrs )*
            impl #forwarder_ident {
                #( #constructors )*
                #( #messages )*
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
        let input_bindings = message.inputs().map(|input| &input.pat).collect::<Vec<_>>();
        let input_types = message.inputs().map(|input| &input.ty).collect::<Vec<_>>();
        let output_type = message.output().map(|ty| quote! { -> #ty });
        let wrapped_output_type = message.wrapped_output();
        quote_spanned!(span=>
            #( #attrs )*
            #[inline]
            pub fn #message_ident(
                & #mut_token self
                #( , #input_bindings : #input_types )*
            ) #output_type {
                self.#try_message_ident( #( #input_bindings, )* )
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
                #( , #input_bindings : #input_types )*
            ) -> #wrapped_output_type {
                <Self as ::ink::codegen::TraitCallBuilder>::#call_operator(self)
                    .#message_ident( #( #input_bindings ),* )
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
    ) -> TokenStream2 {
        let span = constructor.span();
        let attrs = self
            .contract
            .config()
            .whitelisted_attributes()
            .filter_attr(constructor.attrs().to_vec());
        let constructor_ident = constructor.ident();
        let selector_bytes = constructor.composed_selector().hex_lits();
        let input_bindings = generator::input_bindings(constructor.inputs());
        let input_types = generator::input_types(constructor.inputs());
        let arg_list = generator::generate_argument_list(input_types.iter().cloned());
        let ret_type = constructor
            .output()
            .map(quote::ToTokens::to_token_stream)
            .unwrap_or_else(|| quote::quote! { Self });
        quote_spanned!(span =>
            #( #attrs )*
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn #constructor_ident(
                #( #input_bindings : #input_types ),*
            ) -> ::ink::env::call::CreateBuilder<
                Environment,
                Self,
                ::ink::env::call::utils::Unset<Hash>,
                ::ink::env::call::utils::Unset<u64>,
                ::ink::env::call::utils::Unset<Balance>,
                ::ink::env::call::utils::Set<::ink::env::call::ExecutionInput<#arg_list>>,
                ::ink::env::call::utils::Unset<::ink::env::call::state::Salt>,
                ::ink::env::call::utils::Set<::ink::env::call::utils::ReturnType<#ret_type>>,
            > {
                ::ink::env::call::build_create::<Self>()
                    .exec_input(
                        ::ink::env::call::ExecutionInput::new(
                            ::ink::env::call::Selector::new([ #( #selector_bytes ),* ])
                        )
                        #(
                            .push_arg(#input_bindings)
                        )*
                    )
                    .returns::<#ret_type>()
            }
        )
    }
}
