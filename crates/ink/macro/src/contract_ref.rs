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

use ink_codegen::generate_code;
use ink_ir::{
    ast,
    format_err_spanned,
    utils::duplicate_config_err,
};
use ink_primitives::abi::Abi;
use proc_macro2::TokenStream as TokenStream2;
use quote::{
    format_ident,
    quote,
    quote_spanned,
};

pub fn analyze(config: TokenStream2, input: TokenStream2) -> TokenStream2 {
    match analyze_or_err(config, input) {
        Ok(tokens) => tokens,
        Err(err) => err.to_compile_error(),
    }
}

pub fn analyze_or_err(
    config: TokenStream2,
    input: TokenStream2,
) -> syn::Result<TokenStream2> {
    // Parses interface/contract ref config (if any).
    let config = Config::parse(config)?;
    // Re-uses trait definition IR and codegen.
    let trait_def = ink_ir::InkTraitDefinition::new(TokenStream2::new(), input)?;
    let trait_def_impl = generate_code((&trait_def, config.abi));

    let span = trait_def.item().span();
    let trait_name = trait_def.item().ident();
    let contract_ref_name = format_ident!("{trait_name}Ref");
    let abi_ty = match config.abi.unwrap_or({
        #[cfg(not(ink_abi = "sol"))]
        {
            Abi::Ink
        }

        #[cfg(ink_abi = "sol")]
        {
            Abi::Sol
        }
    }) {
        Abi::Ink => quote!(::ink::abi::Ink),
        Abi::Sol => quote!(::ink::abi::Sol),
    };
    let env = config
        .env
        .unwrap_or_else(|| syn::parse_quote! { ::ink::env::DefaultEnvironment });
    Ok(quote_spanned!(span =>
        // Trait def implementation.
        #trait_def_impl

        // Type alias for contract ref.
        pub type #contract_ref_name =
            <<::ink::reflect::TraitDefinitionRegistry<#env> as #trait_name>
                    ::__ink_TraitInfo as ::ink::codegen::TraitCallForwarder>::Forwarder<#abi_ty>;
    ))
}

/// The interface/contract ref configuration.
#[derive(Debug, PartialEq, Eq, Default)]
struct Config {
    /// The callee contract's ABI.
    abi: Option<Abi>,
    /// The environmental types definition.
    ///
    /// This must be a type that implements `ink_env::Environment` and can
    /// be used to change the underlying environmental types of an ink! smart
    /// contract.
    env: Option<syn::Path>,
}

impl Config {
    /// Parses contract ref config from token stream.
    fn parse(config: TokenStream2) -> syn::Result<Config> {
        let args = syn::parse2::<ast::AttributeArgs>(config)?;
        let mut abi_info: Option<(Abi, ast::MetaNameValue)> = None;
        let mut env_info: Option<(syn::Path, ast::MetaNameValue)> = None;
        for arg in args.into_iter() {
            if arg.name().is_ident("abi") {
                if let Some((_, ast)) = abi_info {
                    return Err(duplicate_config_err(ast, arg, "abi", "contract"));
                }
                let arg_info = arg
                    .name_value()
                    .zip(arg.value().and_then(ast::MetaValue::to_string));
                if let Some((name_value, abi_str)) = arg_info {
                    let abi = match abi_str.as_str() {
                        "ink" => Abi::Ink,
                        "sol" => Abi::Sol,
                        _ => {
                            return Err(format_err_spanned!(
                                arg,
                                "expected one of `ink` or `sol` for `abi` ink! configuration argument",
                            ))
                        }
                    };
                    abi_info = Some((abi, name_value.clone()));
                } else {
                    return Err(format_err_spanned!(
                        arg,
                        "expected a string literal value for `abi` ink! configuration argument",
                    ));
                }
            } else if arg.name().is_ident("env") {
                if let Some((_, ast)) = env_info {
                    return Err(duplicate_config_err(ast, arg, "env", "contract"));
                }
                let arg_info = arg
                    .name_value()
                    .zip(arg.value().and_then(ast::MetaValue::as_path));
                if let Some((name_value, path)) = arg_info {
                    env_info = Some((path.clone(), name_value.clone()))
                } else {
                    return Err(format_err_spanned!(
                        arg,
                        "expected a path value for `env` ink! configuration argument",
                    ));
                }
            } else {
                return Err(format_err_spanned!(
                    arg,
                    "encountered unknown or unsupported ink! configuration argument",
                ));
            }
        }
        Ok(Config {
            abi: abi_info.map(|(abi, _)| abi),
            env: env_info.map(|(path, _)| path),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Asserts that the given input configuration attribute argument are converted
    /// into the expected ink! configuration or yields the expected error message.
    fn assert_try_from(input: TokenStream2, expected: Result<Config, &'static str>) {
        assert_eq!(
            Config::parse(input).map_err(|err| err.to_string()),
            expected.map_err(ToString::to_string),
        );
    }

    #[test]
    fn empty_config_works() {
        assert_try_from(quote! {}, Ok(Config::default()))
    }

    #[test]
    fn abi_works() {
        assert_try_from(
            quote! {
                abi = "ink"
            },
            Ok(Config {
                abi: Some(Abi::Ink),
                env: None,
            }),
        );
        assert_try_from(
            quote! {
                abi = "sol"
            },
            Ok(Config {
                abi: Some(Abi::Sol),
                env: None,
            }),
        );
    }

    #[test]
    fn abi_invalid_value_fails() {
        assert_try_from(
            quote! { abi = "move" },
            Err("expected one of `ink` or `sol` for `abi` ink! configuration argument"),
        );
        assert_try_from(
            quote! { abi = 1u8 },
            Err("expected a string literal value for `abi` ink! configuration argument"),
        );
    }

    #[test]
    fn abi_missing_value_fails() {
        assert_try_from(
            syn::parse_quote! { abi },
            Err("expected a string literal value for `abi` ink! configuration argument"),
        );
    }

    #[test]
    fn env_works() {
        assert_try_from(
            quote! {
                env = ::my::env::Types
            },
            Ok(Config {
                abi: None,
                env: Some(syn::parse_quote! { ::my::env::Types }),
            }),
        )
    }

    #[test]
    fn env_invalid_value_fails() {
        assert_try_from(
            quote! { env = "invalid" },
            Err("expected a path value for `env` ink! configuration argument"),
        );
    }

    #[test]
    fn env_missing_value_fails() {
        assert_try_from(
            quote! { env },
            Err("expected a path value for `env` ink! configuration argument"),
        );
    }

    #[test]
    fn unknown_arg_fails() {
        assert_try_from(
            quote! { unknown = argument },
            Err("encountered unknown or unsupported ink! configuration argument"),
        );
    }

    #[test]
    fn duplicate_args_fails() {
        assert_try_from(
            quote! {
                abi = "ink",
                abi = "sol",
            },
            Err("encountered duplicate ink! contract `abi` configuration argument"),
        );
        assert_try_from(
            quote! {
                env = ::my::env::Types,
                env = ::my::other::env::Types,
            },
            Err("encountered duplicate ink! contract `env` configuration argument"),
        );
    }
}
