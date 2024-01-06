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

use crate::GenerateCode;
use derive_more::From;
use proc_macro2::TokenStream as TokenStream2;
use syn::spanned::Spanned;

/// Generates code for the event item.
#[derive(From, Copy, Clone)]
pub struct Event<'a> {
    /// The storage item to generate code for.
    item: &'a ir::Event,
}

impl GenerateCode for Event<'_> {
    /// Generates ink! event item code.
    fn generate_code(&self) -> TokenStream2 {
        let item = self.item.item();
        let anonymous = self
            .item
            .anonymous()
            .then(|| quote::quote! { #[ink(anonymous)] });
        let signature_topic = self
            .item
            .signature_topic_hash()
            .map(|hash| quote::quote! { #[ink(signature_topic = #hash)] });
        let cfg_attrs = self.item.get_cfg_attrs(item.span());

        quote::quote! (
            #( #cfg_attrs )*
            #[cfg_attr(feature = "std", derive(::ink::EventMetadata))]
            #[derive(::ink::Event)]
            #[::ink::scale_derive(Encode, Decode)]
            #anonymous
            #signature_topic
            #item
        )
    }
}
