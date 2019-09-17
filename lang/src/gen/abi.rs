// Copyright 2018-2019 Parity Technologies (UK) Ltd.
// This file is part of ink!.
//
// ink! is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// ink! is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with ink!.  If not, see <http://www.gnu.org/licenses/>.

//! Code generation for smart contract ABI and metadata generation.
//!
//! This two-steps process is required because Rust macros (and thus `ink_lang`)
//! are not able to access type information or anything that is related to that.

use crate::{
    ast,
    hir,
};
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
    self,
    punctuated::Punctuated,
    Token,
};

/// Trims a doc string obtained from an attribute token stream into the actual doc string.
///
/// Practically speaking this method removes the trailing start `" = \""` and end `\"`
/// of documentation strings coming from Syn attribute token streams.
fn trim_doc_string(attr: &syn::Attribute) -> String {
    attr.tts
        .to_string()
        .trim_start_matches('=')
        .trim_start()
        .trim_start_matches("r")
        .trim_start_matches("\"")
        .trim_end_matches("\"")
        .trim()
        .into()
}

pub fn generate_code(tokens: &mut TokenStream2, contract: &hir::Contract) {
    let abi_mod_body = generate_abi_mod_body(contract);
    tokens.extend(abi_mod_body);
}

fn generate_abi_mod_body(contract: &hir::Contract) -> TokenStream2 {
    let ink_generate_abi_contract = generate_abi_contract(contract);
    let ink_generate_abi_layout = generate_abi_layout(contract);

    quote! {
        #[cfg(not(feature = "ink-as-dependency"))]
        #[cfg(feature = "ink-generate-abi")]
        pub fn ink_generate_abi() -> ink_abi::InkProject {
            let contract = {
                #ink_generate_abi_contract
            };
            let layout = {
                #ink_generate_abi_layout
            };
            ink_abi::InkProject::new(layout, contract)
        }
    }
}

fn generate_abi_deploy_handler(contract: &hir::Contract) -> TokenStream2 {
    let deploy_handler = &contract.on_deploy;

    let args = deploy_handler
        .decl
        .inputs
        .iter()
        .filter_map(ast::FnArg::is_captured)
        .map(|capt| {
            let name = match &capt.pat {
                syn::Pat::Ident(pat_ident) => {
                    if pat_ident.by_ref.is_none()
                        && pat_ident.mutability.is_none()
                        && pat_ident.subpat.is_none()
                    {
                        pat_ident.ident.to_string()
                    } else {
                        unreachable!("encountered invalid deploy argument")
                    }
                }
                syn::Pat::Path(pat_path) => {
                    if pat_path.qself.is_none()
                        && pat_path.path.leading_colon.is_none()
                        && pat_path.path.segments.len() == 1
                        && pat_path.path.segments[0].arguments.is_empty()
                    {
                        pat_path.path.segments[0].ident.to_string()
                    } else {
                        unreachable!("invalid arg name encountered")
                    }
                }
                _ => {
                    unreachable!(
                        "encountered invalid argument syntax: the only allowed is `ident : type`",
                    )
                }
            };
            let ty = &capt.ty;
            quote! {
                ink_abi::MessageParamSpec::new::<#ty>(#name)
            }
        })
        .collect::<Punctuated<_, Token![,]>>();
    let docs = deploy_handler.docs().map(trim_doc_string);

    quote! {
        ink_abi::DeploySpec::new()
            .args(vec![
                #(#args ,)*
            ])
            .docs(vec![
                #(#docs ,)*
            ])
            .done()
    }
}

fn generate_abi_messages<'a>(
    contract: &'a hir::Contract,
) -> impl Iterator<Item = TokenStream2> + 'a {
    contract.messages.iter().map(|message| {
        let selector = message.selector();
        let is_mut = message.is_mut();
        let docs = message.docs().map(trim_doc_string);
        let name = message.sig.ident.to_string();
        let inputs = message
            .sig
            .decl
            .inputs
            .iter()
            .filter_map(ast::FnArg::is_captured)
            .map(|capt| {
                let name: String = match &capt.pat {
                    syn::Pat::Ident(pat_ident) => {
                        if pat_ident.by_ref.is_none()
                            && pat_ident.mutability.is_none()
                            && pat_ident.subpat.is_none()
                        {
                            pat_ident.ident.to_string()
                        } else {
                            unreachable!("encountered invalid deploy argument")
                        }
                    }
                    syn::Pat::Path(pat_path) => {
                        if pat_path.qself.is_none()
                            && pat_path.path.leading_colon.is_none()
                            && pat_path.path.segments.len() == 1
                            && pat_path.path.segments[0].arguments.is_empty()
                        {
                            pat_path.path.segments[0].ident.to_string()
                        } else {
                            unreachable!("invalid arg name encountered")
                        }
                    }
                    _ => unreachable!("encountered invalid argument syntax: the only allowed is `ident : type`"),
                };
                let ty = &capt.ty;
                quote! {
                    ink_abi::MessageParamSpec::new::<#ty>(#name)
                }
            });
        let ret_type = match &message.sig.decl.output {
            syn::ReturnType::Default => {
                quote! {
                    ink_abi::ReturnTypeSpec::none()
                }
            }
            syn::ReturnType::Type(_, ty) => {
                quote! {
                    ink_abi::ReturnTypeSpec::new::<#ty>()
                }
            }
        };
        quote! {
            ink_abi::MessageSpec::new(#name)
                .selector(#selector)
                .mutates(#is_mut)
                .args(vec![
                    #(#inputs ,)*
                ])
                .docs(vec![
                    #(#docs ,)*
                ])
                .returns(
                    #ret_type
                )
                .done()
        }
    })
}

fn generate_abi_events<'a>(
    contract: &'a hir::Contract,
) -> impl Iterator<Item = TokenStream2> + 'a {
    contract.events.iter().map(|event| {
        let name = &event.ident;
        let args = event.args.iter().map(|event_arg| {
            let name = &event_arg.ident;
            let indexed = event_arg.is_indexed();
            let ty = &event_arg.ty;
            quote! {
                ink_abi::EventParamSpec::new::<#ty>(stringify!(#name), #indexed)
            }
        });
        let docs = event.docs().map(trim_doc_string);
        quote! {
            ink_abi::EventSpec::new(stringify!(#name))
                .args(vec![
                    #(#args ,)*
                ])
                .docs(vec![
                    #(#docs ,)*
                ])
                .done()
        }
    })
}

fn generate_abi_contract(contract: &hir::Contract) -> TokenStream2 {
    let contract_name = &contract.name;
    let contract_name_lit = contract_name.to_string();

    // We currently do not have a way to specify docs for whole contracts.
    // For this we could either take the docs of the contract state struct
    // or allow for inner-attribute doc style within the `contract!` macro call.
    let docs = quote! {};

    let deploy_handler = generate_abi_deploy_handler(contract);
    let messages = generate_abi_messages(contract);
    let events = generate_abi_events(contract);

    quote! {
        ink_abi::ContractSpec::new(#contract_name_lit)
            .on_deploy(
                #deploy_handler
            )
            .messages(vec![
                #(#messages ,)*
            ])
            .events(vec![
                #(#events ,)*
            ])
            .docs(vec![
                #docs
            ])
            .done()
    }
}

fn generate_abi_layout(contract: &hir::Contract) -> TokenStream2 {
    let contract_name = &contract.name;
    quote! {
        unsafe {
            use ink_core::storage::alloc::AllocateUsing as _;
            use ink_abi::HasLayout as _;
            #contract_name::allocate_using(
                &mut ink_core::storage::alloc::BumpAlloc::from_raw_parts(ink_core::storage::Key([0x0; 32]))
            ).layout()
        }
    }
}
