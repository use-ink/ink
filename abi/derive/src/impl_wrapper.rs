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

#[cfg(not(feature = "std"))]
use alloc::{
    format,
    string::{
        String,
        ToString,
    },
};

use proc_macro2::{
    Span,
    TokenStream as TokenStream2,
};
use quote::quote;
use syn::Ident;

pub fn wrap(
    ident: &Ident,
    trait_name: &'static str,
    impl_quote: TokenStream2,
) -> TokenStream2 {
    let mut renamed = String::from(format!("_IMPL_{}_FOR_", trait_name));
    renamed.push_str(ident.to_string().trim_start_matches("r#"));
    let dummy_const = Ident::new(&renamed, Span::call_site());

    quote! {
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const #dummy_const: () = {
            #[allow(unknown_lints)]
            #[cfg_attr(feature = "cargo-clippy", allow(useless_attribute))]
            #[allow(rust_2018_idioms)]
            use type_metadata as _type_metadata;
            use ink_abi as _ink_abi;

            #[cfg(not(feature = "std"))]
            extern crate alloc;

            #[cfg(feature = "std")]
            mod __core {
                pub use ::core::*;
                pub use ::std::{vec, vec::Vec};
            }

            #[cfg(not(feature = "std"))]
            mod __core {
                pub use ::core::*;
                pub use ::alloc::{vec, vec::Vec};
            }

            #impl_quote;
        };
    }
}
