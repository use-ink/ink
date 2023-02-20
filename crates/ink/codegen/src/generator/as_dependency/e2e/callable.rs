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

use crate::{
    generator,
    GenerateCode,
};
use derive_more::From;
use ink_env::call::Call;
use ir::{
    Callable,
    IsDocAttribute as _,
};
use proc_macro2::TokenStream as TokenStream2;
use quote::{
    quote,
    quote_spanned,
};
use syn::spanned::Spanned as _;


pub struct ContractCallable<'a> {
    contract: &'a ir::Contract
}

impl GenerateCode for ContractCallable<'_> {
    fn generate_code(&self) -> TokenStream2 {
        quote! {

        }
    }
}

impl ContractCallable<'_> {
    fn generate_constructor(&self) -> TokenStream2 {
        quote!()
    }
}
