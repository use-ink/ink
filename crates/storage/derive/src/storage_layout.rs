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

fn field_layout<'a>(
    variant: &'a synstructure::VariantInfo,
) -> impl Iterator<Item = TokenStream2> + 'a {
    variant.ast().fields.iter().enumerate().map(|(i, field)| {
        let ident = match field.ident.as_ref() {
            Some(ident) => {
                let ident_str = ident.to_string();
                quote! { #ident_str }
            }
            None => {
                let index = i.to_string();
                quote! { #index }
            }
        };
        let ty = &field.ty;
        quote! {
            ::ink_metadata::layout::FieldLayout::new(
                #ident,
                <#ty as ::ink_storage::traits::StorageLayout>::layout(__key),
            )
        }
    })
}

fn storage_layout_struct(s: &synstructure::Structure) -> TokenStream2 {
    assert!(
        matches!(s.ast().data, syn::Data::Struct(_)),
        "s must be a struct item"
    );
    assert!(
        s.variants().len() == 1,
        "structs must have at most one variant"
    );
    let struct_ident = s.ast().ident.clone();
    let variant: &synstructure::VariantInfo = &s.variants()[0];
    let field_layouts = field_layout(variant);
    s.gen_impl(quote! {
        gen impl ::ink_storage::traits::StorageLayout for @Self {
            fn layout(__key: &::ink_primitives::StorageKey) -> ::ink_metadata::layout::Layout {
                ::ink_metadata::layout::Layout::Struct(
                    ::ink_metadata::layout::StructLayout::new(
                        ::core::stringify!(#struct_ident),
                        [
                            #(#field_layouts ,)*
                        ]
                    )
                )
            }
        }
    })
}

fn storage_layout_enum(s: &synstructure::Structure) -> TokenStream2 {
    assert!(
        matches!(s.ast().data, syn::Data::Enum(_)),
        "s must be an enum item"
    );
    let variant_layouts = s.variants().iter().enumerate().map(|(n, variant)| {
        let variant_ident = variant.ast().ident;
        let discriminant = variant
            .ast()
            .discriminant
            .as_ref()
            .map(|(_, expr)| quote! { #expr })
            .unwrap_or_else(|| quote! { #n });
        let field_layouts = field_layout(variant);
        quote! {
            {
                (
                    ::ink_metadata::layout::Discriminant::from(#discriminant),
                    ::ink_metadata::layout::StructLayout::new(
                        ::core::stringify!(#variant_ident),
                        [
                            #(#field_layouts ,)*
                        ]
                    ),
                )
            }
        }
    });
    let enum_ident = s.ast().ident.clone();
    s.gen_impl(quote! {
        gen impl ::ink_storage::traits::StorageLayout for @Self {
            fn layout(__key: &::ink_primitives::StorageKey) -> ::ink_metadata::layout::Layout {
                ::ink_metadata::layout::Layout::Enum(
                    ::ink_metadata::layout::EnumLayout::new(
                        ::core::stringify!(#enum_ident),
                        ::ink_metadata::layout::LayoutKey::from(__key),
                        [
                            #(#variant_layouts ,)*
                        ]
                    )
                )
            }
        }
    })
}

pub fn storage_layout_derive(mut s: synstructure::Structure) -> TokenStream2 {
    s.bind_with(|_| synstructure::BindStyle::Move)
        .add_bounds(synstructure::AddBounds::Fields)
        .underscore_const(true);
    match s.ast().data {
        syn::Data::Struct(_) => storage_layout_struct(&s),
        syn::Data::Enum(_) => storage_layout_enum(&s),
        _ => panic!("cannot derive `StorageLayout` for Rust `union` items"),
    }
}
