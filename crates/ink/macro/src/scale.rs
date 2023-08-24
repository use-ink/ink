// Copyright (C) Parity Technologies (UK) Ltd.
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
use quote::{
    quote_spanned,
};
use syn::spanned::Spanned;

/// Derives the `::ink::scale::Encode` trait for the given data type.
pub fn encode_derive(s: synstructure::Structure) -> TokenStream2 {
    let derive_input = s.ast();
    quote_spanned!(derive_input.span() =>
        #[derive(::ink::scale::Encode)]
        #[codec(crate = ::ink::scale)]
        #derive_input
    )
}

/// Derives the `::ink::scale::Decode` trait for the given data type.
pub fn decode_derive(s: synstructure::Structure) -> TokenStream2 {
    let derive_input = s.ast();
    quote_spanned!(derive_input.span() =>
        #[derive(::ink::scale::Decode)]
        #[codec(crate = ::ink::scale)]
        #derive_input
    )
}

/// Derives the `::ink::scale::Decode` trait for the given data type.
pub fn compact_as_derive(s: synstructure::Structure) -> TokenStream2 {
    let derive_input = s.ast();
    quote_spanned!(derive_input.span() =>
        #[derive(::ink::scale::CompactAs)]
        #[codec(crate = ::ink::scale)]
        #derive_input
    )
}

/// Derives the `::ink::scale::Decode` trait for the given data type.
pub fn type_info_derive(s: synstructure::Structure) -> TokenStream2 {
    let derive_input = s.ast();
    quote_spanned!(derive_input.span() =>
        #[derive(::ink::scale_info::TypeInfo)]
        #[codec(crate = ::ink::scale_info)]
        #derive_input
    )
}