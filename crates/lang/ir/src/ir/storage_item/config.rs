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

use crate::{
    ast,
    error::ExtError as _,
};
use syn::spanned::Spanned;

/// The ink! configuration.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct StorageItemConfig {
    /// If set to `true`, all storage related traits are implemented automatically,
    /// this is the default value.
    /// If set to `false`, implementing all storage traits is disabled. In some cases
    /// this can be helpful to override the default implementation of the trait.
    derive: bool,
}

/// Return an error to notify about duplicate ink! ink storage configuration arguments.
fn duplicate_config_err<F, S>(fst: F, snd: S, name: &str) -> syn::Error
where
    F: Spanned,
    S: Spanned,
{
    format_err!(
        snd.span(),
        "encountered duplicate ink! storage item `{}` configuration argument",
        name,
    )
    .into_combine(format_err!(
        fst.span(),
        "first `{}` configuration argument here",
        name
    ))
}

impl TryFrom<ast::AttributeArgs> for StorageItemConfig {
    type Error = syn::Error;

    fn try_from(args: ast::AttributeArgs) -> Result<Self, Self::Error> {
        let mut derive: Option<syn::LitBool> = None;
        for arg in args.into_iter() {
            if arg.name.is_ident("derive") {
                if let Some(lit_bool) = derive {
                    return Err(duplicate_config_err(lit_bool, arg, "derive"))
                }
                if let ast::PathOrLit::Lit(syn::Lit::Bool(lit_bool)) = &arg.value {
                    derive = Some(lit_bool.clone())
                } else {
                    return Err(format_err_spanned!(
                        arg,
                        "expected a bool literal for `derive` ink! storage item configuration argument",
                    ))
                }
            } else {
                return Err(format_err_spanned!(
                    arg,
                    "encountered unknown or unsupported ink! storage item configuration argument",
                ))
            }
        }
        Ok(StorageItemConfig {
            derive: derive.map(|lit_bool| lit_bool.value).unwrap_or(true),
        })
    }
}

impl StorageItemConfig {
    /// Returns the derive configuration argument.
    pub fn derive(&self) -> bool {
        self.derive
    }
}
