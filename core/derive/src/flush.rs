// Copyright 2018-2020 Parity Technologies (UK) Ltd.
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

pub(crate) fn flush_derive(mut s: synstructure::Structure) -> TokenStream2 {
    if s.variants().is_empty() {
        panic!("deriving Flush for empty enum is invalid")
    }
    s.bind_with(|_| synstructure::BindStyle::Move);
    s.add_bounds(synstructure::AddBounds::Fields);
    // Upon seeing the first variant we set this to true.
    // This is needed so that we do not have a `match self`
    // for empty enums which apparently causes errors.
    // If there is a better solution to tackle this please
    // update this.
    let mut requires_match = false;
    let body = s.each(|bi| {
        requires_match = true;
        quote! {
            ink_core::storage::Flush::flush(#bi)
        }
    });
    let body = if requires_match {
        quote! {
            match self {
                #body
            }
        }
    } else {
        quote! {}
    };
    s.gen_impl(quote! {
        gen impl ink_core::storage::Flush for @Self {
            fn flush(&mut self) {
                #body
            }
        }
    })
}
