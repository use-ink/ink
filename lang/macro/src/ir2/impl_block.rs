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
use syn::spanned::Spanned as _;

/// An ink! implementation block.
///
/// # Note
///
/// This can be either an inherent implementation block that implements some
/// constructors, messages or internal functions for the storage struct; OR it
/// can be a trait implementation for the storage struct.
#[derive(Debug, PartialEq, Eq)]
pub struct ImplBlock {
    attrs: Vec<syn::Attribute>,
    defaultness: Option<syn::token::Default>,
    unsafety: Option<syn::token::Unsafe>,
    impl_token: syn::token::Impl,
    generics: syn::Generics,
    trait_: Option<(Option<syn::token::Bang>, syn::Path, syn::token::For)>,
    self_ty: Box<syn::Type>,
    brace_token: syn::token::Brace,
    items: Vec<ImplBlockItem>,
}

impl ImplBlock {
    /// Returns `true` if the Rust implementation block is an ink! implementation
    /// block.
    ///
    /// # Note
    ///
    /// This is the case if:
    ///
    /// - The ink! implementation block has been annotatated as in:
    ///
    /// ```rust
    /// #[ink(impl)]
    /// impl MyStorage {
    ///     fn my_function(&self) { /* ... */ }
    /// }
    /// ```
    ///
    /// - Or if any of the ink! implementation block methods do have ink!
    ///   specific annotations:
    ///
    /// ```rust
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

impl TryFrom<syn::ItemImpl> for ImplBlock {
    type Error = syn::Error;

    fn try_from(item_impl: syn::ItemImpl) -> Result<Self, Self::Error> {
        // This can be either the ink! storage struct or an ink! event.
        let (ink_attrs, other_attrs) = ir2::partition_attributes(item_impl.attrs)?;
        todo!()
    }
}

impl ImplBlock {
    /// Returns an iterator yielding the ink! messages of the implementation block.
    pub fn iter_messages(&self) -> IterMessages {
        IterMessages::new(self)
    }

    /// Returns an iterator yielding the ink! messages of the implementation block.
    pub fn iter_constructors(&self) -> IterConstructors {
        IterConstructors::new(self)
    }
}

/// An item within an ink! implementation block.
#[derive(Debug, PartialEq, Eq)]
#[allow(clippy::large_enum_variant)]
pub enum ImplBlockItem {
    /// A `#[ink(constructor)]` marked inherent function.
    Constructor(Constructor),
    /// A `#[ink(message)]` marked method.
    Message(Message),
    /// Any other implementation block item.
    Other(syn::ImplItem),
}

impl TryFrom<syn::ImplItemMethod> for Message {
    type Error = syn::Error;

    fn try_from(method_item: syn::ImplItemMethod) -> Result<Self, Self::Error> {
        todo!()
    }
}

impl TryFrom<syn::ImplItemMethod> for Constructor {
    type Error = syn::Error;

    fn try_from(method_item: syn::ImplItemMethod) -> Result<Self, Self::Error> {
        todo!()
    }
}

impl TryFrom<syn::ImplItem> for ImplBlockItem {
    type Error = syn::Error;

    fn try_from(impl_item: syn::ImplItem) -> Result<Self, Self::Error> {
        match impl_item {
            syn::ImplItem::Method(method_item) => {
                if !ir2::contains_ink_attributes(&method_item.attrs) {
                    return Ok(Self::Other(method_item.into()))
                }
                let attr = ir2::first_ink_attribute(&method_item.attrs)?
                    .expect("missing expected ink! attribute for struct");
                match attr.first().kind() {
                    ir2::AttributeArgKind::Message => {
                        <Message as TryFrom<_>>::try_from(method_item)
                            .map(Into::into)
                            .map(Self::Message)
                    }
                    ir2::AttributeArgKind::Constructor => {
                        <Constructor as TryFrom<_>>::try_from(method_item)
                            .map(Into::into)
                            .map(Self::Constructor)
                    }
                    _ => Err(format_err!(
                        method_item,
                        "encountered invalid ink! attribute at this point, expected either \
                        #[ink(message)] or #[ink(constructor) attributes"
                    )),
                }
            }
            other_item => {
                // This is an error if the impl item contains any unexpected
                // ink! attributes. Otherwise it is a normal Rust item.
                if ir2::contains_ink_attributes(other_item.attrs()) {
                    let (ink_attrs, _) =
                        ir2::partition_attributes(other_item.attrs().iter().cloned())?;
                    assert!(!ink_attrs.is_empty());
                    fn into_err(attr: &ir2::InkAttribute) -> syn::Error {
                        format_err_span!(
                            attr.span(),
                            "encountered unexpected ink! attribute",
                        )
                    }
                    return Err(ink_attrs[1..]
                        .iter()
                        .map(into_err)
                        .fold(into_err(&ink_attrs[0]), |fst, snd| fst.into_combine(snd)))
                }
                Ok(Self::Other(other_item))
            }
        }
    }
}

impl ImplBlockItem {
    /// Returns `true` if the impl block item is an ink! message.
    pub fn is_message(&self) -> bool {
        self.filter_map_message().is_some()
    }

    /// Returns `Some` if `self` is an ink! message.
    ///
    /// Otherwise, returns `None`.
    pub fn filter_map_message(&self) -> Option<&Message> {
        match self {
            ImplBlockItem::Message(message) => Some(message),
            _ => None,
        }
    }

    /// Returns `true` if the impl block item is an ink! message.
    pub fn is_constructor(&self) -> bool {
        self.filter_map_constructor().is_some()
    }

    /// Returns `Some` if `self` is an ink! constructor.
    ///
    /// Otherwise, returns `None`.
    pub fn filter_map_constructor(&self) -> Option<&Constructor> {
        match self {
            ImplBlockItem::Constructor(constructor) => Some(constructor),
            _ => None,
        }
    }

    /// Returns `true` if the impl block item is a non ink! specific item.
    pub fn is_other_item(&self) -> bool {
        self.filter_map_other_item().is_some()
    }

    /// Returns `Some` if `self` is a not an ink! specific item.
    ///
    /// Otherwise, returns `None`.
    pub fn filter_map_other_item(&self) -> Option<&syn::ImplItem> {
        match self {
            ImplBlockItem::Other(rust_item) => Some(rust_item),
            _ => None,
        }
    }
}

/// An ink! constructor definition.
#[derive(Debug, PartialEq, Eq)]
pub struct Constructor {
    /// The underlying Rust method item.
    item: syn::ImplItemMethod,
}

/// An ink! message definition.
#[derive(Debug, PartialEq, Eq)]
pub struct Message {
    /// The underlying Rust method item.
    item: syn::ImplItemMethod,
}

/// The receiver of an ink! message.
#[derive(Copy, Clone)]
pub enum Receiver {
    /// The `&self` message receiver.
    Ref,
    /// The `&mut self` message receiver.
    RefMut,
}

impl Receiver {
    /// Returns `true` if the receiver is `&mut self`.
    pub fn is_ref(self) -> bool {
        match self {
            Receiver::Ref => true,
            _ => false,
        }
    }

    /// Returns `true` if the receiver is `&mut self`.
    pub fn is_ref_mut(self) -> bool {
        match self {
            Receiver::RefMut => true,
            _ => false,
        }
    }
}

impl Message {
    /// Returns the `self` receiver of the ink! message.
    pub fn receiver(&self) -> Receiver {
        todo!()
    }
}

/// Iterator yielding constructors of the ink! smart contract definition.
pub struct IterConstructors<'a> {
    impl_items: core::slice::Iter<'a, ImplBlockItem>,
}

impl<'a> IterConstructors<'a> {
    /// Creates a new ink! messages iterator.
    fn new(impl_block: &'a ImplBlock) -> Self {
        Self {
            impl_items: impl_block.items.iter(),
        }
    }
}

impl<'a> Iterator for IterConstructors<'a> {
    type Item = &'a ir2::Constructor;

    fn next(&mut self) -> Option<Self::Item> {
        'outer: loop {
            match self.impl_items.next() {
                None => return None,
                Some(impl_item) => {
                    if let Some(constructor) = impl_item.filter_map_constructor() {
                        return Some(constructor)
                    }
                    continue 'outer
                }
            }
        }
    }
}

/// Iterator yielding messages of the ink! smart contract definition.
pub struct IterMessages<'a> {
    impl_items: core::slice::Iter<'a, ImplBlockItem>,
}

impl<'a> IterMessages<'a> {
    /// Creates a new ink! messages iterator.
    fn new(impl_block: &'a ImplBlock) -> Self {
        Self {
            impl_items: impl_block.items.iter(),
        }
    }
}

impl<'a> Iterator for IterMessages<'a> {
    type Item = &'a ir2::Message;

    fn next(&mut self) -> Option<Self::Item> {
        'outer: loop {
            match self.impl_items.next() {
                None => return None,
                Some(impl_item) => {
                    if let Some(message) = impl_item.filter_map_message() {
                        return Some(message)
                    }
                    continue 'outer
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_ink_impl_block_eval_false_works() {
        let item_impls: Vec<syn::ItemImpl> = vec![
            syn::parse_quote! {
                impl MyStorage {}
            },
            syn::parse_quote! {
                impl MyTrait for MyStorage {}
            },
        ];
        for item_impl in &item_impls {
            assert_eq!(
                ir2::ImplBlock::is_ink_impl_block(item_impl)
                    .map_err(|err| err.to_string()),
                Ok(false),
            )
        }
    }

    #[test]
    fn is_ink_impl_block_eval_true_works() {
        let item_impls: Vec<syn::ItemImpl> = vec![
            syn::parse_quote! {
                #[ink(impl)]
                impl MyStorage {}
            },
            syn::parse_quote! {
                impl MyStorage {
                    #[ink(constructor)]
                    fn my_constructor() -> Self {}
                }
            },
            syn::parse_quote! {
                impl MyStorage {
                    #[ink(message)]
                    fn my_message(&self) {}
                }
            },
            syn::parse_quote! {
                #[ink(impl)]
                impl MyTrait for MyStorage {}
            },
            syn::parse_quote! {
                impl MyTrait for MyStorage {
                    #[ink(message)]
                    fn my_message(&self) {}
                }
            },
            syn::parse_quote! {
                #[ink(impl)]
                impl MyStorage {
                    #[ink(constructor)]
                    fn my_constructor() -> Self {}
                    #[ink(message)]
                    fn my_message(&self) {}
                }
            },
            syn::parse_quote! {
                #[ink(impl)]
                impl MyTrait for MyStorage {
                    #[ink(constructor)]
                    fn my_constructor() -> Self {}
                    #[ink(message)]
                    fn my_message(&self) {}
                }
            },
            syn::parse_quote! {
                // This is actually invalid but the function under test will
                // still determine this to be a valid ink! implementation block.
                #[ink(impl)]
                impl MyStorage {
                    #[ink(..)]
                    fn invalid_ink_attribute(&self) {}
                }
            },
        ];
        for item_impl in &item_impls {
            assert_eq!(
                ir2::ImplBlock::is_ink_impl_block(item_impl)
                    .map_err(|err| err.to_string()),
                Ok(true),
            )
        }
    }

    fn assert_is_ink_impl_block_fails(impl_block: &syn::ItemImpl, expected: &str) {
        assert_eq!(
            ir2::ImplBlock::is_ink_impl_block(impl_block).map_err(|err| err.to_string()),
            Err(expected.to_string())
        )
    }

    #[test]
    fn is_ink_impl_block_fails() {
        assert_is_ink_impl_block_fails(
            &syn::parse_quote! {
                #[ink(invalid)]
                impl MyStorage {}
            },
            "unknown ink! attribute (path)",
        );
        assert_is_ink_impl_block_fails(
            &syn::parse_quote! {
                #[ink(invalid)]
                impl MyTrait for MyStorage {}
            },
            "unknown ink! attribute (path)",
        );
        assert_is_ink_impl_block_fails(
            &syn::parse_quote! {
                #[ink(impl)]
                #[ink(impl)]
                impl MyStorage {}
            },
            "encountered duplicate ink! attribute",
        );
        assert_is_ink_impl_block_fails(
            &syn::parse_quote! {
                #[ink(impl)]
                #[ink(impl)]
                impl MyTrait for MyStorage {}
            },
            "encountered duplicate ink! attribute",
        );
        assert_is_ink_impl_block_fails(
            &syn::parse_quote! {
                impl MyStorage {
                    #[ink(invalid)]
                    fn invalid_fn_attr(&self) {}
                }
            },
            "unknown ink! attribute (path)",
        );
        assert_is_ink_impl_block_fails(
            &syn::parse_quote! {
                impl MyTrait for MyStorage {
                    #[ink(invalid)]
                    fn invalid_fn_attr(&self) {}
                }
            },
            "unknown ink! attribute (path)",
        );
    }
}
