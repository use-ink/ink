// Copyright 2018-2021 Parity Technologies (UK) Ltd.
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

mod iter;
mod trait_item;

use self::iter::IterInkTraitItemsRaw;
pub use self::{
    iter::IterInkTraitItems,
    trait_item::{
        InkTraitItem,
        InkTraitMessage,
    },
};
use super::TraitDefinitionConfig;
use crate::{
    ir,
    ir::idents_lint,
    Selector,
};
#[cfg(test)]
use core::convert::TryFrom;
use ir::TraitPrefix;
use proc_macro2::{
    Ident,
    Span,
};
use std::collections::HashMap;
use syn::{
    spanned::Spanned as _,
    Result,
};

/// A checked ink! trait definition without its configuration.
#[derive(Debug, PartialEq, Eq)]
pub struct InkItemTrait {
    item: syn::ItemTrait,
    message_selectors: HashMap<syn::Ident, Selector>,
}

#[cfg(test)]
impl TryFrom<syn::ItemTrait> for InkItemTrait {
    type Error = syn::Error;

    fn try_from(item_trait: syn::ItemTrait) -> core::result::Result<Self, Self::Error> {
        let config = TraitDefinitionConfig::default();
        Self::new(&config, item_trait)
    }
}

impl InkItemTrait {
    /// Creates a new ink! item trait from the given configuration and trait
    /// definition.
    pub fn new(
        config: &TraitDefinitionConfig,
        item_trait: syn::ItemTrait,
    ) -> Result<Self> {
        idents_lint::ensure_no_ink_identifiers(&item_trait)?;
        Self::analyse_properties(&item_trait)?;
        Self::analyse_items(&item_trait)?;
        let mut message_selectors = <HashMap<syn::Ident, Selector>>::new();
        Self::extract_selectors(config, &item_trait, &mut message_selectors)?;
        if message_selectors.is_empty() {
            return Err(format_err!(
                item_trait.span(),
                "encountered invalid empty ink! trait definition"
            ))
        }
        Ok(Self {
            item: item_trait,
            message_selectors,
        })
    }
}

impl InkItemTrait {
    /// Returns span of the ink! trait definition.
    pub fn span(&self) -> Span {
        self.item.span()
    }

    /// Returns the attributes of the ink! trait definition.
    pub fn attrs(&self) -> &[syn::Attribute] {
        &self.item.attrs
    }

    /// Returns the identifier of the ink! trait definition.
    pub fn ident(&self) -> &Ident {
        &self.item.ident
    }

    /// Returns an iterator yielding the ink! specific items of the ink! trait
    /// definition.
    pub fn iter_items(&self) -> IterInkTraitItems {
        IterInkTraitItems::new(self)
    }

    /// Analyses the properties of the ink! trait definition.
    ///
    /// # Errors
    ///
    /// - If the trait has been defined as `unsafe`.
    /// - If the trait is an automatically implemented trait (`auto trait`).
    /// - If the trait is generic over some set of types.
    /// - If the trait's visibility is not public (`pub`).
    fn analyse_properties(item_trait: &syn::ItemTrait) -> Result<()> {
        if let Some(unsafety) = &item_trait.unsafety {
            return Err(format_err_spanned!(
                unsafety,
                "ink! trait definitions cannot be unsafe"
            ))
        }
        if let Some(auto) = &item_trait.auto_token {
            return Err(format_err_spanned!(
                auto,
                "ink! trait definitions cannot be automatically implemented traits"
            ))
        }
        if !item_trait.generics.params.is_empty() {
            return Err(format_err_spanned!(
                item_trait.generics.params,
                "ink! trait definitions must not be generic"
            ))
        }
        if !matches!(item_trait.vis, syn::Visibility::Public(_)) {
            return Err(format_err_spanned!(
                item_trait.vis,
                "ink! trait definitions must have public visibility"
            ))
        }
        if !item_trait.supertraits.is_empty() {
            return Err(format_err_spanned!(
                item_trait.supertraits,
                "ink! trait definitions with supertraits are not supported, yet"
            ))
        }
        Ok(())
    }

    /// Returns `Ok` if all trait items respects the requirements for an ink!
    /// trait definition.
    ///
    /// # Errors
    ///
    /// - If the trait contains an unsupported trait item such as
    ///     - associated constants (`const`)
    ///     - associated types (`type`)
    ///     - macros definitions or usages
    ///     - unknown token sequences (verbatim)
    ///     - methods with default implementations
    /// - If the trait contains methods which do not respect the ink! trait
    ///   definition requirements:
    ///     - All trait methods need to be declared as either `#[ink(message)]`
    ///       or `#[ink(constructor)]` and need to respect their respective
    ///       rules.
    ///
    /// # Note
    ///
    /// Associated types and constants might be allowed in the future.
    fn analyse_items(item_trait: &syn::ItemTrait) -> Result<()> {
        for trait_item in &item_trait.items {
            match trait_item {
                syn::TraitItem::Const(const_trait_item) => {
                    return Err(format_err_spanned!(
                        const_trait_item,
                        "associated constants in ink! trait definitions are not supported, yet"
                    ))
                }
                syn::TraitItem::Macro(macro_trait_item) => {
                    return Err(format_err_spanned!(
                        macro_trait_item,
                        "macros in ink! trait definitions are not supported"
                    ))
                }
                syn::TraitItem::Type(type_trait_item) => {
                    return Err(format_err_spanned!(
                    type_trait_item,
                    "associated types in ink! trait definitions are not supported, yet"
                ))
                }
                syn::TraitItem::Verbatim(verbatim) => {
                    return Err(format_err_spanned!(
                        verbatim,
                        "encountered unsupported item in ink! trait definition"
                    ))
                }
                syn::TraitItem::Method(method_trait_item) => {
                    Self::analyse_trait_method(method_trait_item)?;
                }
                unknown => {
                    return Err(format_err_spanned!(
                        unknown,
                        "encountered unknown or unsupported item in ink! trait definition"
                    ))
                }
            }
        }
        Ok(())
    }

    /// Analyses an ink! method that can be either an ink! message or
    /// constructor.
    ///
    /// # Errors
    ///
    /// - If the method declared as `unsafe`, `const` or `async`.
    /// - If the method has some explicit API.
    /// - If the method is variadic or has generic parameters.
    /// - If the method does not respect the properties of either an ink!
    ///   message or ink! constructor.
    fn analyse_trait_method(method: &syn::TraitItemMethod) -> Result<()> {
        if let Some(default_impl) = &method.default {
            return Err(format_err_spanned!(
                default_impl,
                "ink! trait methods with default implementations are not supported"
            ))
        }
        if let Some(constness) = &method.sig.constness {
            return Err(format_err_spanned!(
                constness,
                "const ink! trait methods are not supported"
            ))
        }
        if let Some(asyncness) = &method.sig.asyncness {
            return Err(format_err_spanned!(
                asyncness,
                "async ink! trait methods are not supported"
            ))
        }
        if let Some(unsafety) = &method.sig.unsafety {
            return Err(format_err_spanned!(
                unsafety,
                "unsafe ink! trait methods are not supported"
            ))
        }
        if let Some(abi) = &method.sig.abi {
            return Err(format_err_spanned!(
                abi,
                "ink! trait methods with non default ABI are not supported"
            ))
        }
        if let Some(variadic) = &method.sig.variadic {
            return Err(format_err_spanned!(
                variadic,
                "variadic ink! trait methods are not supported"
            ))
        }
        if !method.sig.generics.params.is_empty() {
            return Err(format_err_spanned!(
                method.sig.generics.params,
                "generic ink! trait methods are not supported"
            ))
        }
        match ir::first_ink_attribute(&method.attrs) {
            Ok(Some(ink_attr)) => {
                match ink_attr.first().kind() {
                    ir::AttributeArg::Message => {
                        Self::analyse_trait_message(method)?;
                    }
                    ir::AttributeArg::Constructor => {
                        Self::analyse_trait_constructor(method)?;
                    }
                    _unsupported => {
                        return Err(format_err_spanned!(
                            method,
                            "encountered unsupported ink! attribute for ink! trait method",
                        ))
                    }
                }
            }
            Ok(None) => {
                return Err(format_err_spanned!(
                    method,
                    "missing #[ink(message)] or #[ink(constructor)] flags on ink! trait method"
                ))
            }
            Err(err) => return Err(err),
        }
        Ok(())
    }

    /// Constructors are generally not allowed in ink! trait definitions.
    fn analyse_trait_constructor(constructor: &syn::TraitItemMethod) -> Result<()> {
        return Err(format_err!(
            constructor.span(),
            "ink! trait definitions must not have constructors",
        ))
    }

    /// Analyses the properties of an ink! message.
    ///
    /// # Errors
    ///
    /// - If the message has no `&self` or `&mut self` receiver.
    fn analyse_trait_message(message: &syn::TraitItemMethod) -> Result<()> {
        InkTraitMessage::extract_attributes(message.span(), &message.attrs)?;
        match message.sig.receiver() {
            None | Some(syn::FnArg::Typed(_)) => {
                return Err(format_err_spanned!(
                message.sig,
                "missing or malformed `&self` or `&mut self` receiver for ink! message",
            ))
            }
            Some(syn::FnArg::Receiver(receiver)) => {
                if receiver.reference.is_none() {
                    return Err(format_err_spanned!(
                        receiver,
                        "self receiver of ink! message must be `&self` or `&mut self`"
                    ))
                }
            }
        }
        Ok(())
    }

    /// Extract selectors for ink! trait constructors and messages.
    ///
    /// The composed or manually specified selectors are stored into the
    /// provided hash tables for later look-up when querying ink!
    /// constructors or messages. This way we are more flexible with regard
    /// to the underlying structures of the IR.
    ///
    /// In this step we assume that all sanitation checks have taken place prior
    /// so instead of returning errors we simply panic upon failures.
    ///
    /// # Errors
    ///
    /// Returns an error if there are overlapping selectors for ink!
    /// constructors or ink! messages. Note that overlaps between ink!
    /// constructor and message selectors are allowed.
    fn extract_selectors(
        config: &TraitDefinitionConfig,
        item_trait: &syn::ItemTrait,
        message_selectors: &mut HashMap<syn::Ident, Selector>,
    ) -> Result<()> {
        let mut seen_message_selectors = <HashMap<Selector, syn::Ident>>::new();
        let (_ink_attrs, _) = ir::sanitize_optional_attributes(
            item_trait.span(),
            item_trait.attrs.iter().cloned(),
            |arg| {
                match arg.kind() {
                    ir::AttributeArg::Namespace(_) => Ok(()),
                    _ => Err(None),
                }
            },
        )
        .expect("encountered unexpected invalid attributes on ink! trait definition");
        let namespace = config.namespace();
        let ident = &item_trait.ident;
        let trait_prefix = TraitPrefix::new(ident, namespace);
        for callable in IterInkTraitItemsRaw::from_raw(item_trait) {
            let ident = callable.ident();
            let ink_attrs = callable.ink_attrs();
            let selector = match ink_attrs.selector() {
                Some(manual_selector) => manual_selector,
                None => Selector::compose(trait_prefix, ident),
            };
            let (duplicate_selector, duplicate_ident) = match callable {
                InkTraitItem::Message(_) => {
                    let duplicate_selector =
                        seen_message_selectors.insert(selector, ident.clone());
                    let duplicate_ident =
                        message_selectors.insert(ident.clone(), selector);
                    (duplicate_selector, duplicate_ident)
                }
            };
            if let Some(duplicate_selector) = duplicate_selector {
                use crate::error::ExtError as _;
                return Err(format_err_spanned!(
                    ident,
                    "encountered duplicate selector ({:x?}) in the same ink! trait definition",
                    selector.to_bytes(),
                ).into_combine(format_err_spanned!(
                    duplicate_selector,
                    "first ink! trait constructor or message with same selector found here",
                )))
            }
            assert!(
                duplicate_ident.is_none(),
                "encountered unexpected overlapping ink! trait constructor or message identifier",
            );
        }
        Ok(())
    }
}
