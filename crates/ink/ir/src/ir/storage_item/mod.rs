// Copyright (C) Use Ink (UK) Ltd.
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

mod config;

use crate::utils::find_storage_key_salt;
use config::StorageItemConfig;
use proc_macro2::TokenStream as TokenStream2;
use quote::{
    ToTokens,
    quote,
};
use std::collections::HashSet;

/// A checked ink! storage item with its configuration.
pub struct StorageItem {
    ast: syn::DeriveInput,
    config: StorageItemConfig,
}

impl StorageItem {
    /// Returns `Ok` if the input matches all requirements for an ink! storage item.
    pub fn new(config: TokenStream2, item: TokenStream2) -> Result<Self, syn::Error> {
        let ast = syn::parse2::<syn::DeriveInput>(item)?;
        let parsed_config = syn::parse2::<crate::ast::AttributeArgs>(config)?;
        let config = StorageItemConfig::try_from(parsed_config)?;

        for attr in &ast.attrs {
            if attr
                .path()
                .to_token_stream()
                .to_string()
                .contains("storage_item")
            {
                return Err(format_err_spanned!(
                    attr,
                    "only one `ink::storage_item` is allowed",
                ))
            }
        }

        Ok(Self { ast, config })
    }

    /// Returns AST.
    pub fn ast(&self) -> &syn::DeriveInput {
        &self.ast
    }

    /// Returns all types that were used in the storage declaration.
    pub fn all_used_types(&self) -> Vec<syn::Type> {
        let res: Vec<_> = match self.data().clone() {
            syn::Data::Struct(st) => {
                st.fields.iter().map(|field| field.ty.clone()).collect()
            }
            syn::Data::Enum(en) => {
                en.variants
                    .iter()
                    .flat_map(|variant| variant.fields.iter())
                    .map(|field| field.ty.clone())
                    .collect()
            }
            syn::Data::Union(un) => {
                un.fields
                    .named
                    .iter()
                    .map(|field| field.ty.clone())
                    .collect()
            }
        };
        let mut set = HashSet::new();
        res.into_iter()
            .filter(|ty| {
                if !set.contains(ty) {
                    set.insert(ty.clone());
                    true
                } else {
                    false
                }
            })
            .collect()
    }

    /// Returns the config of the storage.
    pub fn config(&self) -> &StorageItemConfig {
        &self.config
    }

    /// Returns the visibility of the storage.
    pub fn vis(&self) -> &syn::Visibility {
        &self.ast.vis
    }

    /// Returns the attributes of the storage.
    pub fn attrs(&self) -> &[syn::Attribute] {
        &self.ast.attrs
    }

    /// Returns the identifier of the storage.
    pub fn ident(&self) -> &syn::Ident {
        &self.ast.ident
    }

    /// Returns the generics of the storage.
    pub fn generics(&self) -> TokenStream2 {
        let types = self.ast.generics.clone();
        // `where_closure` is not included into `types`, so add it manually.
        let (_, _, where_closure) = self.ast.generics.split_for_impl();
        quote! {
            #types #where_closure
        }
    }

    /// Returns data of the storage.
    pub fn data(&self) -> &syn::Data {
        &self.ast.data
    }

    /// Returns salt for storage key.
    pub fn salt(&self) -> TokenStream2 {
        if let Some(param) = find_storage_key_salt(&self.ast) {
            param.ident.to_token_stream()
        } else {
            quote! { () }
        }
    }
}
