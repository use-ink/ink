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

use derive_more::From;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

pub use crate::{
    codegen::{
        cross_calling::CrossCalling,
        dispatch::Dispatch,
        env_types::EnvTypes,
        events::{
            EventHelpers,
            EventStructs,
        },
        metadata::GenerateMetadata,
        storage::Storage,
        GenerateCode,
        GenerateCodeUsing,
    },
    ir::InkTest,
};

/// Generates code for the `[ink::test]` macro.
#[derive(From)]
pub struct InkTestModule<'a> {
    /// The test function to generate code for.
    ink_test: &'a InkTest,
}

impl GenerateCode for InkTestModule<'_> {
    /// Generates the code for `#[ink:test]`.
    fn generate_code(&self) -> TokenStream2 {
        let item_fn = &self.ink_test.item_fn;
        let attrs = &item_fn.attrs;
        let sig = &item_fn.sig;
        let fn_name = &sig.ident;
        let fn_return_type = &sig.output;
        let fn_block = &item_fn.block;
        let _vis = &item_fn.vis;
        let _fn_args = &sig.inputs;
        match fn_return_type {
            syn::ReturnType::Default => {
                quote! {
                    #( #attrs )*
                    #[test]
                    fn #fn_name() {
                        env::test::run_test::<env::DefaultEnvTypes, _>(|_| {
                            #fn_block
                            Ok(())
                        })
                        .expect(&format!("{}: the off-chain testing environment returned an error", stringify!(#fn_name)));
                    }
                }
            }
            syn::ReturnType::Type(_rarrow, _type) => {
                quote! {
                    #( #attrs )*
                    #[test]
                    fn #fn_name() -> env::Result<()> {
                        env::test::run_test::<env::DefaultEnvTypes, _>(|_| {
                            #fn_block
                            Ok(())
                        })
                    }
                }
            }
        }
    }
}

impl GenerateCode for InkTest {
    fn generate_code(&self) -> TokenStream2 {
        InkTestModule::from(self).generate_code()
    }
}
