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

use super::super::InkAttribute;
use crate::{
    ir::{
        self,
        utils,
    },
    InputsIter,
    Receiver,
};
use proc_macro2::Span;
use syn::{
    spanned::Spanned as _,
    Result,
};

/// An ink! item within an ink! trait definition.
#[derive(Debug, Clone)]
pub enum InkTraitItem<'a> {
    Message(InkTraitMessage<'a>),
}

impl<'a> InkTraitItem<'a> {
    /// Returns the Rust identifier of the ink! trait item.
    pub fn ident(&self) -> &syn::Ident {
        match self {
            Self::Message(message) => message.ident(),
        }
    }

    /// Returns the ink! attributes of the ink! trait item.
    pub fn ink_attrs(&self) -> InkAttribute {
        match self {
            Self::Message(message) => message.ink_attrs(),
        }
    }

    /// Returns `Some` if the ink! trait item is a message.
    pub fn filter_map_message(self) -> Option<InkTraitMessage<'a>> {
        match self {
            Self::Message(ink_trait_message) => Some(ink_trait_message),
        }
    }
}

/// A checked ink! message of an ink! trait definition.
#[derive(Debug, Clone)]
pub struct InkTraitMessage<'a> {
    item: &'a syn::TraitItemMethod,
}

impl<'a> InkTraitMessage<'a> {
    /// Panic message in case a user encounters invalid attributes.
    const INVALID_ATTRIBUTES_ERRSTR: &'static str =
        "encountered invalid attributes for ink! trait message";

    /// Creates a new ink! trait definition message.
    pub(super) fn new(item: &'a syn::TraitItemMethod) -> Self {
        Self { item }
    }

    /// Analyses and extracts the ink! and non-ink! attributes of an ink! trait message.
    pub(super) fn extract_attributes(
        span: Span,
        attrs: &[syn::Attribute],
    ) -> Result<(InkAttribute, Vec<syn::Attribute>)> {
        let (ink_attrs, non_ink_attrs) = ir::sanitize_attributes(
            span,
            attrs.iter().cloned(),
            &ir::AttributeArgKind::Message,
            |arg| {
                match arg.kind() {
                    ir::AttributeArg::Message
                    | ir::AttributeArg::Payable
                    | ir::AttributeArg::Selector(_) => Ok(()),
                    ir::AttributeArg::WildcardSelector =>
                        Err(Some(format_err!(arg.span(), "wildcard selectors are only supported for plain ink! messages, not for traits."))),
                    _ => Err(None),
                }
            },
        )?;
        Ok((ink_attrs, non_ink_attrs))
    }

    /// Returns all non-ink! attributes.
    pub fn attrs(&self) -> Vec<syn::Attribute> {
        let (_, rust_attrs) = Self::extract_attributes(self.span(), &self.item.attrs)
            .expect(Self::INVALID_ATTRIBUTES_ERRSTR);
        rust_attrs
    }

    /// Returns all ink! attributes.
    pub fn ink_attrs(&self) -> InkAttribute {
        let (ink_attrs, _) = Self::extract_attributes(self.span(), &self.item.attrs)
            .expect(Self::INVALID_ATTRIBUTES_ERRSTR);
        ink_attrs
    }

    /// Returns the original signature of the ink! message.
    pub fn sig(&self) -> &syn::Signature {
        &self.item.sig
    }

    /// Returns the `self` receiver of the ink! trait message.
    ///
    /// Returns `Ref` for `&self` messages and `RefMut` for `&mut self` messages.
    pub fn receiver(&self) -> Receiver {
        match self.item.sig.inputs.iter().next() {
            Some(syn::FnArg::Receiver(receiver)) => {
                debug_assert!(receiver.reference.is_some());
                if receiver.mutability.is_some() {
                    Receiver::RefMut
                } else {
                    Receiver::Ref
                }
            }
            _ => unreachable!("encountered invalid receiver argument for ink! message"),
        }
    }

    /// Returns an iterator over the inputs of the ink! trait message.
    pub fn inputs(&self) -> InputsIter {
        InputsIter::from(self)
    }

    /// Returns the return type of the ink! message if any.
    pub fn output(&self) -> Option<&syn::Type> {
        match &self.item.sig.output {
            syn::ReturnType::Default => None,
            syn::ReturnType::Type(_, return_type) => Some(return_type),
        }
    }

    /// Returns the Rust identifier of the ink! message.
    pub fn ident(&self) -> &syn::Ident {
        &self.item.sig.ident
    }

    /// Returns a local ID unique to the ink! trait definition of the ink! trait message.
    ///
    /// # Note
    ///
    /// It is a compile error if two ink! trait messages share the same local ID.
    /// Although the above scenario is very unlikely since the local ID is computed
    /// solely by the identifier of the ink! message.
    pub fn local_id(&self) -> u32 {
        utils::local_message_id(self.ident())
    }

    /// Returns the span of the ink! message.
    pub fn span(&self) -> Span {
        self.item.span()
    }

    /// Returns `true` if the ink! message may mutate the contract storage.
    pub fn mutates(&self) -> bool {
        self.sig()
            .receiver()
            .map(|fn_arg| {
                match fn_arg {
                    syn::FnArg::Receiver(receiver) if receiver.mutability.is_some() => {
                        true
                    }
                    syn::FnArg::Typed(pat_type) => {
                        match &*pat_type.ty {
                            syn::Type::Reference(reference)
                                if reference.mutability.is_some() =>
                            {
                                true
                            }
                            _ => false,
                        }
                    }
                    _ => false,
                }
            })
            .expect("encountered missing receiver for ink! message")
    }
}

impl<'a> From<&'a InkTraitMessage<'a>> for InputsIter<'a> {
    fn from(message: &'a InkTraitMessage) -> Self {
        Self::new(&message.item.sig.inputs)
    }
}
