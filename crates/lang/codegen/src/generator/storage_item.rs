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

use crate::GenerateCode;
use derive_more::From;
use ink_storage_codegen::DeriveUtils;
use ir::Selector;
use proc_macro2::{
    Ident,
    TokenStream as TokenStream2,
    TokenStream,
};
use quote::{
    format_ident,
    quote,
};
use syn::{
    parse2,
    Data,
    DataEnum,
    DataStruct,
    DataUnion,
    Field,
    Fields,
};

/// Generates code for the storage item.
#[derive(From, Copy, Clone)]
pub struct StorageItem<'a> {
    /// The storage item to generate code for.
    item: &'a ir::StorageItem,
}

impl GenerateCode for StorageItem<'_> {
    /// Generates ink! storage item code.
    fn generate_code(&self) -> TokenStream2 {
        let attrs = self.item.attrs();
        let generated_struct = match self.item.data().clone() {
            Data::Struct(struct_item) => self.generate_struct(struct_item),
            Data::Enum(enum_item) => self.generate_enum(enum_item),
            Data::Union(union_item) => self.generate_union(union_item),
        };

        let mut derive = quote! {};
        let mut impls = quote! {};
        if self.item.config().derive() {
            derive = quote! {
                #[derive(
                    ::ink_storage::traits::StorageType,
                    ::ink_storage::traits::StorageKeyHolder,
                    ::scale::Encode,
                    ::scale::Decode,
                )]
            };
            // Derive `AtomicGuard` requires `AtomicGuard<true>` for all types.
            // For storage item we try to calculate is the struct is atomic or not
            // via `ink_storage::is_atomic` macro.
            impls = self.generic_atomic_guard();
        }

        quote! {
            #(#attrs)*
            #derive
            #generated_struct
            #impls
        }
    }
}

impl<'a> StorageItem<'a> {
    fn generate_struct(&self, struct_item: DataStruct) -> TokenStream2 {
        let item = self.item;
        let struct_ident = item.ident();
        let vis = item.vis();
        let generics = item.generics();
        let salt = item.salt();

        let fields = struct_item.fields.iter().enumerate().map(|(i, field)| {
            convert_into_storage_field(struct_ident, None, &salt, i, field)
        });

        quote! {
            #vis struct #struct_ident #generics {
                #(#fields),*
            }
        }
    }

    fn generate_enum(&self, enum_item: DataEnum) -> TokenStream2 {
        let item = self.item;
        let enum_ident = item.ident();
        let vis = item.vis();
        let generics = item.generics();
        let salt = item.salt();

        let variants = enum_item.variants.into_iter().map(|variant| {
            let attrs = variant.attrs;
            let variant_ident = &variant.ident;
            let discriminant = if let Some((eq, expr)) = variant.discriminant {
                quote! { #eq #expr}
            } else {
                quote! {}
            };

            let fields: Vec<_> = variant
                .fields
                .iter()
                .enumerate()
                .map(|(i, field)| {
                    convert_into_storage_field(
                        enum_ident,
                        Some(variant_ident),
                        &salt,
                        i,
                        field,
                    )
                })
                .collect();

            let fields = match variant.fields {
                Fields::Named(_) => quote! { { #(#fields),* } },
                Fields::Unnamed(_) => quote! { ( #(#fields),* ) },
                Fields::Unit => quote! {},
            };

            quote! {
                #(#attrs)*
                #variant_ident #fields #discriminant
            }
        });

        quote! {
            #vis enum #enum_ident #generics {
                #(#variants),*
            }
        }
    }

    fn generate_union(&self, union_item: DataUnion) -> TokenStream2 {
        let item = self.item;
        let union_ident = item.ident();
        let vis = item.vis();
        let generics = item.generics();
        let salt = item.salt();

        let fields = union_item
            .fields
            .named
            .iter()
            .enumerate()
            .map(|(i, field)| {
                convert_into_storage_field(union_ident, None, &salt, i, field)
            });

        quote! {
            #vis union #union_ident #generics {
                #(#fields),*
            }
        }
    }

    fn generic_atomic_guard(&self) -> TokenStream2 {
        let ident = self.item.ident();

        let (impl_generics, ty_generics, where_clause) =
            self.item.generics().split_for_impl();

        let mut inner_is_atomic: Vec<_> = self
            .item
            .ast()
            .all_types()
            .iter()
            .map(|t| {
                quote! { ::ink_storage::is_atomic!(#t) }
            })
            .collect();

        if inner_is_atomic.is_empty() {
            inner_is_atomic.push(quote! { true })
        }

        quote! {
            impl #impl_generics ::ink_storage::traits::AtomicGuard< { #(#inner_is_atomic)&&* } >
                for #ident #ty_generics #where_clause {}
        }
    }
}

/// # Note
///
/// - `variant_ident` is `None` for structures and unions.
/// - if the field is unnamed then `field_ident` is `field_{}` where `{}` is a number of the field.
///
/// Evaluates the storage key of the field in the structure, variant or union.
///
/// 1. Compute the ASCII byte representation of `struct_ident` and call it `S`.
/// 1. If `variant_ident` is `Some` then computes the ASCII byte representation and call it `V`.
/// 1. Compute the ASCII byte representation of `field_ident` and call it `F`.
/// 1. Concatenate (`S` and `F`) or (`S`, `V` and `F`) using `::` as separator and call it `C`.
/// 1. Apply the `BLAKE2` 256-bit hash `H` of `C`.
/// 1. The first 4 bytes of `H` make up the storage key.
fn compute_storage_key(
    struct_ident: &syn::Ident,
    variant_ident: Option<&syn::Ident>,
    field_ident: &syn::Ident,
) -> u32 {
    let separator = &b"::"[..];
    let composed_key = if let Some(variant) = variant_ident {
        [
            &struct_ident.to_string().into_bytes()[..],
            &variant.to_string().into_bytes()[..],
            &field_ident.to_string().into_bytes()[..],
        ]
        .join(separator)
    } else {
        [
            &struct_ident.to_string().into_bytes()[..],
            &field_ident.to_string().into_bytes()[..],
        ]
        .join(separator)
    };

    Selector::compute(&composed_key).into_be_u32()
}

fn convert_into_storage_field(
    struct_ident: &Ident,
    variant_ident: Option<&syn::Ident>,
    salt: &TokenStream,
    index: usize,
    field: &Field,
) -> Field {
    let field_ident = if let Some(field_ident) = &field.ident {
        field_ident.clone()
    } else {
        format_ident!("field_{}", index)
    };

    let key = compute_storage_key(&struct_ident, variant_ident, &field_ident);

    let mut new_field = field.clone();
    let ty = field.ty.clone();
    new_field.ty = parse2(quote! {
        <#ty as ::ink_storage::traits::AutoStorageType<
            ::ink_storage::traits::ManualKey<#key, #salt>,
        >>::Type
    })
    .unwrap();
    new_field
}
