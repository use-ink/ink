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

//! Code generation for smart contract ABI and metadata generation.
//!
//! The generated code here conflicts with all other generation purposes
//! and should always be generated in isolation.
//! It outputs code that needs to be run in order to generate the actual
//! metadata and ABI files.
//!
//! This two-steps process is required because Rust macros (and thus `ink_lang`)
//! are not able to access type information or anything that is related to that.

pub fn generate_code(tokens: &mut TokenStream2, contract: &hir::Contract) {
    let abi_mod_body = generate_abi_mod_body(contract);

    tokens.extend(quote! {
        #[cfg(feature = "abi")]
        mod abi {
            use super::*;

            #test_mod_body
        }
    })
}

fn generate_abi_mod_body(contract: &hir::Contract) -> TokenStream2 {
    let mut tokens = quote! {};
    // generate_test_struct(&mut tokens, contract);
    // generate_test_deploy(&mut tokens, contract);
    // generate_test_allocate_deploy_block(&mut tokens, contract);
    // generate_test_methods(&mut tokens, contract);

	

    tokens
}

