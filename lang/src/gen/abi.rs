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

//! Code generation for smart contract ABI and metadata generation.
//!
//! This two-steps process is required because Rust macros (and thus `ink_lang`)
//! are not able to access type information or anything that is related to that.

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

use crate::{
    ast,
    gen::selector_to_expr,
    hir,
};

/// Trims a doc string obtained from an attribute token stream into the actual doc string.
///
/// Practically speaking this method removes the trailing start `" = \""` and end `\"`
/// of documentation strings coming from Syn attribute token streams.
fn trim_doc_string(attr: &syn::Attribute) -> String {
    attr.tokens
        .to_string()
        .trim_start_matches('=')
        .trim_start()
        .trim_start_matches('r')
        .trim_start_matches('\"')
        .trim_end_matches('\"')
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

fn generate_abi_constructor(contract: &hir::Contract) -> TokenStream2 {
    let constructor = &contract.on_deploy;

    let args = constructor
        .sig
        .inputs
        .iter()
        .filter_map(ast::FnArg::is_typed)
        .map(|capt| {
            let name = match &*capt.pat {
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
            let type_spec = generate_type_spec_code(ty);
            quote! {
                ink_abi::MessageParamSpec::new(#name)
                    .of_type(#type_spec)
                    .done()
            }
        });
    let docs = constructor.docs().map(trim_doc_string);

    quote! {
        ink_abi::ConstructorSpec::new("on_deploy")
            .selector([0u8; 4])
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
        let selector = selector_to_expr(message.selector());
        let is_mut = message.is_mut();
        let docs = message.docs().map(trim_doc_string);
        let name = message.sig.ident.to_string();
        let inputs = message
            .sig
            .inputs
            .iter()
            .filter_map(ast::FnArg::is_typed)
            .map(|capt| {
                let name: String = match &*capt.pat {
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
                let type_spec = generate_type_spec_code(ty);
                quote! {
                    ink_abi::MessageParamSpec::new(#name)
                        .of_type(#type_spec)
                        .done()
                }
            });
        let ret_type = match &message.sig.output {
            syn::ReturnType::Default => {
                quote! {
                    ink_abi::ReturnTypeSpec::new(None)
                }
            }
            syn::ReturnType::Type(_, ty) => {
                let type_spec = generate_type_spec_code(&*ty);
                quote! {
                    ink_abi::ReturnTypeSpec::new(#type_spec)
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

fn generate_type_spec_code(ty: &syn::Type) -> TokenStream2 {
    fn without_display_name(ty: &syn::Type) -> TokenStream2 {
        quote! { ink_abi::TypeSpec::new::<#ty>() }
    }
    if let syn::Type::Path(type_path) = ty {
        if type_path.qself.is_some() {
            return without_display_name(ty)
        }
        let path = &type_path.path;
        if path.segments.is_empty() {
            return without_display_name(ty)
        }
        let segs = path
            .segments
            .iter()
            .map(|seg| seg.ident.to_string())
            .collect::<Vec<_>>();
        return quote! {
            ink_abi::TypeSpec::with_name_segs::<#ty, _>(vec![#(#segs),*].into_iter().map(AsRef::as_ref))
        }
    }
    without_display_name(ty)
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
            let type_spec = generate_type_spec_code(ty);
            quote! {
                ink_abi::EventParamSpec::new(stringify!(#name))
                    .of_type(#type_spec)
                    .indexed(#indexed)
                    .done()
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

    let constructor = generate_abi_constructor(contract);
    let messages = generate_abi_messages(contract);
    let events = generate_abi_events(contract);

    quote! {
        ink_abi::ContractSpec::new(#contract_name_lit)
            .constructors(vec![
                #constructor
            ])
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
