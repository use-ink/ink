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
use proc_macro2::TokenStream as TokenStream2;
use quote::{
    format_ident,
    quote,
    ToTokens,
};
use syn::{
    parse2,
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
        let generated_struct = self.generate_struct();
        let generated_atomic_status = self.generate_atomic_status();
        let generated_storage_type = self.generate_storage_type();

        quote! {
            #generated_struct
            #generated_atomic_status
            #generated_storage_type
        }
    }
}

impl<'a> StorageItem<'a> {
    fn generate_struct(&self) -> TokenStream2 {
        let item = self.item;
        let ident = item.ident();
        let attrs = item.attrs();
        let vis = item.vis();
        let generics = item.generics();
        let salt = item.salt();

        let fields = item.fields().map(|field| {
            let key_bytes: Vec<u8> = if let Some(field_ident) = &field.ident {
                [
                    &ident.to_string().into_bytes()[..],
                    &field_ident.to_string().into_bytes()[..],
                ]
                .concat()
            } else {
                ident.to_string().into_bytes()
            };

            let key = Selector::compute(&key_bytes).into_be_u32();

            let mut new_field = field.clone();
            let ty = field.ty.clone();
            new_field.ty = parse2(quote! {
                <#ty as ::ink_storage::traits::StorageType<
                    ::ink_storage::traits::ManualKey<#key, #salt>,
                >>::Type
            })
            .unwrap();
            new_field
        });

        quote! {
            #(#attrs)*
            #vis struct #ident #generics {
                #(#fields),*
            }
        }
    }

    fn generate_atomic_status(&self) -> TokenStream2 {
        let item = self.item;
        let ident = item.ident();

        let (impl_generics, ty_generics, where_clause) = item.generics().split_for_impl();
        let inner_is_atomic: Vec<_> = item
            .fields()
            .map(|field| {
                let ty = field.ty.clone();
                quote! { <#ty as ::ink_storage::traits::AtomicStatus>::IS_ATOMIC }
            })
            .collect();

        quote! {
            impl #impl_generics ::ink_storage::traits::AtomicStatus for #ident #ty_generics #where_clause {
                const IS_ATOMIC: bool = #(#inner_is_atomic)&&*;
            }

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
}
