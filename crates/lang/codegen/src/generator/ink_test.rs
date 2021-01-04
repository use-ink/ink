// Copyright 2018-2021 Parity Technologies (UK) Ltd.
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

/// Generates code for the `[ink::test]` macro.
#[derive(From)]
pub struct InkTest<'a> {
    /// The test function to generate code for.
    test: &'a ir::InkTest,
}

impl GenerateCode for InkTest<'_> {
    /// Generates the code for `#[ink:test]`.
    fn generate_code(&self) -> TokenStream2 {
        let item_fn = &self.test.item_fn;
        let attrs = &item_fn.attrs;
        let sig = &item_fn.sig;
        let fn_name = &sig.ident;
        let fn_return_type = &sig.output;
        let fn_block = &item_fn.block;
        let vis = &item_fn.vis;
        let fn_args = &sig.inputs;
        let expect_msg = format!(
            "{}: the off-chain testing environment returned an error",
            stringify!(#fn_name)
        );
        match fn_return_type {
            syn::ReturnType::Default => {
                quote! {
                    #( #attrs )*
                    #[test]
                    #vis fn #fn_name( #fn_args ) {
                        ::ink_env::test::run_test::<::ink_env::DefaultEnvironment, _>(|_| {
                            {
                                let _: () = {
                                    #fn_block
                                };
                                Ok(())
                            }
                        })
                        .expect(#expect_msg);
                    }
                }
            }
            syn::ReturnType::Type(rarrow, ret_type) => {
                quote! {
                    #( #attrs )*
                    #[test]
                    #vis fn #fn_name( #fn_args ) #rarrow #ret_type {
                        ::ink_env::test::run_test::<::ink_env::DefaultEnvironment, _>(|_| {
                            #fn_block
                        })
                    }
                }
            }
        }
    }
}

impl GenerateCode for ir::InkTest {
    fn generate_code(&self) -> TokenStream2 {
        InkTest::from(self).generate_code()
    }
}
