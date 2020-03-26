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
use quote::{
    quote,
    quote_spanned,
};

use crate::{
    codegen::{
        GenerateCode,
        GenerateCodeUsing,
    },
    ir,
    ir::utils,
};

#[derive(From)]
pub struct TestWrapper<'a> {
    contract: &'a ir::Contract,
}

impl GenerateCodeUsing for TestWrapper<'_> {
    fn contract(&self) -> &ir::Contract {
        &self.contract
    }
}

impl GenerateCode for TestWrapper<'_> {
    fn generate_code(&self) -> TokenStream2 {
        let wrapped_constructors = self.generate_constructors();
        let testable_storage_and_env = self.generate_testable_storage_and_env();

        quote! {
            #[cfg(all(test, feature = "test-env"))]
            pub use self::__ink_testable::TestableStorage;

            #[cfg(all(test, feature = "test-env"))]
            mod __ink_testable {
                use super::*;

                impl ink_lang::InstantiateTestable for Storage {
                    type Wrapped = TestableStorage;

                    fn instantiate() -> Self::Wrapped {
                        ink_core::env::test::initialize_as_default::<ink_core::env::DefaultEnvTypes>()
                            .expect("encountered already initialized off-chain environment");
                        let mut contract: Self = unsafe {
                            let mut alloc =
                                ink_core::storage::alloc::BumpAlloc::from_raw_parts(
                                    ink_primitives::Key([0x00; 32]),
                                );
                            ink_core::storage::alloc::AllocateUsing::allocate_using(
                                &mut alloc,
                            )
                        };
                        ink_core::storage::alloc::Initialize::try_default_initialize(
                            &mut contract,
                        );
                        contract.into()
                    }
                }

                pub use self::__ink_private::TestableStorage;
                mod __ink_private {
                    use super::*;

                    #testable_storage_and_env
                }

                impl TestableStorage {
                    #(
                        #wrapped_constructors
                    )*
                }
            }
        }
    }
}

impl TestWrapper<'_> {
    fn generate_constructors<'a>(&'a self) -> impl Iterator<Item = TokenStream2> + 'a {
        self.contract.functions.iter().filter_map(|function| {
            function
                .filter_constructor()
                .map(|kind| (function, kind))
        }).map(|(function, _kind)| {
            let span = function.span();
            let ident = &function.sig.ident;
            let fn_args = function.sig.inputs();
            let arg_idents = function.sig.inputs().map(move |fn_arg| &fn_arg.ident);

            quote_spanned!(span=>
                pub fn #ident(
                    #(#fn_args),*
                ) -> <Storage as ink_lang::InstantiateTestable>::Wrapped {
                    let mut contract = <Storage as ink_lang::InstantiateTestable>::instantiate();
                    contract.#ident(
                        #(
                            #arg_idents
                        ),*
                    );
                    contract
                }
            )
        })
    }

    fn generate_testable_storage_and_env(&self) -> TokenStream2 {
        let attrs = utils::filter_non_ink_attributes(&self.contract.storage.attrs);

        quote! {
            #( #attrs )*
            #[derive(Debug)]
            pub struct TestableStorage {
                contract: Storage,
            }

            impl From<Storage> for TestableStorage {
                fn from(contract: Storage) -> Self {
                    Self { contract }
                }
            }

            impl core::ops::Deref for TestableStorage {
                type Target = Storage;

                fn deref(&self) -> &Self::Target {
                    &self.contract
                }
            }

            impl core::ops::DerefMut for TestableStorage {
                fn deref_mut(&mut self) -> &mut Self::Target {
                    &mut self.contract
                }
            }
        }
    }
}
