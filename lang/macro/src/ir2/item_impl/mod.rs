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

use crate::{
    error::ExtError as _,
    ir2,
    ir2::attrs::Attrs as _,
};
use core::convert::TryFrom;
use proc_macro2::Span;
use syn::spanned::Spanned as _;

mod callable;
mod constructor;
mod impl_item;
mod iter;
mod message;

#[cfg(test)]
mod tests;

use self::callable::{
    ensure_callable_invariants,
    Callable,
    CallableKind,
    InputsIter,
};
pub use self::{
    callable::Visibility,
    constructor::Constructor,
    impl_item::ImplItem,
    iter::{
        IterConstructors,
        IterMessages,
    },
    message::{
        Message,
        Receiver,
    },
};

/// An ink! implementation block.
///
/// # Note
///
/// - This can be either an inherent implementation block that implements some
///   constructors, messages or internal functions for the storage struct; OR it
///   can be a trait implementation for the storage struct.
/// - We try to support all fields that are supported by the underlying `syn`
///   implementation for [`syn::ItemImpl`] even though they are not really
///   required to represent ink!. This is done for consistency with `syn`.
#[derive(Debug, PartialEq, Eq)]
pub struct ItemImpl {
    attrs: Vec<syn::Attribute>,
    defaultness: Option<syn::token::Default>,
    unsafety: Option<syn::token::Unsafe>,
    impl_token: syn::token::Impl,
    generics: syn::Generics,
    trait_: Option<(Option<syn::token::Bang>, syn::Path, syn::token::For)>,
    self_ty: Box<syn::Type>,
    brace_token: syn::token::Brace,
    items: Vec<ImplItem>,
    /// A salt prefix to disambiguate trait implementation blocks with equal
    /// names. Generally can be used to change computation of message and
    /// constructor selectors of the implementation block.
    salt: Option<ir2::Salt>,
}

impl ItemImpl {
    /// Returns `true` if the Rust implementation block is an ink! implementation
    /// block.
    ///
    /// # Note
    ///
    /// This is the case if:
    ///
    /// - The ink! implementation block has been annotatated as in:
    ///
    /// ```no_compile
    /// #[ink(impl)]
    /// impl MyStorage {
    ///     fn my_function(&self) { /* ... */ }
    /// }
    /// ```
    ///
    /// - Or if any of the ink! implementation block methods do have ink!
    ///   specific annotations:
    ///
    /// ```no_compile
    /// impl MyStorage {
    ///     #[ink(constructor)]
    ///     fn my_constructor() -> Self { /* ... */ }
    /// }
    /// ```
    ///
    /// The same rules apply to ink! trait implementation blocks.
    ///
    /// # Errors
    ///
    /// Returns an error in case of encountered malformed ink! attributes.
    pub fn is_ink_impl_block(item_impl: &syn::ItemImpl) -> Result<bool, syn::Error> {
        // Quick check in order to efficiently bail out in case where there are
        // no ink! attributes:
        if !ir2::contains_ink_attributes(&item_impl.attrs)
            && item_impl
                .items
                .iter()
                .all(|item| !ir2::contains_ink_attributes(item.attrs()))
        {
            return Ok(false)
        }
        // Check if the implementation block itself has been annotated with
        // `#[ink(impl)]` and return `true` if this is the case.
        let (ink_attrs, _) = ir2::partition_attributes(item_impl.attrs.clone())?;
        let impl_block_span = item_impl.span();
        if !ink_attrs.is_empty() {
            let normalized =
                ir2::InkAttribute::from_expanded(ink_attrs).map_err(|err| {
                    err.into_combine(format_err_span!(
                        impl_block_span,
                        "at this invokation",
                    ))
                })?;
            if normalized
                .ensure_first(&ir2::AttributeArgKind::Implementation)
                .is_ok()
            {
                return Ok(true)
            }
        }
        // Check if any of the implementation block's methods either resembles
        // an ink! constructor or an ink! message:
        'outer: for item in &item_impl.items {
            match item {
                syn::ImplItem::Method(method_item) => {
                    if !ir2::contains_ink_attributes(&method_item.attrs) {
                        continue 'outer
                    }
                    let attr = ir2::first_ink_attribute(&method_item.attrs)?
                        .expect("missing expected ink! attribute for struct");
                    match attr.first().kind() {
                        ir2::AttributeArgKind::Constructor
                        | ir2::AttributeArgKind::Message => return Ok(true),
                        _ => continue 'outer,
                    }
                }
                _ => continue 'outer,
            }
        }
        Ok(false)
    }
}

impl TryFrom<syn::ItemImpl> for ItemImpl {
    type Error = syn::Error;

    fn try_from(item_impl: syn::ItemImpl) -> Result<Self, Self::Error> {
        let impl_block_span = item_impl.span();
        if !Self::is_ink_impl_block(&item_impl)? {
            return Err(format_err!(
                item_impl,
                "missing ink! annotations on the impl block or on any of its items"
            ))
        }
        if let Some(defaultness) = item_impl.defaultness {
            return Err(format_err!(
                defaultness,
                "default implementations are unsupported for ink! implementation blocks",
            ))
        }
        if let Some(unsafety) = item_impl.unsafety {
            return Err(format_err!(
                unsafety,
                "unsafe ink! implementation blocks are not supported",
            ))
        }
        if !item_impl.generics.params.is_empty() {
            return Err(format_err!(
                item_impl.generics.params,
                "generic ink! implementation blocks are not supported",
            ))
        }
        let impl_items = item_impl
            .items
            .into_iter()
            .map(<ImplItem as TryFrom<_>>::try_from)
            .collect::<Result<Vec<_>, syn::Error>>()?;
        let is_trait_impl = item_impl.trait_.is_some();
        for impl_item in &impl_items {
            /// Ensures that visibility of ink! messages and constructors is
            /// valid in dependency of the containing ink! impl block.
            ///
            /// # Note
            ///
            /// Trait implementation blocks expect inherited visibility
            /// while inherent implementation block expect public visibility.
            fn ensure_valid_visibility(
                vis: ir2::Visibility,
                span: Span,
                what: &str,
                is_trait_impl: bool,
            ) -> Result<(), syn::Error> {
                let requires_pub = !is_trait_impl;
                if requires_pub != vis.is_pub() {
                    return Err(format_err_span!(
                        span,
                        "ink! {} in {} impl blocks must have {} visibility",
                        what,
                        if is_trait_impl { "trait" } else { "inherent" },
                        if requires_pub { "public" } else { "inherited" },
                    ))
                }
                Ok(())
            }
            match impl_item {
                ir2::ImplItem::Message(message) => {
                    ensure_valid_visibility(
                        message.visibility(),
                        message.item.span(),
                        "message",
                        is_trait_impl,
                    )?;
                }
                ir2::ImplItem::Constructor(constructor) => {
                    ensure_valid_visibility(
                        constructor.visibility(),
                        constructor.item.span(),
                        "constructor",
                        is_trait_impl,
                    )?;
                }
                _ => (),
            }
        }
        let (ink_attrs, other_attrs) = ir2::partition_attributes(item_impl.attrs)?;
        let mut salt = None;
        if !ink_attrs.is_empty() {
            let normalized =
                ir2::InkAttribute::from_expanded(ink_attrs).map_err(|err| {
                    err.into_combine(format_err_span!(
                        impl_block_span,
                        "at this invokation",
                    ))
                })?;
            normalized.ensure_no_conflicts(|arg| {
                match arg.kind() {
                    ir2::AttributeArgKind::Implementation
                    | ir2::AttributeArgKind::Salt(_) => false,
                    _ => true,
                }
            })?;
            salt = normalized.salt();
        }
        Ok(Self {
            attrs: other_attrs,
            defaultness: item_impl.defaultness,
            unsafety: item_impl.unsafety,
            impl_token: item_impl.impl_token,
            generics: item_impl.generics,
            trait_: item_impl.trait_,
            self_ty: item_impl.self_ty,
            brace_token: item_impl.brace_token,
            items: impl_items,
            salt,
        })
    }
}

impl ItemImpl {
    /// Retursn the `Self` type of the implementation block.
    pub fn self_type(&self) -> &syn::Type {
        self.self_ty.as_ref()
    }

    /// Returns the trait type path if this is a trait implementation block.
    ///
    /// Returns `None` if this is an inherent implementation block.
    pub fn trait_path(&self) -> Option<&syn::Path> {
        self.trait_.as_ref().map(|(_, path, _)| path)
    }

    /// Returns the salt of the implementation block if any has been provided.
    pub fn salt(&self) -> Option<&ir2::Salt> {
        self.salt.as_ref()
    }

    /// Returns an iterator yielding the ink! messages of the implementation block.
    pub fn iter_messages(&self) -> IterMessages {
        IterMessages::new(self)
    }

    /// Returns an iterator yielding the ink! messages of the implementation block.
    pub fn iter_constructors(&self) -> IterConstructors {
        IterConstructors::new(self)
    }
}
