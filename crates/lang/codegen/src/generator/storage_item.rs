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
use ir::Selector;
use proc_macro2::{
    Ident,
    TokenStream as TokenStream2,
    TokenStream,
};
use quote::{
    format_ident,
    quote,
    ToTokens,
};
use syn::{
    parse2,
    Data,
    DataEnum,
    DataStruct,
    DataUnion,
    Field,
    Fields,
    GenericParam,
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
        let generated_struct = match self.item.data().clone() {
            Data::Struct(struct_item) => self.generate_struct(struct_item),
            Data::Enum(enum_item) => self.generate_enum(enum_item),
            Data::Union(union_item) => self.generate_union(union_item),
        };
        let generated_atomic_status = self.generate_atomic_status();
        let generated_storage_type = self.generate_storage_type();
        let generated_storage_key_holder = self.generate_storage_key_holder();

        quote! {
            #generated_struct
            #generated_atomic_status
            #generated_storage_type
            #generated_storage_key_holder
        }
    }
}

impl<'a> StorageItem<'a> {
    fn generate_struct(&self, struct_item: DataStruct) -> TokenStream2 {
        let item = self.item;
        let struct_ident = item.ident();
        let attrs = item.attrs();
        let vis = item.vis();
        let generics = item.generics();
        let salt = item.salt();

        let fields = struct_item.fields.iter().enumerate().map(|(i, field)| {
            convert_into_storage_field(struct_ident, None, &salt, i, field)
        });

        quote! {
            #(#attrs)*
            #vis struct #struct_ident #generics {
                #(#fields),*
            }
        }
    }

    fn generate_enum(&self, enum_item: DataEnum) -> TokenStream2 {
        let item = self.item;
        let enum_ident = item.ident();
        let attrs = item.attrs();
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
            #(#attrs)*
            #vis enum #enum_ident #generics {
                #(#variants),*
            }
        }
    }

    fn generate_union(&self, union_item: DataUnion) -> TokenStream2 {
        let item = self.item;
        let union_ident = item.ident();
        let attrs = item.attrs();
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
            #(#attrs)*
            #vis union #union_ident #generics {
                #(#fields),*
            }
        }
    }

    fn generate_atomic_status(&self) -> TokenStream2 {
        let item = self.item;
        let ident = item.ident();

        let (impl_generics, ty_generics, where_clause) = item.generics().split_for_impl();

        let types: Vec<_> = match item.data() {
            Data::Struct(st) => st.fields.iter().map(|field| field.ty.clone()).collect(),
            Data::Enum(en) => {
                en.variants
                    .iter()
                    .map(|variant| variant.fields.iter())
                    .flatten()
                    .map(|field| field.ty.clone())
                    .collect()
            }
            Data::Union(un) => {
                un.fields
                    .named
                    .iter()
                    .map(|field| field.ty.clone())
                    .collect()
            }
        };

        let inner_is_atomic: Vec<_> = types
            .iter()
            .map(|t| {
                quote! { ::ink_storage::is_atomic!(#t) }
            })
            .collect();

        quote! {
            impl #impl_generics ::ink_storage::traits::AtomicGuard< { #(#inner_is_atomic)&&* } >
                for #ident #ty_generics #where_clause {}
        }
    }

    fn generate_storage_type(&self) -> TokenStream2 {
        let item = self.item;
        let ident = item.ident();
        let (_, ty_generics, where_clause) = item.generics().split_for_impl();

        let mut generics = item.generics().clone();

        // If the generic salt is specified, then we add two implementations. One for `AutoKey`
        // and another for `ManualKey`. The implementation for `AutoKey` uses key and salt from the
        // `StorageType` trait. The `ManualKey` ignores the `StorageType` trait and uses its values.
        if item.has_specified_salt() {
            let salt_ident = item.salt_ident().unwrap();
            let manual_key_ident = format_ident!("__ink_generic_manual_key");
            let manual_salt_ident = format_ident!("__ink_generic_manual_salt");

            let mut auto_key_ty_generics = Vec::new();
            let mut manual_key_ty_generics = Vec::new();

            for param in item.generics().params.iter() {
                match param {
                    GenericParam::Type(t) => {
                        if t.ident == salt_ident {
                            auto_key_ty_generics.push(quote! {
                                ::ink_storage::traits::AutoKey
                            });
                            manual_key_ty_generics.push(quote! {
                                ::ink_storage::traits::ManualKey<
                                    #manual_key_ident,
                                    #manual_salt_ident
                                >
                            });
                        } else {
                            auto_key_ty_generics.push(t.ident.to_token_stream());
                            manual_key_ty_generics.push(t.ident.to_token_stream());
                        }
                    }
                    GenericParam::Lifetime(l) => {
                        auto_key_ty_generics.push(l.lifetime.to_token_stream());
                        manual_key_ty_generics.push(l.lifetime.to_token_stream());
                    }
                    GenericParam::Const(c) => {
                        auto_key_ty_generics.push(c.ident.to_token_stream());
                        manual_key_ty_generics.push(c.ident.to_token_stream());
                    }
                }
            }

            let auto_key_generics = generics.clone();
            let mut manual_key_generics = generics;

            manual_key_generics.params.push(
                parse2(quote! {
                    const #manual_key_ident : ::ink_primitives::StorageKey
                })
                .unwrap(),
            );
            manual_key_generics.params.push(
                parse2(quote! {
                    #manual_salt_ident : ::ink_storage::traits::StorageKeyHolder
                })
                .unwrap(),
            );
            let (auto_impl_generics, _, _) = auto_key_generics.split_for_impl();
            let (manual_impl_generics, _, _) = manual_key_generics.split_for_impl();

            quote! {
                impl #auto_impl_generics ::ink_storage::traits::StorageType<#salt_ident>
                    for #ident <#(#auto_key_ty_generics),*> #where_clause {

                    type Type = #ident <#(#auto_key_ty_generics),*>;
                }

                impl #manual_impl_generics ::ink_storage::traits::StorageType<#salt_ident>
                    for #ident <#(#manual_key_ty_generics),*> #where_clause {

                    type Type = #ident <#(#manual_key_ty_generics),*>;
                }
            }
        } else {
            let salt_ident = format_ident!("__ink_generic_salt");
            generics.params.push(
                parse2(quote! {
                    #salt_ident : ::ink_storage::traits::StorageKeyHolder
                })
                .unwrap(),
            );

            let (impl_generics, _, _) = generics.split_for_impl();
            quote! {
                impl #impl_generics ::ink_storage::traits::StorageType<#salt_ident>
                    for #ident #ty_generics #where_clause {

                    type Type = #ident #ty_generics;
                }
            }
        }
    }

    fn generate_storage_key_holder(&self) -> TokenStream2 {
        let item = self.item;
        let ident = item.ident();
        let salt = item.salt();

        let (impl_generics, ty_generics, where_clause) = item.generics().split_for_impl();

        quote! {
            impl #impl_generics ::ink_storage::traits::StorageKeyHolder for #ident #ty_generics #where_clause {
                const KEY: ::ink_primitives::StorageKey = <#salt as ::ink_storage::traits::StorageKeyHolder>::KEY;
            }
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
        <#ty as ::ink_storage::traits::StorageType<
            ::ink_storage::traits::ManualKey<#key, #salt>,
        >>::Type
    })
    .unwrap();
    new_field
}
