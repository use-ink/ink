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

use ink_storage_codegen::DeriveUtils;
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

fn common_storage_type_derive(mut s: synstructure::Structure) -> TokenStream2 {
    let (_, ty_generics, _) = s.ast().generics.split_for_impl();
    let ident = s.ast().ident.clone();

    let salt_ident = format_ident!("__ink_generic_salt");

    s.add_impl_generic(
        parse2(quote! {
            #salt_ident : ::ink_storage::traits::StorageKeyHolder
        })
        .unwrap(),
    )
    .bound_impl(
        quote!(::ink_storage::traits::StorageType<#salt_ident>),
        quote! { type Type = #ident #ty_generics; },
    )
}

fn auto_storage_type_derive(s: synstructure::Structure) -> TokenStream2 {
    let ident = s.ast().ident.clone();
    let salt_ident = s.ast().find_salt().unwrap().ident.to_token_stream();

    let (impl_generics, _, where_clause) = s.ast().generics.split_for_impl();
    let ty_generics: Vec<_> = s
        .ast()
        .generics
        .params
        .clone()
        .into_iter()
        .filter_map(|param| {
            let ident = match param {
                GenericParam::Type(t) => t.ident.to_token_stream(),
                GenericParam::Lifetime(l) => l.lifetime.to_token_stream(),
                GenericParam::Const(c) => c.ident.to_token_stream(),
            };
            if salt_ident.to_string() == ident.to_string() {
                Some(quote! {
                    ::ink_storage::traits::AutoKey
                })
            } else {
                Some(ident)
            }
        })
        .collect();

    quote! {
        impl #impl_generics ::ink_storage::traits::StorageType<#salt_ident> for #ident <#(#ty_generics),*> #where_clause {
            type Type = #ident <#(#ty_generics),*>;
        }
    }
}

fn manual_key_storage_type_derive(s: synstructure::Structure) -> TokenStream2 {
    let ident = s.ast().ident.clone();
    let salt_ident = s.ast().find_salt().unwrap().ident.to_token_stream();
    let manual_key_ident = format_ident!("__ink_generic_manual_key");
    let manual_salt_ident = format_ident!("__ink_generic_manual_salt");

    let ty_generics: Vec<_> = s
        .ast()
        .generics
        .params
        .clone()
        .into_iter()
        .filter_map(|param| {
            let ident = match param {
                GenericParam::Type(t) => t.ident.to_token_stream(),
                GenericParam::Lifetime(l) => l.lifetime.to_token_stream(),
                GenericParam::Const(c) => c.ident.to_token_stream(),
            };
            if salt_ident.to_string() == ident.to_string() {
                Some(quote! {
                    ::ink_storage::traits::ManualKey<#manual_key_ident, #manual_salt_ident>
                })
            } else {
                Some(ident)
            }
        })
        .collect();

    let mut generics = s.ast().generics.clone();
    generics.params.push(
        parse2(quote! { const #manual_key_ident : ::ink_primitives::StorageKey })
            .unwrap(),
    );
    generics.params.push(
        parse2(quote! { #manual_salt_ident : ::ink_storage::traits::StorageKeyHolder })
            .unwrap(),
    );
    let (impl_generics, _, where_clause) = generics.split_for_impl();

    quote! {
        impl #impl_generics ::ink_storage::traits::StorageType<#salt_ident> for #ident <#(#ty_generics),*> #where_clause {
            type Type = #ident <#(#ty_generics),*>;
        }
    }
}

pub fn storage_type_derive(s: synstructure::Structure) -> TokenStream2 {
    // If the generic salt is specified, then we add two implementations. One for `AutoKey`
    // and another for `ManualKey`.
    // - The implementation for `AutoKey` uses key and salt from the `StorageType` trait.
    // - The `ManualKey` ignores the `StorageType` trait and uses its values.
    if s.ast().find_salt().is_some() {
        let auto_key = auto_storage_type_derive(s.clone());
        let manual_key = manual_key_storage_type_derive(s);
        quote! {
            #auto_key
            #manual_key
        }
    } else {
        common_storage_type_derive(s)
    }
}

pub fn storage_type_derive2(mut s: synstructure::Structure) -> TokenStream2 {
    s.bind_with(|_| synstructure::BindStyle::Move)
        .add_bounds(synstructure::AddBounds::None)
        .underscore_const(true);

    let ident = s.ast().ident.clone();
    let inner_salt_ident = format_ident!("__ink_generic_salt");

    if s.ast().find_salt().is_some() {
        let salt_ident = s.ast().find_salt().unwrap().ident.to_token_stream();

        let (impl_generics, ty_generics_original, where_clause) =
            s.ast().generics.split_for_impl();
        let ty_generics: Vec<_> = s
            .ast()
            .generics
            .params
            .clone()
            .into_iter()
            .filter_map(|param| {
                let ident = match param {
                    GenericParam::Type(t) => t.ident.to_token_stream(),
                    GenericParam::Lifetime(l) => l.lifetime.to_token_stream(),
                    GenericParam::Const(c) => c.ident.to_token_stream(),
                };
                if salt_ident.to_string() == ident.to_string() {
                    Some(quote! {
                        #inner_salt_ident
                    })
                } else {
                    Some(ident)
                }
            })
            .collect();

        quote! {
            impl #impl_generics ::ink_storage::traits::StorageType2 for #ident #ty_generics_original #where_clause {
                type Type<#inner_salt_ident : ::ink_storage::traits::StorageKeyHolder> = #ident <#(#ty_generics),*>;
                type PreferredKey = #salt_ident;
            }
        }
    } else {
        let (_, ty_generics, _) = s.ast().generics.split_for_impl();

        s.gen_impl(quote! {
            gen impl ::ink_storage::traits::StorageType2 for @Self {
                type Type<#inner_salt_ident : ::ink_storage::traits::StorageKeyHolder> = #ident #ty_generics;
                type PreferredKey = ::ink_storage::traits::AutoKey;
            }
        })
    }
}
