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

mod config;

use config::StorageItemConfig;
use ink_storage_codegen::DeriveUtils;
use proc_macro2::TokenStream as TokenStream2;
use quote::{
    quote,
    ToTokens,
};

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

        Ok(Self { ast, config })
    }

    /// Returns AST.
    pub fn ast(&self) -> &syn::DeriveInput {
        &self.ast
    }

    /// Returns all types that were used in the storage declaration.
    pub fn all_used_types(&self) -> Vec<syn::Type> {
        self.ast.all_types()
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
    pub fn generics(&self) -> &syn::Generics {
        &self.ast.generics
    }

    /// Returns data of the storage.
    pub fn data(&self) -> &syn::Data {
        &self.ast.data
    }

    /// Returns salt for storage key.
    pub fn salt(&self) -> TokenStream2 {
        if let Some(param) = self.ast.find_salt() {
            param.ident.to_token_stream()
        } else {
            quote! { () }
        }
    }
}
