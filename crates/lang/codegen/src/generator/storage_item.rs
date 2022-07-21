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
use proc_macro2::{
    Ident,
    TokenStream as TokenStream2,
    TokenStream,
};
use quote::{
    format_ident,
    quote,
    quote_spanned,
    ToTokens,
};
use syn::{
    spanned::Spanned,
    Data,
    DataEnum,
    DataStruct,
    DataUnion,
    Field,
    Fields,
    Type,
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
        if self.item.config().derive() {
            derive = quote! {
                #[cfg_attr(feature = "std", derive(
                    ::scale_info::TypeInfo,
                    ::ink_storage::traits::StorageLayout,
                ))]
                #[derive(
                    ::ink_storage::traits::Item,
                    ::ink_storage::traits::KeyHolder,
                    ::ink_storage::traits::Storable,
                )]
            };
        }

        let type_check = self.generate_type_check();

        quote! {
            #type_check

            #(#attrs)*
            #derive
            #generated_struct
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

        match struct_item.fields {
            Fields::Unnamed(_) => {
                quote! {
                    #vis struct #struct_ident #generics (
                        #(#fields),*
                    );
                }
            }
            _ => {
                quote! {
                    #vis struct #struct_ident #generics {
                        #(#fields),*
                    }
                }
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

    fn generate_type_check(&self) -> TokenStream2 {
        let fields = self
            .item
            .all_used_types()
            .into_iter()
            .enumerate()
            .map(|(i, ty)| {
                let field_name = format_ident!("field_{}", i);
                let span = ty.span();
                quote_spanned!(span =>
                    #field_name: #ty
                )
            });
        let generics = self.item.generics();
        let salt = self.item.salt();

        quote! {
            const _: () = {
                struct Check #generics {
                    salt: #salt,
                    #(#fields),*
                }
            };
        }
    }
}

fn convert_into_storage_field(
    struct_ident: &Ident,
    variant_ident: Option<&syn::Ident>,
    salt: &TokenStream,
    index: usize,
    field: &Field,
) -> Field {
    let field_name = if let Some(field_ident) = &field.ident {
        field_ident.to_string()
    } else {
        index.to_string()
    };

    let variant_name = if let Some(variant_ident) = variant_ident {
        variant_ident.to_string()
    } else {
        "".to_string()
    };

    let key = ink_primitives::KeyComposer::compute_key(
        struct_ident.to_string().as_str(),
        variant_name.as_str(),
        field_name.as_str(),
    );

    let mut new_field = field.clone();
    let ty = field.ty.clone().to_token_stream();
    let span = field.ty.span();
    let new_ty = Type::Verbatim(quote_spanned!(span =>
        <#ty as ::ink_storage::traits::AutoItem<
            ::ink_storage::traits::ManualKey<#key, #salt>,
        >>::Type
    ));
    new_field.ty = new_ty;
    new_field
}
