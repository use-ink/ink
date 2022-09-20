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
use core::cell::RefCell;
use derive_more::From;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use std::{
    path::PathBuf,
    sync::Once,
};

/// We use this to only build the contract once for all tests.
static BUILD_ONCE: Once = Once::new();

// We save the name of the currently executing test here.
thread_local! {
    pub static CONTRACT_PATH: RefCell<Option<PathBuf>> = RefCell::new(None);
}

/// Returns the path to the contract bundle of the contract for which a test
/// is currently executed.
pub fn contract_path() -> Option<PathBuf> {
    CONTRACT_PATH.with(|metadata_path| metadata_path.borrow().clone())
}

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

        // This path will only be used in case `skip_build` is activated
        // and no path was specified for it.
        // TODO(#xxx) we should require specifying a path for `skip_build`.
        let mut path = PathBuf::from("./target/ink/metadata.json".to_string());

        // If a prior test did already build the contract and set the path
        // to the metadata file.
        if let Some(metadata_path) = contract_path() {
            path = metadata_path;
        }

        if !skip_build.value && contract_path().is_none() {
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
                let mut dest_metadata =
                    metadata["metadata_result"]["dest_bundle"].to_string();
                dest_metadata = dest_metadata.trim_matches('"').to_string();
                path = PathBuf::from(dest_metadata);
                log::info!("extracted metadata path: {}", path.display());

                CONTRACT_PATH.with(|metadata_path| {
                    *metadata_path.borrow_mut() = Some(path.clone());
                });
            });
        } else {
            BUILD_ONCE.call_once(|| {
                env_logger::init();
            });
        }

        log::info!("using metadata path: {:?}", path);

        path.try_exists().unwrap_or_else(|err| {
            panic!("path {:?} does not exist: {:?}", path, err);
        });
        let os_path = path
            .as_os_str()
            .to_str()
            .expect("converting path to str failed");
        let path = syn::LitStr::new(os_path, proc_macro2::Span::call_site());

        quote! {
            #( #attrs )*
            #[ink::env::e2e::tokio::test]
            async #vis fn #fn_name () #ret {
                use ink::env::e2e::log_info;
                ink::env::e2e::LOG_PREFIX.with(|log_prefix| {
                    let str = format!("test: {}", stringify!(#fn_name));
                    *log_prefix.borrow_mut() = String::from(str);
                });
                log_info("setting up e2e test");

                ink::env::e2e::INIT.call_once(|| {
                    ink::env::e2e::env_logger::init();
                });

                log_info("extracting metadata");
                // TODO(#xxx) `smart-bench_macro` needs to be forked.
                ink::env::e2e::smart_bench_macro::contract!(#path);

                log_info("creating new client");

                // TODO(#xxx) Make those two generic environments customizable.
                let mut client = ink::env::e2e::Client::<
                    ink::env::e2e::PolkadotConfig,
                    ink::env::DefaultEnvironment
                >::new(&#path, &#ws_url, &#node_log).await;

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
