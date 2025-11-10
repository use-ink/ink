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

use syn::spanned::Spanned;

use crate::{
    ast,
    utils::duplicate_config_err,
};

/// The ink! storage item configuration.
#[derive(Debug, PartialEq, Eq)]
pub struct StorageItemConfig {
    /// If set to `true`, the derived storage item will use a "packed" layout.
    /// If set to `false`, the derived storage item will use a "non-packed" layout,
    /// this is the default value.
    packed: bool,
    /// If set to `true`, all storage related traits are implemented automatically,
    /// this is the default value.
    /// If set to `false`, implementing all storage traits is disabled. In some cases
    /// this can be helpful to override the default implementation of the trait.
    derive: bool,
}

impl Default for StorageItemConfig {
    fn default() -> Self {
        Self {
            packed: false,
            derive: true,
        }
    }
}

impl TryFrom<ast::AttributeArgs> for StorageItemConfig {
    type Error = syn::Error;

    fn try_from(args: ast::AttributeArgs) -> Result<Self, Self::Error> {
        let mut packed: Option<syn::Path> = None;
        let mut derive: Option<syn::LitBool> = None;
        let args_span = args.span();
        for arg in args {
            if arg.name().is_ident("packed") {
                if let Some(path) = packed {
                    return Err(duplicate_config_err(path, arg, "packed", "storage item"));
                }
                if let ast::Meta::Path(path) = arg {
                    packed = Some(path)
                } else {
                    return Err(format_err_spanned!(
                        arg,
                        "encountered an unexpected value for `packed` ink! storage item configuration argument. \
                        Did you mean `#[ink::storage_item(packed)]` ?",
                    ));
                }
            } else if arg.name().is_ident("derive") {
                if let Some(lit_bool) = derive {
                    return Err(duplicate_config_err(
                        lit_bool,
                        arg,
                        "derive",
                        "storage item",
                    ));
                }
                if let Some(lit_bool) = arg.value().and_then(ast::MetaValue::as_lit_bool)
                {
                    derive = Some(lit_bool.clone())
                } else {
                    return Err(format_err_spanned!(
                        arg,
                        "expected a bool literal value for `derive` ink! storage item configuration argument",
                    ));
                }
            } else {
                return Err(format_err_spanned!(
                    arg,
                    "encountered unknown or unsupported ink! storage item configuration argument",
                ));
            }
        }

        // Sanitize user-provided configuration.
        let (packed, derive) = match (packed, derive.map(|lit_bool| lit_bool.value)) {
            // `packed` (i.e. `packed=true`) and `derive=false` conflict.
            // Note: There's really no reasonable use case for this combination.
            (Some(_), Some(false)) => {
                return Err(format_err!(
                    args_span,
                    "cannot use `derive = false` with `packed` flag",
                ))
            }
            // Otherwise, accept the user provided configuration,
            // while defaulting to "non-packed" layout (resolved as `packed=false`) and
            // `derive=true`.
            (packed, derive) => (packed.is_some(), derive.unwrap_or(true)),
        };

        Ok(StorageItemConfig { packed, derive })
    }
}

impl StorageItemConfig {
    /// Returns the `packed` configuration argument.
    pub fn packed(&self) -> bool {
        self.packed
    }

    /// Returns the `derive` configuration argument.
    pub fn derive(&self) -> bool {
        self.derive
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;

    #[test]
    fn valid_args_works() {
        for (config, packed, derive) in [
            // Defaults are "non-packed" layout (resolved as `packed=false`) with
            // `derive=true`.
            (quote!(), false, true),
            // `packed` (resolved as `packed=true`) works only with `derive=true`.
            (quote! { packed }, true, true),
            (quote! { packed, derive = true }, true, true),
            // "non-packed" layout (resolved as `packed=false`) works with any `derive`
            // arg.
            (quote! { derive = true }, false, true),
            (quote! { derive = false }, false, false),
        ] {
            let parsed_config = syn::parse2::<crate::ast::AttributeArgs>(config).unwrap();
            let result = StorageItemConfig::try_from(parsed_config);
            assert!(result.is_ok());
            let storage_item_config = result.unwrap();
            assert_eq!(storage_item_config.packed(), packed);
            assert_eq!(storage_item_config.derive(), derive);
        }
    }

    #[test]
    #[should_panic = "cannot use `derive = false` with `packed` flag"]
    fn conflicting_args_fails() {
        let config = quote! {
            // `packed` and `derive = false` conflict.
            packed, derive = false
        };
        let parsed_config = syn::parse2::<crate::ast::AttributeArgs>(config).unwrap();
        StorageItemConfig::try_from(parsed_config).unwrap();
    }

    #[test]
    #[should_panic = "encountered an unexpected value for `packed` ink! storage item configuration argument. Did you mean `#[ink::storage_item(packed)]` ?"]
    fn invalid_packed_value_fails() {
        let config = quote! {
            // `packed` arg doesn't accept a value.
            packed = true
        };
        let parsed_config = syn::parse2::<crate::ast::AttributeArgs>(config).unwrap();
        StorageItemConfig::try_from(parsed_config).unwrap();
    }
}
