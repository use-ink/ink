// Copyright 2018-2020 Parity Technologies (UK) Ltd.
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

/// Generates `#[cfg(..)]` code to guard against compilation under `ink-as-dependency`.
#[derive(From)]
pub struct CrossCallingConflictCfg<'a> {
    contract: &'a ir::Contract,
}

impl GenerateCode for CrossCallingConflictCfg<'_> {
    fn generate_code(&self) -> TokenStream2 {
        if self.contract.config().is_compile_as_dependency_enabled() {
            return quote! { #[cfg(feature = "__ink_DO_NOT_COMPILE")] }
        }
        quote! { #[cfg(not(feature = "ink-as-dependency"))] }
    }
}
