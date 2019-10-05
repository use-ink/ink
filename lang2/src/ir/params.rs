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

use crate::ir::{
    MetaVersion,
};
use core::convert::TryFrom;
use derive_more::From;
use proc_macro2::{
    Ident,
    Span,
};
use regex::Regex;
use syn::{
    parse::{
        Parse,
        ParseStream,
    },
    punctuated::Punctuated,
    spanned::Spanned,
    LitStr,
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
    /// The input literal string.
    pub value: LitStr,
    /// The decoded major, minor and patch version.
    pub data: MetaVersion,
}

impl ParamVersion {
    /// Returns the span of `self`.
    pub fn span(&self) -> Span {
        self.version
            .span()
            .join(self.value.span())
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

impl<'a> TryFrom<&'a str> for MetaVersion {
    type Error = regex::Error;

    fn try_from(content: &'a str) -> core::result::Result<Self, Self::Error> {
        let re = Regex::new(
            r"(?x)
            ^(?P<major>0|[1-9]\d*) # major version
            \.
            (?P<minor>0|[1-9]\d*)  # minor version
            \.
            (?P<patch>0|[1-9]\d*)  # patch version

            (?:-
                (?P<prerelease>(?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*)
                (?:\.(?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*))
            *))?

            (?:\+(?P<buildmetadata>[0-9a-zA-Z-]+(?:\.[0-9a-zA-Z-]+)*))?$
        ",
        )
        .unwrap();
        let caps = re.captures(content).ok_or(regex::Error::Syntax(
            "couldn't properly match against semantic version".into(),
        ))?;
        let major = caps["major"]
            .parse::<usize>()
            .expect("major version parsing cannot fail since guaranteed by regex; qed");
        let minor = caps["minor"]
            .parse::<usize>()
            .expect("minor version parsing cannot fail since guaranteed by regex; qed");
        let patch = caps["patch"]
            .parse::<usize>()
            .expect("patch version parsing cannot fail since guaranteed by regex; qed");
        Ok(Self {
            major,
            minor,
            patch,
        })
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
        let value: LitStr = input.parse()?;
        let content: &str = &value.value();
        let data = MetaVersion::try_from(content).map_err(|_| {
            format_err_span!(
                value.span(),
                "couldn't match provided version as semantic version string: {}",
                content,
            )
        })?;
        Ok(Self {
            version: version_ident,
            eq_token,
            value,
            data,
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
