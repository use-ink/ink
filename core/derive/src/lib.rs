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

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;
extern crate proc_macro;

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

synstructure::decl_derive!([Flush] => flush_derive);
synstructure::decl_derive!([AllocateUsing] => allocate_using_derive);

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
