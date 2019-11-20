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

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
    self,
    parse::Result,
    parse_quote,
    punctuated::Punctuated,
    Data,
    DataStruct,
    DeriveInput,
    Field,
    Fields,
    Token,
};

pub fn generate(input: TokenStream2) -> TokenStream2 {
    match generate_impl(input) {
        Ok(output) => output,
        Err(err) => err.to_compile_error(),
    }
}

fn generate_impl(input: TokenStream2) -> Result<TokenStream2> {
    Ok(quote! {})
}
