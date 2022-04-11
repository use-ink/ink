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
use quote::{
    quote,
    ToTokens,
};

/// TODO: Add comment
pub struct StorageItem {
    ast: syn::ItemStruct,
}

impl StorageItem {
    /// TODO: Add comment
    pub fn new(_: TokenStream2, item: TokenStream2) -> Result<Self, syn::Error> {
        let ast = syn::parse2::<syn::ItemStruct>(item)?;
        Ok(Self { ast })
    }

    /// Returns the ast of the storage struct
    pub fn ast(&self) -> &syn::ItemStruct {
        &self.ast
    }

    /// Returns the visibility of the storage struct.
    pub fn vis(&self) -> &syn::Visibility {
        &self.ast.vis
    }

    /// Returns the attributes of the storage struct.
    pub fn attrs(&self) -> &[syn::Attribute] {
        &self.ast.attrs
    }

    /// Returns the identifier of the storage struct.
    pub fn ident(&self) -> &syn::Ident {
        &self.ast.ident
    }

    /// Returns the generics of the storage struct.
    pub fn generics(&self) -> &syn::Generics {
        &self.ast.generics
    }

    /// Returns an iterator yielding all fields of the storage struct.
    pub fn fields(&self) -> syn::punctuated::Iter<syn::Field> {
        self.ast.fields.iter()
    }

    /// Returns true if the generic of the struct contains salt for storage key specified
    /// by the user.
    ///
    /// ```no_compile
    /// struct<Salt: ::ink_storage::traits::StorageKeyHolder> SomeStruct;
    /// ```
    pub fn has_specified_salt(&self) -> bool {
        self.find_salt().is_some()
    }

    /// Returns salt for storage key
    pub fn salt(&self) -> TokenStream2 {
        if let Some(param) = self.find_salt() {
            param.ident.to_token_stream()
        } else {
            quote! { () }
        }
    }

    /// Returns salt ident
    pub fn salt_ident(&self) -> Option<syn::Ident> {
        if let Some(param) = self.find_salt() {
            Some(param.ident)
        } else {
            None
        }
    }

    fn find_salt(&self) -> Option<syn::TypeParam> {
        self.generics().params.iter().find_map(|param| {
            if let syn::GenericParam::Type(type_param) = param {
                if type_param.bounds.len() == 1 {
                    let bound = type_param.bounds.first().unwrap();
                    if let syn::TypeParamBound::Trait(trait_bound) = bound {
                        let segments = &trait_bound.path.segments;
                        if !segments.is_empty()
                            && segments.last().unwrap().ident.to_string()
                                == "StorageKeyHolder"
                        {
                            return Some(type_param.clone())
                        }
                    }
                }
            }
            None
        })
    }
}
