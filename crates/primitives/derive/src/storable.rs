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
use quote::{
    quote,
    quote_spanned,
};
use syn::spanned::Spanned;

/// `Storable` derive implementation for `struct` types.
fn storable_struct_derive(s: &synstructure::Structure) -> TokenStream2 {
    assert_eq!(s.variants().len(), 1, "can only operate on structs");
    let variant: &synstructure::VariantInfo = &s.variants()[0];
    let decode_body = variant.construct(|field, _index| {
        let ty = &field.ty;
        let span = ty.span();
        quote_spanned!(span =>
            <#ty as ::ink_primitives::traits::Storable>::decode(__input)?
        )
    });
    let encode_body = variant.each(|binding| {
        let span = binding.ast().ty.span();
        quote_spanned!(span =>
            ::ink_primitives::traits::Storable::encode(#binding, __dest);
        )
    });

    s.gen_impl(quote! {
         gen impl ::ink_primitives::traits::Storable for @Self {
            #[inline(always)]
            #[allow(non_camel_case_types)]
            fn decode<__ink_I: ::scale::Input>(__input: &mut __ink_I) -> ::core::result::Result<Self, ::scale::Error> {
                ::core::result::Result::Ok(#decode_body)
            }

            #[inline(always)]
            #[allow(non_camel_case_types)]
            fn encode<__ink_O: ::scale::Output + ?::core::marker::Sized>(&self, __dest: &mut __ink_O) {
                match self { #encode_body }
            }
         }
     })
}

/// `Storable` derive implementation for `enum` types.
fn storable_enum_derive(s: &synstructure::Structure) -> TokenStream2 {
    assert!(
        !s.variants().is_empty(),
        "encountered invalid empty enum type deriving Storable trait"
    );
    let decode_body = s
        .variants()
        .iter()
        .map(|variant| {
            variant.construct(|field, _index| {
                let ty = &field.ty;
                let span = ty.span();
                quote_spanned!(span =>
                    <#ty as ::ink_primitives::traits::Storable>::decode(__input)?
                )
            })
        })
        .enumerate()
        .fold(quote! {}, |acc, (index, variant)| {
            let index = index as u8;
            quote! {
                #acc
                #index => #variant,
            }
        });

    let encode_body = s.variants().iter().enumerate().map(|(index, variant)| {
        let pat = variant.pat();
        let index = index as u8;
        let fields = variant.bindings().iter().map(|field| {
            let span = field.ast().ty.span();
            quote_spanned!(span =>
                ::ink_primitives::traits::Storable::encode(#field, __dest);
            )
        });
        quote! {
             #pat => {
                 { <::core::primitive::u8 as ::ink_primitives::traits::Storable>::encode(&#index, __dest); }
                 #(
                     { #fields }
                 )*
             }
         }
    });
    s.gen_impl(quote! {
         gen impl ::ink_primitives::traits::Storable for @Self {
            #[inline(always)]
            #[allow(non_camel_case_types)]
            fn decode<__ink_I: ::scale::Input>(__input: &mut __ink_I) -> ::core::result::Result<Self, ::scale::Error> {
                ::core::result::Result::Ok(
                    match <::core::primitive::u8 as ::ink_primitives::traits::Storable>::decode(__input)? {
                        #decode_body
                        _ => unreachable!("encountered invalid enum discriminant"),
                    }
                )
            }

            #[inline(always)]
            #[allow(non_camel_case_types)]
            fn encode<__ink_O: ::scale::Output + ?::core::marker::Sized>(&self, __dest: &mut __ink_O) {
                match self {
                    #(
                        #encode_body
                    )*
                }
            }
         }
     })
}

/// Derives `ink_storage`'s `Storable` trait for the given `struct` or `enum`.
pub fn storable_derive(mut s: synstructure::Structure) -> TokenStream2 {
    s.bind_with(|_| synstructure::BindStyle::Move)
        .add_bounds(synstructure::AddBounds::Fields)
        .underscore_const(true);
    match &s.ast().data {
        syn::Data::Struct(_) => storable_struct_derive(&s),
        syn::Data::Enum(data) => {
            if s.variants().len() > 256 {
                return syn::Error::new(
                    data.variants.span(),
                    "Currently only enums with at most 256 variants are supported.",
                )
                .to_compile_error()
            }

            storable_enum_derive(&s)
        }
        _ => {
            panic!("cannot derive `Storable` for Rust `union` items")
        }
    }
}
