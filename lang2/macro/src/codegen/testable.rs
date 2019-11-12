// Copyright 2018-2019 Parity Technologies (UK) Ltd.
// This file is part of ink!.
//
// ink! is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// ink! is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with ink!.  If not, see <http://www.gnu.org/licenses/>.

use crate::{
    codegen::{
        GenerateCode,
        GenerateCodeUsing,
    },
    ir,
    ir::utils,
};
use derive_more::From;
use proc_macro2::TokenStream as TokenStream2;
use quote::{
    quote,
    quote_spanned,
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
            pub use self::__ink_testable::TestableStorageAndEnv;

            #[cfg(all(test, feature = "test-env"))]
            mod __ink_testable {
                use super::*;

                impl ink_lang2::InstantiateTestable for StorageAndEnv {
                    type Wrapped = TestableStorageAndEnv;

                    fn instantiate() -> Self::Wrapped {
                        let mut contract: Self = unsafe {
                            let mut alloc =
                                ink_core::storage::alloc::BumpAlloc::from_raw_parts(
                                    ink_core::storage::Key([0x00; 32]),
                                );
                            ink_core::storage::alloc::AllocateUsing::allocate_using(
                                &mut alloc,
                            )
                        };
                        ink_core::env2::test::TestEnv::<ink_core::env2::DefaultSrmlTypes>::try_initialize()
                            .expect("encountered already initialized test environment");
                        ink_core::storage::alloc::Initialize::try_default_initialize(
                            &mut contract,
                        );
                        contract.into()
                    }
                }

                pub use self::__ink_private::TestableStorageAndEnv;
                mod __ink_private {
                    use super::*;

                    #testable_storage_and_env
                }

                impl TestableStorageAndEnv {
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
                ) -> <StorageAndEnv as ink_lang2::InstantiateTestable>::Wrapped {
                    let mut contract = <StorageAndEnv as ink_lang2::InstantiateTestable>::instantiate();
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
            pub struct TestableStorageAndEnv {
                contract: StorageAndEnv,
            }

            impl From<StorageAndEnv> for TestableStorageAndEnv {
                fn from(contract: StorageAndEnv) -> Self {
                    Self { contract }
                }
            }

            impl core::ops::Deref for TestableStorageAndEnv {
                type Target = StorageAndEnv;

                fn deref(&self) -> &Self::Target {
                    &self.contract
                }
            }

            impl core::ops::DerefMut for TestableStorageAndEnv {
                fn deref_mut(&mut self) -> &mut Self::Target {
                    &mut self.contract
                }
            }
        }
    }
}
