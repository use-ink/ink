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

use crate::{
    config::{
        Backend,
        Node,
    },
    ir,
};
use derive_more::From;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

/// Generates code for the `[ink_e2e::test]` macro.
#[derive(From)]
pub struct InkE2ETest {
    /// The test function to generate code for.
    test: ir::InkE2ETest,
}

impl InkE2ETest {
    /// Generates the code for `#[ink_e2e:test]`.
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

        let chosen_test_attr = self
            .test
            .config
            .replace_test_attr()
            .unwrap_or_else(|| "#[test]".to_string());
        let possibly_fn_input = if chosen_test_attr == "#[test]" {
            quote! {}
        } else {
            let inputs = &item_fn.sig.inputs;
            quote! { #inputs }
        };

        let features = self.test.config.features();
        let exec_build_contracts = quote! {
            ::ink_e2e::build_root_and_contract_dependencies(
                vec![#( #features.to_string() ),*]
            )
        };

        let client_building = match self.test.config.backend() {
            Backend::Node(node_config) => {
                build_full_client(&environment, exec_build_contracts, node_config)
            }
            Backend::RuntimeOnly(args) => {
                let runtime: syn::Path = args.runtime_path();
                let client: syn::Path = args.client_path();
                build_runtime_client(exec_build_contracts, runtime, client)
            }
        };

        let parser = syn::Attribute::parse_outer;
        use syn::parse::Parser;
        let chosen_test_attr = parser
            .parse_str(&chosen_test_attr)
            .expect("Failed to parse attribute");

        quote! {
            #( #attrs )*
            #( #chosen_test_attr )*
            #vis fn #fn_name (#possibly_fn_input) #ret {
                use ::ink_e2e::log_info;
                ::ink_e2e::LOG_PREFIX.with(|log_prefix| {
                    let str = format!("test: {}", stringify!(#fn_name));
                    *log_prefix.borrow_mut() = String::from(str);
                });
                log_info("setting up e2e test");

                ::ink_e2e::INIT.call_once(|| {
                    // A global subscriber might already have been set up.
                    let _ = ::ink_e2e::tracing_subscriber::fmt::try_init();
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
    node_config: Node,
) -> TokenStream2 {
    match node_config.url() {
        Some(url) => {
            quote! {
                let contracts = #contracts;
                let rpc = ::ink_e2e::RpcClient::from_url(#url)
                    .await
                    .unwrap_or_else(|err|
                        ::core::panic!("Error connecting to node at {}: {err:?}", #url)
                    );
                let mut client = ::ink_e2e::Client::<
                    ::ink_e2e::PolkadotConfig,
                    #environment
                >::new(rpc, contracts, #url.to_string()).await
                    .expect("Failed creating Client");
            }
        }
        None => {
            quote! {
                let contracts = #contracts;
                let node_rpc = ::ink_e2e::TestNodeProcess::<::ink_e2e::PolkadotConfig>
                    ::build_with_env_or_default()
                    .spawn()
                    .await
                    .unwrap_or_else(|err|
                        ::core::panic!("Error spawning ink-node: {err:?}")
                    );
                let mut client = ::ink_e2e::Client::<
                    ::ink_e2e::PolkadotConfig,
                    #environment
                >::new(node_rpc.rpc(), contracts, node_rpc.url().to_string()).await
                    .expect("Failed creating Client");
            }
        }
    }
}

fn build_runtime_client(
    contracts: TokenStream2,
    runtime: syn::Path,
    client: syn::Path,
) -> TokenStream2 {
    quote! {
        let contracts = #contracts;
        let mut client = #client::<_, #runtime>::new(contracts);
    }
}
