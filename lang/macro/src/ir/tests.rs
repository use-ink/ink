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

//! Tests for the ink! IR module.

use core::convert::TryFrom;

use crate::ir::{
    Marker,
    Params,
};

#[test]
fn parse_meta_storage() {
    let input: syn::Attribute = syn::parse_quote! { #[ink(storage)] };
    let result = Marker::try_from(input);
    assert!(result.is_ok());
    assert!(result.unwrap().is_simple("storage"));
}

#[test]
fn parse_meta_event() {
    let input: syn::Attribute = syn::parse_quote! { #[ink(event)] };
    let result = Marker::try_from(input);
    assert!(result.is_ok());
    assert!(result.unwrap().is_simple("event"));
}

#[test]
fn parse_params() {
    let _input: Params = syn::parse_quote! {
        env = DefaultEnvTypes, version = "0.1.0"
    };
}
