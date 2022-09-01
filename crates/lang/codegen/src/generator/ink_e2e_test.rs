// Copyright 2018-2022 Parity Technologies (UK) Ltd.
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

use crate::GenerateCode;
use derive_more::From;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use std::sync::Once;

/// We use this to only build the contract once for all tests.
static BUILD_ONCE: Once = Once::new();

/// Generates code for the `[ink::e2e_test]` macro.
#[derive(From)]
pub struct InkE2ETest<'a> {
    /// The test function to generate code for.
    test: &'a ir::InkE2ETest,
}

impl GenerateCode for InkE2ETest<'_> {
    /// Generates the code for `#[ink:e2e_test]`.
    fn generate_code(&self) -> TokenStream2 {
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

        let ws_url = &self.test.config.ws_url();
        let node_log = &self.test.config.node_log();
        let skip_build = &self.test.config.skip_build();

        let cargo_target_dir = std::env::var("CARGO_TARGET_DIR");
        let mut path = match cargo_target_dir {
            Ok(mut p) => {
                p.push_str("/metadata.json");
                p
            }
            Err(_) => "./target/ink/metadata.json".to_string(),
        };
        if !skip_build.value {
            BUILD_ONCE.call_once(|| {
                env_logger::init();
                use std::process::{
                    Command,
                    Stdio,
                };
                let output = Command::new("cargo")
                    // TODO(#xxx) Add possibility of configuring `skip_linting` in attributes.
                    .args(["+stable", "contract", "build", "--skip-linting", "--output-json"])
                    .env("RUST_LOG", "")
                    .stderr(Stdio::inherit())
                    .output()
                    .expect("failed to execute `cargo-contract` build process");

                log::info!("`cargo-contract` returned status: {}", output.status);
                eprintln!("`cargo-contract` returned status: {}", output.status);
                log::info!(
                    "`cargo-contract` stdout: {}",
                    String::from_utf8_lossy(&output.stdout)
                );
                eprintln!(
                    "`cargo-contract` stdout: {}",
                    String::from_utf8_lossy(&output.stdout)
                );
                if !output.status.success() {
                    log::info!(
                        "`cargo-contract` stderr: {}",
                        String::from_utf8_lossy(&output.stderr)
                    );
                    eprintln!(
                        "`cargo-contract` stderr: {}",
                        String::from_utf8_lossy(&output.stderr)
                    );
                }

                assert!(output.status.success());

                let json = String::from_utf8_lossy(&output.stdout);
                let metadata: serde_json::Value =
                    serde_json::from_str(&json).expect("cannot convert json to utf8");
                let dest_metadata =
                    metadata["metadata_result"]["dest_metadata"].to_string();
                path = dest_metadata.trim_matches('"').to_string();
                log::info!("extracted metadata path: {}", path);
                eprintln!("extracted metadata path: {}", path);
            });
        }

        log::info!("using metadata path: {}", path);
        eprintln!("using metadata path: {}", path);

        std::path::Path::new(&path)
            .try_exists()
            .unwrap_or_else(|err| {
                panic!("path {:?} does not exist: {:?}", path, err);
            });

        quote! {
            #( #attrs )*
            #[ink_env::e2e::tokio::test]
            async #vis fn #fn_name () #ret {
                use ink_env::e2e::log_info;
                ink_env::e2e::LOG_PREFIX.with(|log_prefix| {
                    let str = format!("test: {}", stringify!(#fn_name));
                    *log_prefix.borrow_mut() = String::from(str);
                });
                log_info("setting up e2e test");

                ink_env::e2e::INIT.call_once(|| {
                    ink_env::e2e::env_logger::init();
                });

                log_info("extracting metadata");
                // TODO(#xxx) `smart-bench_macro` needs to be forked.
                ink_env::e2e::smart_bench_macro::contract!(#path);

                let url = #ws_url;
                let node_log = #node_log;

                log_info("creating new client");

                // TODO(#xxx) Make those two generic environments customizable.
                let mut client = ink_env::e2e::Client::<ink_env::e2e::PolkadotConfig, ink_env::DefaultEnvironment>::new(&url, &node_log).await;

                let __ret = {
                    #block
                };
                __ret
            }
        }
    }
}

impl GenerateCode for ir::InkE2ETest {
    fn generate_code(&self) -> TokenStream2 {
        InkE2ETest::from(self).generate_code()
    }
}
