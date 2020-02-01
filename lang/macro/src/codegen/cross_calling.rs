// Copyright 2018-2019 Parity Technologies (UK) Ltd.
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

//! Generates code for contracts that are compiled as dependencies of other contracts.
//!
//! # Triggers
//!
//! Compiling a contract as dependency is controlled by two different ways:
//!
//! - Enabling the `ink-as-dependency` crate feature of a contract.
//! - Setting `compile_as_dependency = true` in the ink! contract header.
//!
//! Note that the latter is stronger than the former.
//! So setting `compile_as_dependency` will always result in compiling the contract
//! as dependency even though `ink-as-dependency` is not enabled or existent.
//!
//! # Codegen Conflicts
//!
//! Contracts that have been compiled as dependencies strip away most of the
//! other code that is generated for contracts normally. Mainly this conflicts
//! with contract dispatch, normal contract storage generation as well as ABI
//! generation.
//!
//! # Structure
//!
//! Contract storage structs that have been compiled as dependencies are more
//! similar to references to such contracts since they only contain a single
//! `AccountId` which acts as a reference to one instantiated contract of the same
//! type.
//!
//! Calls to those contracts will only encode their parameters and other calling
//! ABI required in order to dispatch the actual call through the running chain.
//!
//! # Usage
//!
//! Generally users should provide a single contract in a single crate.
//! So if a user requires two contracts where one contract calls the other
//! they should write two crates where one crate depends on the other.
//! The root contract should then by default enable the `ink-as-dependency`
//! crate feature of the dependend-on contract.
//! So enabling the `ink-as-dependency` crate feature is only ever done from
//! the outside of a contract crate.
//!
//! Compiling as dependency using the `compile_as_dependency` header setting
//! is inadvisable since it isn't flexible enough and should only ever be used
//! if users want to specify multiple contracts within the same crate because
//! only one of the contracts can ever not be compiled as dependency.
//! However, note that this use case should be rare since there has to be some
//! kind of code duplication for the actual definition of the same version of
//! the contracts that is not being compiled as dependency.

use derive_more::From;
use proc_macro2::TokenStream as TokenStream2;
use quote::{
    quote,
    quote_spanned,
};

use crate::{
    codegen::{
        GenerateCode,
        GenerateCodeUsing,
    },
    ir,
    ir::utils,
};

#[derive(From)]
pub struct CrossCallingConflictCfg<'a> {
    _contract: &'a ir::Contract,
}

impl GenerateCode for CrossCallingConflictCfg<'_> {
    fn generate_code(&self) -> TokenStream2 {
        quote! {
            #[cfg(not(feature = "ink-as-dependency"))]
        }
    }
}

#[derive(From)]
pub struct CrossCalling<'a> {
    contract: &'a ir::Contract,
}

impl GenerateCode for CrossCalling<'_> {
    fn generate_code(&self) -> TokenStream2 {
        let cfg_bounds = self.generate_cfg_bounds();
        let storage = self.generate_storage();
        let storage_impls = self.generate_storage_impls();
        let storage_fns = self.generate_storage_fns();
        let ref_forwarder = self.generate_ref_forwarder();
        let ref_mut_forwarder = self.generate_ref_mut_forwarder();

        quote! {
            #cfg_bounds
            mod __ink_cross_calling {
                use super::*;

                #storage
                #storage_fns
                #storage_impls
                #ref_forwarder
                #ref_mut_forwarder
            }

            #[cfg(feature = "ink-as-dependency")]
            pub use self::__ink_cross_calling::StorageAsDependency;
        }
    }
}

impl GenerateCodeUsing for CrossCalling<'_> {
    fn contract(&self) -> &ir::Contract {
        &self.contract
    }
}

impl CrossCalling<'_> {
    fn generate_cfg_bounds(&self) -> TokenStream2 {
        if self.contract.meta_info.is_compiled_as_dependency() {
            quote! {}
        } else {
            quote! {
                #[cfg(feature = "ink-as-dependency")]
            }
        }
    }

    fn generate_storage(&self) -> TokenStream2 {
        let attrs = utils::filter_non_ink_attributes(&self.contract.storage.attrs);

        quote! {
            #( #attrs )*
            #[derive(Clone, Debug, scale::Encode, scale::Decode)]
            #[cfg_attr(
                feature = "ink-generate-abi",
                derive(type_metadata::Metadata)
            )]
            pub struct StorageAsDependency {
                account_id: AccountId,
            }
        }
    }

    fn generate_storage_impls(&self) -> TokenStream2 {
        quote! {
            impl ink_core::storage::Flush for StorageAsDependency {}

            #[cfg(feature = "ink-generate-abi")]
            impl ink_core::storage::alloc::AllocateUsing for StorageAsDependency {
                unsafe fn allocate_using<A>(alloc: &mut A) -> Self
                where
                    A: ink_core::storage::alloc::Allocate,
                {
                    // We don't want to carry this implementation arround.
                    // Please remove as soon as possible.
                    unimplemented!()
                }
            }

            #[cfg(feature = "ink-generate-abi")]
            impl ink_abi::HasLayout for StorageAsDependency {
                fn layout(&self) -> ink_abi::StorageLayout {
                    ink_abi::LayoutStruct::new(
                        <Self as type_metadata::Metadata>::meta_type(), vec![]
                    ).into()
                }
            }

            impl ink_core::env::call::FromAccountId<EnvTypes> for StorageAsDependency {
                #[inline]
                fn from_account_id(account_id: AccountId) -> Self {
                    Self { account_id }
                }
            }

            impl ink_lang::ToAccountId<EnvTypes> for StorageAsDependency {
                #[inline]
                fn to_account_id(&self) -> AccountId {
                    self.account_id
                }
            }
        }
    }

    fn generate_storage_constructors<'a>(
        &'a self,
    ) -> impl Iterator<Item = TokenStream2> + 'a {
        self.contract
            .functions
            .iter()
            .filter(|function| function.is_constructor())
            .map(move |function| {
                let span = function.span();
                let ident = &function.sig.ident;
                let attrs = utils::filter_non_ink_attributes(&function.attrs);
                let fn_args = function.sig.inputs();
                let arg_idents = function.sig.inputs().map(|fn_arg| &fn_arg.ident);
                let selector = function
                    .selector()
                    .expect("constructors always have selectors");
                let selector_bytes = selector.as_bytes();

                quote_spanned!(span=>
                    #( #attrs )*
                    pub fn #ident(
                        #( #fn_args ),*
                    ) -> ink_core::env::call::InstantiateBuilder<
                        EnvTypes,
                        Self,
                        ink_core::env::call::state::Sealed,
                        ink_core::env::call::state::CodeHashUnassigned,
                    > {
                        ink_core::env::call::InstantiateParams::<EnvTypes, Self>::build(
                            ink_core::env::call::Selector::new([#( #selector_bytes ),*])
                        )
                        #(
                            .push_arg(&#arg_idents)
                        )*
                        .seal()
                    }
                )
            })
    }

    fn generate_storage_messages<'a>(
        &'a self,
    ) -> impl Iterator<Item = TokenStream2> + 'a {
        let storage_ident_lit = self.contract.storage.ident.to_string();
        self.contract
            .functions
            .iter()
            .filter(|function| function.is_message())
            .map(move |function| {
                let span = function.span();
                let ident = &function.sig.ident;
                let ident_lit = ident.to_string();
                let attrs = utils::filter_non_ink_attributes(&function.attrs);
                let fn_args = function.sig.inputs();
                let arg_idents = function.sig.inputs().map(|fn_arg| &fn_arg.ident);
                let output = &function.sig.output;
                let is_mut = function.sig.is_mut();
                let call_path = if is_mut {
                    quote! { ForwardCallMut::call_mut}
                } else {
                    quote! { ForwardCall::call }
                };
                let receiver = if is_mut {
                    quote! { &mut self }
                } else {
                    quote! { &self }
                };
                let failure_msg = match output {
                    syn::ReturnType::Default => {
                        format!(
                            "invocation of {}::{} message was invalid",
                            storage_ident_lit, ident_lit,
                        )
                    }
                    syn::ReturnType::Type(_, _) => {
                        format!(
                            "evaluation of {}::{} message was invalid",
                            storage_ident_lit, ident_lit,
                        )
                    }
                };

                quote_spanned!(span=>
                    #( #attrs )*
                    pub fn #ident(
                        #receiver ,
                        #(
                            #fn_args
                        ),*
                    ) #output {
                        ink_lang::#call_path(self)
                            .#ident( #( #arg_idents ),* )
                            .fire()
                            .expect(#failure_msg)
                    }
                )
            })
    }

    fn generate_storage_fns(&self) -> TokenStream2 {
        let storage_constructors = self.generate_storage_constructors();
        let storage_messages = self.generate_storage_messages();

        quote! {
            impl StorageAsDependency {
                #(
                    #storage_constructors
                )*
                #(
                    #storage_messages
                )*
            }
        }
    }

    fn generate_forwarding_messages<'a>(
        &'a self,
        pred: fn(function: &ir::Function) -> bool,
    ) -> impl Iterator<Item = TokenStream2> + 'a {
        self.contract
            .functions
            .iter()
            .filter(move |function| pred(function))
            .filter_map(|function| function.filter_message().map(|kind| (function, kind)))
            .map(|(function, kind)| {
                let span = function.span();
                let attrs = utils::filter_non_ink_attributes(&function.attrs);
                let ident = &function.sig.ident;
                let selector_bytes = kind.selector.as_bytes();
                let fn_args = function.sig.inputs();
                let arg_idents = function.sig.inputs().map(move |fn_arg| &fn_arg.ident);
                let ret_ty: Option<syn::Type> = match &function.sig.output {
                    syn::ReturnType::Default => None,
                    syn::ReturnType::Type(_, ty) => Some((&**ty).clone()),
                };
                let ret_ty_sig = if ret_ty.is_some() {
                    quote! { ink_core::env::call::ReturnType<#ret_ty> }
                } else {
                    quote! { () }
                };
                let ret_ty_param = if ret_ty.is_some() {
                    quote! { #ret_ty }
                } else {
                    quote! { () }
                };
                let instantiate_fn = if ret_ty.is_some() {
                    quote! { eval }
                } else {
                    quote! { invoke }
                };

                quote_spanned!(span=>
                    #( #attrs )*
                    pub fn #ident(
                        self,
                        #( #fn_args ),*
                    ) -> ink_core::env::call::CallBuilder<
                        EnvTypes, #ret_ty_sig, ink_core::env::call::state::Sealed
                    > {
                        ink_core::env::call::CallParams::<EnvTypes, #ret_ty_param>::#instantiate_fn(
                            ink_lang::ToAccountId::to_account_id(self.contract),
                            ink_core::env::call::Selector::new([ #( #selector_bytes ),* ]),
                        )
                        #(
                            .push_arg(&#arg_idents)
                        )*
                        .seal()
                    }
                )
            })
    }

    fn generate_ref_forwarder(&self) -> TokenStream2 {
        let forwarding_messages =
            self.generate_forwarding_messages(|function| !function.sig.is_mut());

        quote! {
            impl<'a> ink_lang::ForwardCall for &'a StorageAsDependency {
                type Forwarder = CallForwarder<'a>;

                #[inline]
                fn call(self) -> Self::Forwarder {
                    CallForwarder { contract: self }
                }
            }

            pub struct CallForwarder<'a> {
                contract: &'a StorageAsDependency,
            }

            impl CallForwarder<'_> {
                #(
                    #forwarding_messages
                )*
            }
        }
    }

    fn generate_ref_mut_forwarder(&self) -> TokenStream2 {
        let forwarding_messages =
            self.generate_forwarding_messages(|function| function.sig.is_mut());

        quote! {
            impl<'a> ink_lang::ForwardCallMut for &'a mut StorageAsDependency {
                type Forwarder = CallForwarderMut<'a>;

                #[inline]
                fn call_mut(self) -> Self::Forwarder {
                    CallForwarderMut { contract: self }
                }
            }

            pub struct CallForwarderMut<'a> {
                contract: &'a StorageAsDependency,
            }

            impl CallForwarderMut<'_> {
                #(
                    #forwarding_messages
                )*
            }
        }
    }
}
