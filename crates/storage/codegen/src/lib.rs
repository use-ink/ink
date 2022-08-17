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

use std::collections::HashSet;
use syn::Data;

/// Provides common methods for `DeriveInput`.
///
/// # Developer Note
///
/// This is only for internal usage in the `codegen` module.
pub trait DeriveUtils {
    /// Finds the salt of the structure, enum or union.
    /// The salt is any generic that has bound `StorageKey`.
    /// In most cases it is parent storage key or auto-generated storage key.
    fn find_salt(&self) -> Option<syn::TypeParam>;

    /// Return all types of the input.
    fn all_types(&self) -> Vec<syn::Type>;
}

impl DeriveUtils for syn::DeriveInput {
    fn find_salt(&self) -> Option<syn::TypeParam> {
        self.generics.params.iter().find_map(|param| {
            if let syn::GenericParam::Type(type_param) = param {
                if let Some(bound) = type_param.bounds.first() {
                    if let syn::TypeParamBound::Trait(trait_bound) = bound {
                        let segments = &trait_bound.path.segments;
                        if let Some(last) = segments.last() {
                            if last.ident == "StorageKey" {
                                return Some(type_param.clone())
                            }
                        }
                    }
                }
            }
            None
        })
    }

    fn all_types(&self) -> Vec<syn::Type> {
        let res: Vec<_> = match self.data.clone() {
            Data::Struct(st) => st.fields.iter().map(|field| field.ty.clone()).collect(),
            Data::Enum(en) => {
                en.variants
                    .iter()
                    .flat_map(|variant| variant.fields.iter())
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
}
