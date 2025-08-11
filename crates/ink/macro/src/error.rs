// Copyright (C) Use Ink (UK) Ltd.
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
use quote::quote_spanned;
use syn::spanned::Spanned;

/// Derive error traits depending on the ink! project ABI.
pub fn derive(attr: TokenStream2, item: TokenStream2) -> TokenStream2 {
    quote_spanned!(attr.span() =>
        #[cfg_attr(not(ink_abi = "sol"), ::ink::scale_derive(Encode, Decode, TypeInfo))]
        #[cfg_attr(
            any(ink_abi = "sol", ink_abi = "all"),
            derive(::ink::SolErrorDecode, ::ink::SolErrorEncode)
        )]
        #[cfg_attr(
            all(feature = "std", any(ink_abi = "sol", ink_abi = "all")),
            derive(::ink::SolErrorMetadata)
        )]
        #item
    )
}
