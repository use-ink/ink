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

use proc_macro2::TokenStream as TokenStream2;

/// Types implementing this trait are code generators for the ink! language.
pub trait GenerateCode {
    /// Generates ink! contract code.
    fn generate_code(&self) -> TokenStream2;
}

/// Types implementing this trait can forward code generation to other generators.
pub trait GenerateCodeUsing: AsRef<ir::Contract> {
    /// Generates code using the given codegen module.
    fn generate_code_using<'a, G>(&'a self) -> TokenStream2
    where
        G: GenerateCode + From<&'a ir::Contract>;
}

impl<T> GenerateCodeUsing for T
where
    T: AsRef<ir::Contract>,
{
    fn generate_code_using<'a, G>(&'a self) -> TokenStream2
    where
        G: GenerateCode + From<&'a ir::Contract>,
    {
        <G as GenerateCode>::generate_code(&G::from(
            <Self as AsRef<ir::Contract>>::as_ref(self),
        ))
    }
}
