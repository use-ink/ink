// Copyright 2018-2019 Parity Technologies (UK) Ltd.
// This file is part of ink!.
//
// ink! is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// ink! is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with ink!.  If not, see <http://www.gnu.org/licenses/>.

//! Contains data structures and parsing routines for parameters to the ink! macro.

use crate::ir::UnsuffixedLitInt;
use derive_more::From;
use proc_macro2::{
    Ident,
    Span,
};
use syn::{
    parse::{
        Parse,
        ParseStream,
    },
    punctuated::Punctuated,
    spanned::Spanned,
    Result,
    Token,
};

/// Parameters given to ink!'s `#[contract(..)]` attribute.
///
/// # Example
///
/// ```no_compile
/// #[ink::contract(env = DefaultSrmlTypes, version = 0.1.0)]
/// ```
pub struct Params {
    /// The delimited meta information parameters.
    pub params: Punctuated<MetaParam, Token![,]>,
}

impl Spanned for Params {
    fn span(&self) -> Span {
        if self.params.len() == 0 {
            Span::call_site()
        } else {
            self.params
                .first()
                .unwrap()
                .span()
                .join(self.params.last().unwrap().span())
                .expect("params in `self` must be within the same file; qed")
        }
    }
}

/// A specialized ink! contract meta information.
///
/// This information is usually given at the contract definition via attribute parameters.
///
/// # Example
///
/// ```no_compile
/// #[ink::contract(
///     env = DefaultSrmlTypes, // The used chain types.
///     version = 0.1.0,        // The used ink! version.
/// )]
/// mod my_contract { ... }
/// ```
///
/// # Note
///
/// Even though ink! could define some defaults for this meta information we currently
/// require contracts to specify them and may relax this in the future.
#[derive(Debug, Clone, From)]
pub enum MetaParam {
    /// Environmental types definition: `#[ink(env = DefaultSrmlTypes)]`
    Types(ParamTypes),
    /// Information about the ink! version: `#[ink(version = x.y.z)]`
    Version(ParamVersion),
}

impl MetaParam {
    /// Returns the identifier of the meta information.
    ///
    /// # Examples
    ///
    /// - for `types = DefaultSrmlTypes` this is `types`
    /// - for `version = [0, 1, 0]` this is `version`
    pub fn ident(&self) -> &Ident {
        match self {
            MetaParam::Types(meta_env) => &meta_env.env,
            MetaParam::Version(version) => &version.version,
        }
    }
}

impl Spanned for MetaParam {
    fn span(&self) -> Span {
        match self {
            MetaParam::Types(param_types) => param_types.span(),
            MetaParam::Version(param_version) => param_version.span(),
        }
    }
}

/// The environment types definition: `#[ink(env = DefaultSrmlTypes)]`
#[derive(Debug, Clone)]
pub struct ParamTypes {
    /// The `env` identifier.
    pub env: Ident,
    /// The `=` token.
    pub eq_token: Token![=],
    /// The environmental types type.
    pub ty: syn::Type,
}

impl ParamTypes {
    /// Returns the span of `self`.
    pub fn span(&self) -> Span {
        self.env
            .span()
            .join(self.ty.span())
            .expect("both spans are in the same file AND we are using nightly Rust; qed")
    }
}

/// The used ink! version: `#[ink(version = 0.1.0)]`
#[derive(Debug, Clone)]
pub struct ParamVersion {
    /// The `version` identifier.
    pub version: Ident,
    /// The `=` token.
    pub eq_token: Token![=],
    /// The `[` and `]` surrounding the actual version information.
    pub bracket_token: syn::token::Bracket,
    /// The version information.
    pub parts: Punctuated<UnsuffixedLitInt, Token![,]>,
}

impl ParamVersion {
    /// Returns the span of `self`.
    pub fn span(&self) -> Span {
        self.version
            .span()
            .join(self.bracket_token.span)
            .expect("both spans are in the same file AND we are using nightly Rust; qed")
    }
}

impl Parse for Params {
    fn parse(input: ParseStream) -> Result<Self> {
        let params = Punctuated::parse_terminated(input)?;
        Ok(Self { params })
    }
}

impl Parse for MetaParam {
    fn parse(input: ParseStream) -> Result<Self> {
        let ident = input.fork().parse::<Ident>()?;
        match ident.to_string().as_str() {
            "version" => input.parse::<ParamVersion>().map(Into::into),
            "env" => input.parse::<ParamTypes>().map(Into::into),
            unknown => {
                Err(format_err_span!(
                    ident.span(),
                    "unknown ink! meta information: {}",
                    unknown
                ))
            }
        }
    }
}

impl Parse for ParamVersion {
    fn parse(input: ParseStream) -> Result<Self> {
        let version_ident = input.parse()?;
        if version_ident != "version" {
            bail!(
                version_ident,
                "invalid identifier for meta version information",
            )
        }
        let eq_token = input.parse()?;
        let content;
        let bracket_token = syn::bracketed!(content in input);
        let parts = Punctuated::parse_terminated(&content)?;
        if parts.len() != 3 {
            bail_span!(bracket_token.span, "expected 3 elements in version array",)
        }
        Ok(Self {
            version: version_ident,
            eq_token,
            bracket_token,
            parts,
        })
    }
}

impl Parse for ParamTypes {
    fn parse(input: ParseStream) -> Result<Self> {
        let env_ident = input.parse()?;
        if env_ident != "env" {
            bail!(
                env_ident,
                "invalid identifier for meta environment information",
            )
        }
        let eq_token = input.parse()?;
        let ty = input.parse()?;
        Ok(Self {
            env: env_ident,
            eq_token,
            ty,
        })
    }
}
