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

use crate::ir2;
use core::convert::TryFrom;

/// An ink! implementation block.
///
/// # Note
///
/// This can be either an inherent implementation block that implements some
/// constructors, messages or internal functions for the storage struct; OR it
/// can be a trait implementation for the storage struct.
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
pub enum ImplBlockItem {
    /// A `#[ink(constructor)]` marked inherent function.
    Constructor(Constructor),
    /// A `#[ink(message)]` marked method.
    Message(Message),
    /// Any other normal Rust implementation block item.
    Rust(syn::ImplItem),
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

    /// Returns `true` if the impl block item is a normal Rust item.
    pub fn is_rust_item(&self) -> bool {
        self.filter_map_rust_item().is_some()
    }

    /// Returns `Some` if `self` is a normal Rust item.
    ///
    /// Otherwise, returns `None`.
    pub fn filter_map_rust_item(&self) -> Option<&syn::ImplItem> {
        match self {
            ImplBlockItem::Rust(rust_item) => Some(rust_item),
            _ => None,
        }
    }
}

/// An ink! constructor definition.
pub struct Constructor {
    /// The underlying Rust method item.
    item: syn::ImplItemMethod,
}

/// An ink! message definition.
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
