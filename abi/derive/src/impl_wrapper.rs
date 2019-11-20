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
    let mut renamed = format!("_IMPL_{}_FOR_", trait_name);
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
