// Copyright 2018-2022 Parity Technologies (UK) Ltd.
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

/// Derives `ink_storage`'s `SpreadAllocate` trait for the given type.
pub fn spread_allocate_derive(mut s: synstructure::Structure) -> TokenStream2 {
    s.bind_with(|_| synstructure::BindStyle::Move)
        .add_bounds(synstructure::AddBounds::Generics)
        .underscore_const(true);
    match s.ast().data {
        syn::Data::Struct(_) => derive_struct(s),
        syn::Data::Enum(_) => {
            panic!("cannot derive `SpreadAllocate` for `enum` types")
        }
        syn::Data::Union(_) => {
            panic!("cannot derive `SpreadAllocate` for `union` types")
        }
    }
}

/// Derives `ink_storage`'s `SpreadAllocate` trait for the given `struct`.
fn derive_struct(s: synstructure::Structure) -> TokenStream2 {
    assert!(s.variants().len() == 1, "can only operate on structs");
    let variant = &s.variants()[0];
    let allocate_body = variant.construct(|field, _index| {
        let ty = &field.ty;
        quote! {
            <#ty as ::ink::storage::traits::SpreadAllocate>::allocate_spread(__key_ptr)
        }
    });
    s.gen_impl(quote! {
        gen impl ::ink::storage::traits::SpreadAllocate for @Self {
            fn allocate_spread(__key_ptr: &mut ::ink_primitives::KeyPtr) -> Self {
                #allocate_body
            }
        }
    })
}
