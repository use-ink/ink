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
            return quote! {};
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

        let node_url = self.test.config.node_url();

        let client_building = match self.test.config.backend() {
            Backend::Full => {
                build_full_client(&environment, exec_build_contracts, node_url)
            }
            #[cfg(any(test, feature = "drink"))]
            Backend::RuntimeOnly { runtime } => {
                build_runtime_client(exec_build_contracts, runtime)
            }
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

fn build_full_client(
    environment: &syn::Path,
    contracts: TokenStream2,
    node_url: Option<String>,
) -> TokenStream2 {
    match node_url {
        Some(url) => {
            quote! {
                let rpc = ::ink_e2e::RpcClient::from_url(#url)
                    .await
                    .unwrap_or_else(|err|
                        ::core::panic!("Error connecting to Chopsticks node: {err:?}")
                    );
                let contracts = #contracts;
                let mut client = ::ink_e2e::Client::<
                    ::ink_e2e::PolkadotConfig,
                    #environment
                >::new(rpc, contracts).await?;
            }
        }
        None => {
            quote! {
                let node_rpc = ::ink_e2e::TestNodeProcess::<::ink_e2e::PolkadotConfig>
                    ::build_with_env_or_default()
                    .spawn()
                    .await
                    .unwrap_or_else(|err|
                        ::core::panic!("Error spawning substrate-contracts-node: {err:?}")
                    );
                let contracts = #contracts;
                let mut client = ::ink_e2e::Client::<
                    ::ink_e2e::PolkadotConfig,
                    #environment
                >::new(node_rpc.rpc(), contracts).await?;
            }
        }
    }
}

#[cfg(any(test, feature = "drink"))]
fn build_runtime_client(
    contracts: TokenStream2,
    runtime: Option<syn::Path>,
) -> TokenStream2 {
    let runtime =
        runtime.unwrap_or_else(|| syn::parse_quote! { ::ink_e2e::MinimalRuntime });
    quote! {
        let contracts = #contracts;
        let mut client = ::ink_e2e::DrinkClient::<_, _, #runtime>::new(contracts);
    }
}
