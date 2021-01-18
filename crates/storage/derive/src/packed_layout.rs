// Copyright 2018-2021 Parity Technologies (UK) Ltd.
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

/// Derives `ink_storage`'s `PackedLayout` trait for the given `struct` or `enum`.
pub fn packed_layout_derive(mut s: synstructure::Structure) -> TokenStream2 {
    s.bind_with(|_| synstructure::BindStyle::Move)
        .add_bounds(synstructure::AddBounds::Generics)
        .underscore_const(true);
    let pull_body = s.each(|binding| {
        quote! { ::ink_storage::traits::PackedLayout::pull_packed(#binding, __key); }
    });
    let push_body = s.each(|binding| {
        quote! { ::ink_storage::traits::PackedLayout::push_packed(#binding, __key); }
    });
    let clear_body = s.each(|binding| {
        quote! { ::ink_storage::traits::PackedLayout::clear_packed(#binding, __key); }
    });
    s.gen_impl(quote! {
        gen impl ::ink_storage::traits::PackedLayout for @Self {
            fn pull_packed(&mut self, __key: &::ink_primitives::Key) {
                match self { #pull_body }
            }
            fn push_packed(&self, __key: &::ink_primitives::Key) {
                match self { #push_body }
            }
            fn clear_packed(&self, __key: &::ink_primitives::Key) {
                match self { #clear_body }
            }
        }
    })
}
