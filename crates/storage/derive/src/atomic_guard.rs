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
use quote::quote;
use syn::parse2;

pub fn atomic_guard_derive(s: synstructure::Structure) -> TokenStream2 {
    let ident = &s.ast().ident;

    let mut generics = s.ast().generics.clone();
    let mut where_clause = generics.where_clause.clone().unwrap_or(syn::WhereClause {
        where_token: Default::default(),
        predicates: Default::default(),
    });

    s.ast().all_types().iter().for_each(|ty| {
        where_clause.predicates.push(
            parse2(quote! { #ty : ::ink_storage::traits::AtomicGuard<true> }).unwrap(),
        );
    });
    if !where_clause.predicates.is_empty() {
        generics.where_clause = Some(where_clause);
    }

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    quote! {
        const _ : () = {
            impl #impl_generics ::ink_storage::traits::AtomicGuard<true>
                for #ident #ty_generics #where_clause {}
        };
    }
}
