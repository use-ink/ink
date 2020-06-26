// Copyright 2018-2020 Parity Technologies (UK) Ltd.
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

use super::{
    Constructor,
    Message,
};
use crate::{
    error::ExtError as _,
    ir2,
    ir2::attrs::Attrs as _,
};
use core::{
    convert::TryFrom,
    fmt,
};
use syn::spanned::Spanned as _;

/// The kind of externally callable smart contract entity.
pub(super) enum CallableKind {
    /// An ink! message externally callable.
    Message,
    /// An ink! constructor externally callable.
    Constructor,
}

impl fmt::Display for CallableKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Message => write!(f, "message"),
            Self::Constructor => write!(f, "constructor"),
        }
    }
}

/// Ensures that common invariants of externally callable ink! entities are met.
///
/// # Errors
///
/// In case any of the common externally callable invariants are not met:
/// - This is `true` if the externally callable is:
///  - generic
///  - const (compile-time evaluatable)
///  - async (async WebAssembly smart contract calling is not allowed)
///  - unsafe (caller provided assertions not yet stable)
/// - Furthermore this is `true` if the externally callable is defined for a
///   non default ABI (e.g. `extern "C"`) or doesn't have valid visibility.
pub(super) fn ensure_callable_invariants(
    method_item: &syn::ImplItemMethod,
    kind: CallableKind,
) -> Result<(), syn::Error> {
    if !matches!(method_item.vis, syn::Visibility::Public(_) | syn::Visibility::Inherited)
    {
        return Err(format_err!(
            method_item.vis,
            "ink! {}s must have public or inherited visibility",
            kind,
        ))
    }
    if !method_item.sig.generics.params.is_empty() {
        return Err(format_err!(
            method_item.sig.generics.params,
            "ink! {}s must not be generic",
            kind,
        ))
    }
    if method_item.sig.constness.is_some() {
        return Err(format_err!(
            method_item.sig.constness,
            "ink! {}s must not be const",
            kind,
        ))
    }
    if method_item.sig.asyncness.is_some() {
        return Err(format_err!(
            method_item.sig.asyncness,
            "ink! {}s must not be async",
            kind,
        ))
    }
    if method_item.sig.unsafety.is_some() {
        return Err(format_err!(
            method_item.sig.unsafety,
            "ink! {}s must not be unsafe",
            kind,
        ))
    }
    if method_item.sig.abi.is_some() {
        return Err(format_err!(
            method_item.sig.abi,
            "ink! {}s must have explicit ABI",
            kind,
        ))
    }
    if method_item.sig.variadic.is_some() {
        return Err(format_err!(
            method_item.sig.variadic,
            "ink! {}s must not be variadic",
            kind,
        ))
    }
    Ok(())
}

/// The visibility of an ink! message or constructor.
#[derive(Debug, Clone)]
pub enum Visibility {
    Public(syn::VisPublic),
    Inherited,
}

impl Visibility {
    /// Returns `true` if the visibility of the ink! message of constructor is public (`pub`).
    ///
    /// # Note
    ///
    /// Messages in normal implementation blocks must have public visibility.
    pub fn is_pub(&self) -> bool {
        matches!(self, Self::Public(_))
    }

    /// Returns `true` if the visibility of the ink! message of constructor is inherited.
    ///
    /// # Note
    ///
    /// Messages in trait implementation blocks must have inherited visibility.
    pub fn is_inherited(&self) -> bool {
        matches!(self, Self::Inherited)
    }
}
