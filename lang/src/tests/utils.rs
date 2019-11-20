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

use pretty_assertions::assert_eq;

use crate::generate_or_err;
use proc_macro2::TokenStream as TokenStream2;

pub fn assert_eq_tokenstreams(input: TokenStream2, expected: TokenStream2) {
    let result = generate_or_err(input)
        .map(|result| result.to_string())
        .map_err(|err| err.to_string());
    let expected = Ok(expected.to_string());
    assert_eq!(result, expected,)
}

pub fn assert_failure(input: TokenStream2, err_str: &'static str) {
    assert_eq!(
        generate_or_err(input)
            .map(|result| result.to_string())
            .map_err(|err| err.to_string()),
        Err(err_str.to_string())
    )
}
