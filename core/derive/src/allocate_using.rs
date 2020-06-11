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

pub(crate) fn allocate_using_derive(mut s: synstructure::Structure) -> TokenStream2 {
    // We cannot implement AllocateUsing on enums because we cannot specify
    // which variant we are going to use.
    if let syn::Data::Enum(ref _enum_data) = s.ast().data {
        panic!("cannot derive AllocateUsing for enums")
    }
    s.bind_with(|_| synstructure::BindStyle::Move);
    s.add_bounds(synstructure::AddBounds::Fields);
    // The `synstructure` crate treats every input as `enum`.
    // So even `struct`s are treated as single-variant enums.
    // Some line above we exclude `enum` for deriving this trait so
    // all inputs (`struct` only) have exactly one variant which
    // is the `struct` itself.
    let body = s.variants()[0].construct(|field, _| {
        let ty = &field.ty;
        quote! {
            <#ty as ink_core::storage::alloc::AllocateUsing>::allocate_using(alloc)
        }
    });
    s.gen_impl(quote! {
        gen impl ink_core::storage::alloc::AllocateUsing for @Self {
            unsafe fn allocate_using<A>(alloc: &mut A) -> Self
            where
                A: ink_core::storage::alloc::Allocate,
            {
                #body
            }
        }
    })
}
