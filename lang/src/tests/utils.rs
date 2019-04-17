// Copyright 2018-2019 Parity Technologies (UK) Ltd.
// This file is part of pDSL.
//
// pDSL is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// pDSL is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with pDSL.  If not, see <http://www.gnu.org/licenses/>.

use pretty_assertions::assert_eq;

use crate::contract_gen_impl2;
use proc_macro2::TokenStream as TokenStream2;

pub fn assert_eq_tokenstreams(input: TokenStream2, expected: TokenStream2) {
    assert_eq!(
        contract_gen_impl2(input)
            .map(|result| result.to_string())
            .map_err(|err| err.to_string()),
        Ok(expected.to_string())
    )
}

pub fn assert_failure(input: TokenStream2, err_str: &'static str) {
    assert_eq!(
        contract_gen_impl2(input)
            .map(|result| result.to_string())
            .map_err(|err| err.to_string()),
        Err(err_str.to_string())
    )
}
