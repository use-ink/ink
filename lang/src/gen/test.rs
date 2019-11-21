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

//! Code generation for test generation of Wasm smart contracts.
//!
//! Test code is generated under the `#[cfg(test)]` compile flag.

use proc_macro2::TokenStream as TokenStream2;
use quote::{
    quote,
    ToTokens,
};
use syn::{
    self,
    punctuated::Punctuated,
    Token,
};

use crate::{
    ast,
    hir,
    ident_ext::IdentExt,
};

pub fn generate_code(tokens: &mut TokenStream2, contract: &hir::Contract) {
    let test_mod_body = generate_test_mod_body(contract);

    tokens.extend(quote! {
        #[cfg(test)]
        mod test {
            use super::*;

            #test_mod_body
        }
    })
}

fn generate_test_mod_body(contract: &hir::Contract) -> TokenStream2 {
    let mut tokens = quote! {};
    generate_test_struct(&mut tokens, contract);
    generate_test_deploy(&mut tokens, contract);
    generate_test_allocate_deploy_block(&mut tokens, contract);
    generate_test_methods(&mut tokens, contract);
    tokens
}

/// For a contract returns its testable type name.
///
/// # Example
///
/// For a contract called `Flipper` this returns `TestableFlipper`.
fn testable_contract_name(contract: &hir::Contract) -> proc_macro2::Ident {
    proc_macro2::Ident::from_str(["Testable", &contract.name.to_string()].concat())
}

fn generate_test_struct(tokens: &mut TokenStream2, contract: &hir::Contract) {
    let contract_name = &contract.name;
    let env_types = &contract.env_types_type;
    let testable_name = testable_contract_name(contract);
    tokens.extend(quote! {
        pub struct #testable_name {
            env: ink_model::ExecutionEnv<#contract_name, ink_core::env::ContractEnv<#env_types>>,
        }
    })
}

fn generate_test_deploy(tokens: &mut TokenStream2, contract: &hir::Contract) {
    let contract_name = &contract.name;
    let testable_name = testable_contract_name(contract);
    let deploy_fn_toks = {
        let mut content = quote! {};
        <Token![pub]>::default().to_tokens(&mut content);
        <Token![fn]>::default().to_tokens(&mut content);
        syn::Ident::from_str("deploy_mock").to_tokens(&mut content);
        syn::token::Paren::default().surround(&mut content, |inner| {
            contract
                .on_deploy
                .sig
                .inputs_without_self()
                .to_tokens(inner)
        });
        <Token![->]>::default().to_tokens(&mut content);
        testable_name.to_tokens(&mut content);
        syn::token::Brace::default().surround(&mut content, |inner| {
            let inputs = {
                let mut inputs: Punctuated<syn::Pat, Token![,]> = Default::default();
                for input in &contract.on_deploy.sig.inputs {
                    if let ast::FnArg::Typed(pat_ty) = input {
                        inputs.push((*pat_ty.pat).clone())
                    }
                }
                inputs
            };
            inner.extend(quote! {
                let mut mock = #testable_name::allocate();
                mock.deploy(#inputs);
                mock
            })
        });
        content
    };
    tokens.extend(quote! {
        impl #contract_name {
            /// Returns a testable version of the contract.
            #deploy_fn_toks
        }
    })
}

fn generate_test_allocate_deploy_block(
    tokens: &mut TokenStream2,
    contract: &hir::Contract,
) {
    let testable_name = testable_contract_name(contract);
    let mut impl_body = quote! {};
    generate_test_allocate_fn(&mut impl_body, contract);
    generate_test_deploy_fn(&mut impl_body, contract);
    tokens.extend(quote! {
        impl #testable_name {
            #impl_body
        }
    })
}

fn generate_test_allocate_fn(tokens: &mut TokenStream2, _contract: &hir::Contract) {
    tokens.extend(quote! {
        /// Allocates the testable contract storage.
        fn allocate() -> Self {
            use ink_core::storage::{
                Key,
                alloc::{
                    AllocateUsing as _,
                    Initialize as _,
                    BumpAlloc,
                },
            };
            Self {
                env: unsafe {
                    let mut alloc = BumpAlloc::from_raw_parts(Key([0x0; 32]));
                    ink_model::ExecutionEnv::allocate_using(&mut alloc).initialize_into(())
                }
            }
        }
    })
}

fn generate_test_deploy_fn(tokens: &mut TokenStream2, contract: &hir::Contract) {
    let log_params = contract.on_deploy.sig.inputs_without_self();
    let act_params = log_params.to_actual_params();
    tokens.extend(quote! {
        /// Deploys the testable contract by initializing it with the given values.
        fn deploy(&mut self, #log_params) {
            let (handler, state) = self.env.split_mut();
            state.deploy(handler, #act_params)
        }
    })
}

fn generate_test_methods(tokens: &mut TokenStream2, contract: &hir::Contract) {
    let impl_for = testable_contract_name(contract);
    let mut impl_body = quote! {};
    generate_test_method_fns(&mut impl_body, contract);
    tokens.extend(quote! {
        impl #impl_for {
            #impl_body
        }
    })
}

fn generate_test_method_fns(tokens: &mut TokenStream2, contract: &hir::Contract) {
    for msg in &contract.messages {
        generate_test_method_fn(tokens, msg);
    }
}

fn generate_test_method_fn(tokens: &mut TokenStream2, msg: &hir::Message) {
    <Token![pub]>::default().to_tokens(tokens);
    <Token![fn]>::default().to_tokens(tokens);
    msg.sig.ident.to_tokens(tokens);
    syn::token::Paren::default()
        .surround(tokens, |inner| msg.sig.inputs().to_tokens(inner));
    msg.sig.output.to_tokens(tokens);
    syn::token::Brace::default().surround(tokens, |inner| {
        let params = msg.sig.inputs_without_self().to_actual_params();
        let name = &msg.sig.ident;
        let split_impl = if msg.is_mut() {
            quote! { self.env.split_mut() }
        } else {
            quote! { self.env.split() }
        };
        inner.extend(quote! {
            let (handler, state) = #split_impl;
            state.#name(handler, #params)
        })
    });
}
