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

fn storage_type_inner(s: synstructure::Structure) -> TokenStream2 {
    let ident = s.ast().ident.clone();
    let salt_ident = format_ident!("__ink_generic_salt");

    let mut generics = s.ast().generics.clone();
    generics.params.push(
        parse2(quote! { #salt_ident : ::ink_storage::traits::StorageKeyHolder }).unwrap(),
    );

    let (impl_generics, _, where_clause) = generics.split_for_impl();
    let (_, ty_generics_original, _) = s.ast().generics.split_for_impl();

    if s.ast().find_salt().is_some() {
        let inner_salt_ident = s.ast().find_salt().unwrap().ident.to_token_stream();
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
                if inner_salt_ident.to_string() == ident.to_string() {
                    Some(quote! {
                        #salt_ident
                    })
                } else {
                    Some(ident)
                }
            })
            .collect();

        quote! {
            impl #impl_generics ::ink_storage::traits::StorageType<#salt_ident> for #ident #ty_generics_original #where_clause {
                type Type = #ident <#(#ty_generics),*>;
                type PreferredKey = #inner_salt_ident;
            }
        }
    } else {
        quote! {
            impl #impl_generics ::ink_storage::traits::StorageType<#salt_ident> for #ident #ty_generics_original #where_clause {
                type Type = #ident #ty_generics_original;
                type PreferredKey = ::ink_storage::traits::AutoKey;
            }
        }
    }
}

pub fn storage_type_derive(s: synstructure::Structure) -> TokenStream2 {
    let derive = storage_type_inner(s);

    quote! {
        const _ : () = {
            #derive
        };
    }
}
