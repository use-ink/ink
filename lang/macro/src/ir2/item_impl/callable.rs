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

//! Utilities and helper routines that are useful for both ink! messages
//! and ink! constructors.

use crate::ir2;
use core::fmt;
use proc_macro2::Ident;
use quote::ToTokens as _;

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

/// An ink! callable.
///
/// This is either an ink! message or an ink! constructor.
/// Used to share common behavior between different callable types.
pub trait Callable {
    /// Returns the identifier of the ink! callable.
    fn ident(&self) -> &Ident;

    /// Returns the selector of the ink! callable if any has been manually set.
    fn selector(&self) -> Option<&ir2::Selector>;

    /// Returns `true` if the ink! callable is flagged as payable.
    ///
    /// # Note
    ///
    /// Flagging as payable is done using the `#[ink(payable)]` attribute.
    fn is_payable(&self) -> bool;

    /// Returns the visibility of the ink! callable.
    fn visibility(&self) -> Visibility;

    /// Returns an iterator yielding all input parameters of the ink! callable.
    fn inputs(&self) -> InputsIter;
}

/// Returns the composed selector of the ink! callable.
///
/// Composition takes into account the given [`ir2::ItemImpl`].
///
/// # Details
///
/// Given
/// - the callable's identifier `i`
/// - the optionally set callable's selector `s`
/// - the impl blocks trait path in case it implements a trait, `P`
/// - the impl blocks optional user provided salt `S`
///
/// Then the selector is composed in the following way:
///
/// - If `s` is given we simply return `s`.
/// - Otherwise if `T` is not `None` (trait impl block) we concatenate
///   `S`, `T` and `i` with `::` as separator if `T` refers to a full-path.
///   If `T` refers to a relative path or is just an identifier we only take
///   its last segment `p` (e.g. the trait's identifier) into consideration
///   and use it instead of `P` in the above concatenation.
///   In the following we refer to the resulting concatenation as `C`.
/// - Now we take the BLAKE-2 hash of `C` which results in 32 bytes of output
///   and take the first 4 bytes that are returned in order as the composed
///   selector.
///
/// # Examples
///
/// ## Overriding the composed selector
///
/// Given
///
/// ```no_compile
/// impl MyStorage {
///     #[ink(message, selector = "0xDEADBEEF")]
///     fn my_message(&self) {}
/// }
/// ```
///
/// ... then the selector of `my_message` is simply `0xDEADBEEF` since it overrides
/// the composed selector.
///
/// ## Inherent implementation block
///
/// Given
///
/// ```no_compile
/// impl MyStorage {
///     #[ink(message)]
///     fn my_message(&self) {}
/// }
/// ```
///
/// ... then the selector of `my_message` is composed such as:
/// ```no_compile
/// BLAKE2("my_message".to_string().as_bytes())[0..4]
/// ```
///
/// ## Trait implementation block
///
/// Given
///
/// ```no_compile
/// impl MyTrait for MyStorage {
///     #[ink(message)]
///     fn my_message(&self) {}
/// }
/// ```
///
/// ... then the selector of `my_message` is composed such as:
/// ```no_compile
/// BLAKE2("MyTrait::my_message".to_string().as_bytes())[0..4]
/// ```
///
/// ## Using full path for trait
///
/// Given
///
/// ```no_compile
/// impl ::my_full::long_path::MyTrait for MyStorage {
///     #[ink(message)]
///     fn my_message(&self) {}
/// }
/// ```
///
/// ... then the selector of `my_message` is composed such as:
/// ```no_compile
/// BLAKE2("::my_full::long_path::MyTrait::my_message".to_string().as_bytes())[0..4]
/// ```
///
/// ## Using a salt
///
/// Given
///
/// ```no_compile
/// #[ink(salt = "my_salt")]
/// impl MyTrait for MyStorage {
///     #[ink(message)]
///     fn my_message(&self) {}
/// }
/// ```
///
/// ... then the selector of `my_message` is composed such as:
/// ```no_compile
/// BLAKE2("my_salt::MyTrait::my_message".to_string().as_bytes())[0..4]
/// ```
///
/// ## Note
///
/// All above examples work similarly for ink! constructors interchangeably.
///
/// ## Usage Recommendations
///
/// These recommendation mainly apply to trait implementation blocks:
///
/// - The recommandation by the ink! team is to use the full-path approach
/// wherever possible; OR import the trait and use only its identifier with
/// an additional salt if required to disambiguate selectors.
/// - Try not to intermix the above recommendations.
/// - Avoid directly setting the selector of an ink! message or constuctor.
///   Only do this if nothing else helps and you need a very specific selector,
///   e.g. in case of backwards compatibility.
/// - Do not use the salt unless required to disambiguate.
pub fn compose_selector<C>(item_impl: &ir2::ItemImpl, callable: &C) -> ir2::Selector
where
    C: Callable,
{
    if let Some(selector) = callable.selector() {
        return *selector
    }
    let callable_ident = callable.ident().to_string().into_bytes();
    let salt_bytes = item_impl
        .salt()
        .map(|salt| salt.as_bytes().to_vec())
        .unwrap_or(vec![]);
    let separator = &b"::"[..];
    let joined = match item_impl.trait_path() {
        None => {
            // Inherent implementation block:
            [salt_bytes, callable_ident].join(separator)
        }
        Some(path) => {
            // Trait implementation block:
            //
            // We need to separate between full-path, e.g. `::my::full::Path`
            // starting with `::` and relative paths for the composition.
            let path_bytes = if path.leading_colon.is_some() {
                path.to_token_stream().to_string().into_bytes()
            } else {
                path.get_ident()
                    .expect("encountered trait path without identifier")
                    .to_string()
                    .into_bytes()
            };
            [salt_bytes, path_bytes, callable_ident].join(separator)
        }
    };
    let hash = <blake2::Blake2b as blake2::Digest>::digest(&joined);
    ir2::Selector::new([hash[0], hash[1], hash[2], hash[3]])
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

/// Iterator over the input parameters of an ink! message or constructor.
///
/// Does not yield the self receiver of ink! messages.
pub struct InputsIter<'a> {
    iter: syn::punctuated::Iter<'a, syn::FnArg>,
}

impl<'a> From<&'a ir2::Message> for InputsIter<'a> {
    fn from(message: &'a ir2::Message) -> Self {
        Self {
            iter: message.item.sig.inputs.iter(),
        }
    }
}

impl<'a> From<&'a ir2::Constructor> for InputsIter<'a> {
    fn from(constructor: &'a ir2::Constructor) -> Self {
        Self {
            iter: constructor.item.sig.inputs.iter(),
        }
    }
}

impl<'a> Iterator for InputsIter<'a> {
    type Item = &'a syn::PatType;

    fn next(&mut self) -> Option<Self::Item> {
        'outer: loop {
            match self.iter.next() {
                None => return None,
                Some(syn::FnArg::Typed(pat_typed)) => return Some(pat_typed),
                Some(syn::FnArg::Receiver(_)) => continue 'outer,
            }
        }
    }
}
