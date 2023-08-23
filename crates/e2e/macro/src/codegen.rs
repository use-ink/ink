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
    config::Backend,
    ir,
};
use derive_more::From;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

const DEFAULT_CONTRACTS_NODE: &str = "substrate-contracts-node";

/// Generates code for the `[ink::e2e_test]` macro.
#[derive(From)]
pub struct InkE2ETest {
    /// The test function to generate code for.
    test: ir::InkE2ETest,
}

impl InkE2ETest {
    /// Generates the code for `#[ink:e2e_test]`.
    pub fn generate_code(&self) -> TokenStream2 {
        #[cfg(clippy)]
        if true {
            return quote! {}
        }

        let item_fn = &self.test.item_fn.item_fn;
        let fn_name = &item_fn.sig.ident;
        let block = &item_fn.block;
        let fn_return_type = &item_fn.sig.output;
        let vis = &item_fn.vis;
        let attrs = &item_fn.attrs;
        let ret = match fn_return_type {
            syn::ReturnType::Default => quote! {},
            syn::ReturnType::Type(rarrow, ret_type) => quote! { #rarrow #ret_type },
        };

        let environment = self
            .test
            .config
            .environment()
            .unwrap_or_else(|| syn::parse_quote! { ::ink::env::DefaultEnvironment });

        let additional_contracts = self.test.config.additional_contracts();

        let exec_build_contracts = if additional_contracts.is_empty() {
            quote! {
                ::ink_e2e::build_root_and_contract_dependencies()
            }
        } else {
            quote! {
                ::ink_e2e::build_root_and_additional_contracts([ #( #additional_contracts ),* ])
            }
        };

        let client_building = match self.test.config.backend() {
            Backend::Full => build_full_client(&environment, exec_build_contracts),
            Backend::RuntimeOnly => build_runtime_client(exec_build_contracts),
        };

        quote! {
            #( #attrs )*
            #[test]
            #vis fn #fn_name () #ret {
                use ::ink_e2e::log_info;
                ::ink_e2e::LOG_PREFIX.with(|log_prefix| {
                    let str = format!("test: {}", stringify!(#fn_name));
                    *log_prefix.borrow_mut() = String::from(str);
                });
                log_info("setting up e2e test");

                ::ink_e2e::INIT.call_once(|| {
                    ::ink_e2e::tracing_subscriber::fmt::init();
                });

                log_info("creating new client");

                let run = async {
                    #client_building

                    let __ret = {
                        #block
                    };
                    __ret
                };

                {
                    return ::ink_e2e::tokio::runtime::Builder::new_current_thread()
                        .enable_all()
                        .build()
                        .unwrap_or_else(|err| panic!("Failed building the Runtime: {err}"))
                        .block_on(run);
                }
            }
        }
    }
}

fn build_full_client(environment: &syn::Path, contracts: TokenStream2) -> TokenStream2 {
    // Use the user supplied `CONTRACTS_NODE` or default to `DEFAULT_CONTRACTS_NODE`.
    let contracts_node: &'static str =
        option_env!("CONTRACTS_NODE").unwrap_or(DEFAULT_CONTRACTS_NODE);

    // Check the specified contracts node.
    if which::which(contracts_node).is_err() {
        if contracts_node == DEFAULT_CONTRACTS_NODE {
            panic!(
                "The '{DEFAULT_CONTRACTS_NODE}' executable was not found. Install '{DEFAULT_CONTRACTS_NODE}' on the PATH, \
                    or specify the `CONTRACTS_NODE` environment variable.",
            )
        } else {
            panic!("The contracts node executable '{contracts_node}' was not found.")
        }
    }

    quote! {
        // Spawn a contracts node process just for this test.
        let node_proc = ::ink_e2e::TestNodeProcess::<::ink_e2e::PolkadotConfig>
            ::build(#contracts_node)
            .spawn()
            .await
            .unwrap_or_else(|err|
                ::core::panic!("Error spawning substrate-contracts-node: {err:?}")
            );

        let contracts = #contracts;
        let mut client = ::ink_e2e::Client::<
            ::ink_e2e::PolkadotConfig,
            #environment
        >::new(node_proc.client(), contracts).await;
    }
}

fn build_runtime_client(contracts: TokenStream2) -> TokenStream2 {
    quote! {
        let contracts = #contracts;
        let mut client = ::ink_e2e::DrinkClient::new(contracts);
    }
}
