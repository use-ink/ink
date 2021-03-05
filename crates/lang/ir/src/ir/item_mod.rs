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

use crate::{
    ir,
    ir::idents_lint,
};
use core::convert::TryFrom;
use proc_macro2::{
    Ident,
    Span,
};
use quote::TokenStreamExt as _;
use std::collections::HashMap;
use syn::{
    spanned::Spanned,
    token,
};

/// The ink! module.
///
/// This is the root of all ink! smart contracts and is defined similarly to
/// a normal Rust module annotated with
/// `#[ink::contract( /* optional configuration */ )]` attribute.
///
/// It contains ink! specific items as well as normal Rust items.
///
/// # Example
///
/// ```
/// // #[ink::contract] <-- this line belongs to the ink! configuration!
/// # use core::convert::TryFrom;
/// # use ink_lang_ir as ir;
/// # <ir::ItemMod as TryFrom<syn::ItemMod>>::try_from(syn::parse_quote! {
/// mod my_contract {
///     #[ink(storage)]
///     pub struct MyStorage {
///         /* storage fields */
///     }
///
///     #[ink(event)]
///     pub struct MyEvent {
///         /* event fields */
///     }
///
///     impl MyStorage {
///         #[ink(constructor)]
///         pub fn my_constructor() -> Self {
///             /* constructor initialization */
///         }
///
///         #[ink(message)]
///         pub fn my_message(&self) {
///             /* message statements */
///         }
///     }
/// }
/// # }).unwrap();
/// ```
///
/// # Note
///
/// This type has been named after [`syn::ItemMod`] and inherits all of the
/// fields that are required for inline module definitions.
///
/// # Developer Note
///
/// Structurally the ink! `Module` mirrors an inline Rust module, for example:
///
/// ```
/// mod rust_module {
///     /* some Rust item definitions */
/// }
/// ```
///
/// If the capabilities of an inline Rust module change we have to adjust for that.
#[derive(Debug, PartialEq, Eq)]
pub struct ItemMod {
    attrs: Vec<syn::Attribute>,
    vis: syn::Visibility,
    mod_token: token::Mod,
    ident: Ident,
    brace: token::Brace,
    items: Vec<ir::Item>,
}

impl ItemMod {
    /// Ensures that the ink! storage struct is not missing and that there are
    /// not multiple ink! storage struct definitions for the given slice of items.
    fn ensure_storage_struct_quantity(
        module_span: Span,
        items: &[ir::Item],
    ) -> Result<(), syn::Error> {
        let storage_iter = items
            .iter()
            .filter(|item| matches!(item, ir::Item::Ink(ir::InkItem::Storage(_))));
        if storage_iter.clone().next().is_none() {
            return Err(format_err!(module_span, "missing ink! storage struct",))
        }
        if storage_iter.clone().count() >= 2 {
            let mut error = format_err!(
                module_span,
                "encountered multiple ink! storage structs, expected exactly one"
            );
            for storage in storage_iter {
                error.combine(format_err!(storage, "ink! storage struct here"))
            }
            return Err(error)
        }
        Ok(())
    }

    /// Ensures that the given slice of items contains at least one ink! message.
    fn ensure_contains_message(
        module_span: Span,
        items: &[ir::Item],
    ) -> Result<(), syn::Error> {
        let found_message = items
            .iter()
            .filter_map(|item| {
                match item {
                    ir::Item::Ink(ir::InkItem::ImplBlock(item_impl)) => {
                        Some(item_impl.iter_messages())
                    }
                    _ => None,
                }
            })
            .any(|mut messages| messages.next().is_some());
        if !found_message {
            return Err(format_err!(module_span, "missing ink! message"))
        }
        Ok(())
    }

    /// Ensures that the given slice of items contains at least one ink! constructor.
    fn ensure_contains_constructor(
        module_span: Span,
        items: &[ir::Item],
    ) -> Result<(), syn::Error> {
        let found_constructor = items
            .iter()
            .filter_map(|item| {
                match item {
                    ir::Item::Ink(ir::InkItem::ImplBlock(item_impl)) => {
                        Some(item_impl.iter_constructors())
                    }
                    _ => None,
                }
            })
            .any(|mut constructors| constructors.next().is_some());
        if !found_constructor {
            return Err(format_err!(module_span, "missing ink! constructor"))
        }
        Ok(())
    }

    /// Ensures that no ink! message or constructor selectors are overlapping.
    ///
    /// # Note
    ///
    /// We differentiate between ink! message and ink! constructor selectors
    /// since they are dispatched independently from each other and thus are
    /// allowed to have overlapping selectors.
    fn ensure_no_overlapping_selectors(items: &[ir::Item]) -> Result<(), syn::Error> {
        let mut messages = <HashMap<ir::Selector, &ir::Message>>::new();
        let mut constructors = <HashMap<ir::Selector, &ir::Constructor>>::new();
        for item_impl in items
            .iter()
            .filter_map(ir::Item::map_ink_item)
            .filter_map(ir::InkItem::filter_map_impl_block)
        {
            use std::collections::hash_map::Entry;
            /// Kind is either `"message"` or `"constructor"`.
            fn compose_error(
                first_span: Span,
                second_span: Span,
                selector: ir::Selector,
                kind: &str,
            ) -> syn::Error {
                use crate::error::ExtError as _;
                format_err!(
                    second_span,
                    "encountered ink! {}s with overlapping selectors (= {:02X?})\n\
                     hint: use #[ink(selector = \"0x...\")] on the callable or \
                     #[ink(namespace = \"...\")] on the implementation block to \
                     disambiguate overlapping selectors.",
                    kind,
                    selector.as_bytes(),
                )
                .into_combine(format_err!(
                    first_span,
                    "first ink! {} with overlapping selector here",
                    kind,
                ))
            }
            for message in item_impl.iter_messages() {
                let selector = message.composed_selector();
                match messages.entry(selector) {
                    Entry::Occupied(overlap) => {
                        return Err(compose_error(
                            overlap.get().span(),
                            message.callable().span(),
                            selector,
                            "message",
                        ))
                    }
                    Entry::Vacant(vacant) => {
                        vacant.insert(message.callable());
                    }
                }
            }
            for constructor in item_impl.iter_constructors() {
                let selector = constructor.composed_selector();
                match constructors.entry(selector) {
                    Entry::Occupied(overlap) => {
                        return Err(compose_error(
                            overlap.get().span(),
                            constructor.callable().span(),
                            selector,
                            "constructor",
                        ))
                    }
                    Entry::Vacant(vacant) => {
                        vacant.insert(constructor.callable());
                    }
                }
            }
        }
        Ok(())
    }
}

impl TryFrom<syn::ItemMod> for ItemMod {
    type Error = syn::Error;

    fn try_from(module: syn::ItemMod) -> Result<Self, Self::Error> {
        let module_span = module.span();
        idents_lint::ensure_no_ink_identifiers(&module)?;
        let (brace, items) = match module.content {
            Some((brace, items)) => (brace, items),
            None => {
                return Err(format_err_spanned!(
                    module,
                    "out-of-line ink! modules are not supported, use `#[ink::contract] mod name {{ ... }}`",
                ))
            }
        };
        let (ink_attrs, other_attrs) = ir::partition_attributes(module.attrs)?;
        if !ink_attrs.is_empty() {
            let mut error = format_err!(
                module_span,
                "encountered invalid ink! attributes on ink! module"
            );
            for ink_attr in ink_attrs {
                error.combine(format_err!(
                    ink_attr.span(),
                    "invalid ink! attribute on module"
                ))
            }
            return Err(error)
        }
        let items = items
            .into_iter()
            .map(<ir::Item as TryFrom<syn::Item>>::try_from)
            .collect::<Result<Vec<_>, syn::Error>>()?;
        Self::ensure_storage_struct_quantity(module_span, &items)?;
        Self::ensure_contains_message(module_span, &items)?;
        Self::ensure_contains_constructor(module_span, &items)?;
        Self::ensure_no_overlapping_selectors(&items)?;
        Ok(Self {
            attrs: other_attrs,
            vis: module.vis,
            mod_token: module.mod_token,
            ident: module.ident,
            brace,
            items,
        })
    }
}

impl quote::ToTokens for ItemMod {
    /// We mainly implement this trait for ink! module to have a derived
    /// [`Spanned`](`syn::spanned::Spanned`) implementation for it.
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.append_all(
            self.attrs
                .iter()
                .filter(|attr| matches!(attr.style, syn::AttrStyle::Outer)),
        );
        self.vis.to_tokens(tokens);
        self.mod_token.to_tokens(tokens);
        self.ident.to_tokens(tokens);
        self.brace.surround(tokens, |tokens| {
            tokens.append_all(
                self.attrs
                    .iter()
                    .filter(|attr| matches!(attr.style, syn::AttrStyle::Inner(_))),
            );
            tokens.append_all(&self.items);
        });
    }
}

impl ItemMod {
    /// Returns the identifier of the ink! module.
    pub fn ident(&self) -> &Ident {
        &self.ident
    }

    /// Returns the storage struct definition for this ink! module.
    ///
    /// # Note
    ///
    /// The storage definition is the struct that has been annotated with
    /// `#[ink(storage)]`. This struct is required to be defined in the root
    /// of the ink! inline module.
    ///
    /// # Panics
    ///
    /// If zero or multiple `#[ink(storage)]` annotated structs were found in
    /// the ink! module. This can be expected to never happen since upon
    /// construction of an ink! module it is asserted that exactly one
    /// `#[ink(storage)]` struct exists.
    pub fn storage(&self) -> &ir::Storage {
        let mut iter = IterInkItems::new(self)
            .filter_map(|ink_item| ink_item.filter_map_storage_item());
        let storage = iter
            .next()
            .expect("encountered ink! module without a storage struct");
        assert!(
            iter.next().is_none(),
            "encountered multiple storage structs in ink! module"
        );
        storage
    }

    /// Returns all (ink! and non-ink! specific) item definitions of the ink! inline module.
    pub fn items(&self) -> &[ir::Item] {
        self.items.as_slice()
    }

    /// Returns an iterator yielding all ink! implementation blocks.
    ///
    /// # Note
    ///
    /// An ink! implementation block can be either an inherent `impl` block
    /// directly defined for the contract's storage struct if it includes at
    /// least one `#[ink(message)]` or `#[ink(constructor)]` annotation, e.g.:
    ///
    /// ```
    /// # use core::convert::TryFrom;
    /// # use ink_lang_ir as ir;
    /// # <ir::ItemMod as TryFrom<syn::ItemMod>>::try_from(syn::parse_quote! {
    /// # mod my_module {
    /// # #[ink(storage)]
    /// # pub struct MyStorage {
    /// #     /* storage fields */
    /// # }
    /// #
    /// impl MyStorage {
    /// #   #[ink(constructor)]
    /// #   pub fn my_constructor() -> Self {
    /// #       /* constructor implementation */
    /// #   }
    /// #
    ///     #[ink(message)]
    ///     pub fn my_message(&self) {
    ///         /* message implementation */
    ///     }
    /// }
    /// # }}).unwrap();
    /// ```
    ///
    /// Also an implementation block can be defined as a trait implementation
    /// for the ink! storage struct using the `#[ink(impl)]` annotation even
    /// if none of its interior items have any ink! specific attributes on them,
    /// e.g.:
    ///
    /// ```
    /// # use core::convert::TryFrom;
    /// # use ink_lang_ir as ir;
    /// # <ir::ItemMod as TryFrom<syn::ItemMod>>::try_from(syn::parse_quote! {
    /// # mod my_module {
    /// # #[ink(storage)]
    /// # pub struct MyStorage {
    /// #     /* storage fields */
    /// # }
    /// #
    /// #[ink(impl)]
    /// impl MyStorage {
    ///     fn my_method(&self) -> i32 {
    ///         /* method implementation */
    ///     }
    /// }
    /// #
    /// # impl MyStorage {
    /// #   #[ink(constructor)]
    /// #   pub fn my_constructor() -> Self {
    /// #       /* constructor implementation */
    /// #   }
    /// #
    /// #   #[ink(message)]
    /// #   pub fn my_message(&self) {
    /// #       /* message implementation */
    /// #   }
    /// # }
    /// # }}).unwrap();
    /// ```
    pub fn impls(&self) -> IterItemImpls {
        IterItemImpls::new(self)
    }

    /// Returns an iterator yielding all event definitions in this ink! module.
    pub fn events(&self) -> IterEvents {
        IterEvents::new(self)
    }

    /// Returns all non-ink! attributes of the ink! module.
    pub fn attrs(&self) -> &[syn::Attribute] {
        &self.attrs
    }

    /// Returns the visibility of the ink! module.
    pub fn vis(&self) -> &syn::Visibility {
        &self.vis
    }
}

/// Iterator yielding ink! item definitions of the ink! smart contract.
pub struct IterInkItems<'a> {
    items_iter: core::slice::Iter<'a, ir::Item>,
}

impl<'a> IterInkItems<'a> {
    /// Creates a new ink! module items iterator.
    fn new(ink_module: &'a ItemMod) -> Self {
        Self {
            items_iter: ink_module.items.iter(),
        }
    }
}

impl<'a> Iterator for IterInkItems<'a> {
    type Item = &'a ir::InkItem;

    fn next(&mut self) -> Option<Self::Item> {
        'repeat: loop {
            match self.items_iter.next() {
                None => return None,
                Some(item) => {
                    if let Some(event) = item.map_ink_item() {
                        return Some(event)
                    }
                    continue 'repeat
                }
            }
        }
    }
}

/// Iterator yielding all ink! event definitions within the ink!
/// [`ItemMod`](`crate::ir::ItemMod`).
pub struct IterEvents<'a> {
    items_iter: IterInkItems<'a>,
}

impl<'a> IterEvents<'a> {
    /// Creates a new ink! events iterator.
    fn new(ink_module: &'a ItemMod) -> Self {
        Self {
            items_iter: IterInkItems::new(ink_module),
        }
    }
}

impl<'a> Iterator for IterEvents<'a> {
    type Item = &'a ir::Event;

    fn next(&mut self) -> Option<Self::Item> {
        'repeat: loop {
            match self.items_iter.next() {
                None => return None,
                Some(ink_item) => {
                    if let Some(event) = ink_item.filter_map_event_item() {
                        return Some(event)
                    }
                    continue 'repeat
                }
            }
        }
    }
}

/// Iterator yielding all ink! implementation block definitions within the ink!
/// [`ItemMod`](`crate::ir::ItemMod`).
pub struct IterItemImpls<'a> {
    items_iter: IterInkItems<'a>,
}

impl<'a> IterItemImpls<'a> {
    /// Creates a new ink! implementation blocks iterator.
    fn new(ink_module: &'a ItemMod) -> Self {
        Self {
            items_iter: IterInkItems::new(ink_module),
        }
    }
}

impl<'a> Iterator for IterItemImpls<'a> {
    type Item = &'a ir::ItemImpl;

    fn next(&mut self) -> Option<Self::Item> {
        'repeat: loop {
            match self.items_iter.next() {
                None => return None,
                Some(ink_item) => {
                    if let Some(event) = ink_item.filter_map_impl_block() {
                        return Some(event)
                    }
                    continue 'repeat
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate as ir;

    #[test]
    fn item_mod_try_from_works() {
        let item_mods: Vec<syn::ItemMod> = vec![
            syn::parse_quote! {
                mod minimal {
                    #[ink(storage)]
                    pub struct Minimal {}

                    impl Minimal {
                        #[ink(constructor)]
                        pub fn new() -> Self {}
                        #[ink(message)]
                        pub fn minimal_message(&self) {}
                    }
                }
            },
            syn::parse_quote! {
                mod flipper {
                    #[ink(storage)]
                    pub struct Flipper {
                        value: bool,
                    }

                    impl Default for Flipper {
                        #[ink(constructor)]
                        fn default() -> Self {
                            Self { value: false }
                        }
                    }

                    impl Flipper {
                        #[ink(message)]
                        pub fn flip(&mut self) {
                            self.value = !self.value
                        }

                        #[ink(message)]
                        pub fn get(&self) -> bool {
                            self.value
                        }
                    }
                }
            },
        ];
        for item_mod in item_mods {
            assert!(<ir::ItemMod as TryFrom<syn::ItemMod>>::try_from(item_mod).is_ok())
        }
    }

    fn assert_fail(item_mod: syn::ItemMod, expected_err: &str) {
        assert_eq!(
            <ir::ItemMod as TryFrom<syn::ItemMod>>::try_from(item_mod)
                .map_err(|err| err.to_string()),
            Err(expected_err.to_string()),
        );
    }

    #[test]
    fn missing_storage_struct_fails() {
        assert_fail(
            syn::parse_quote! {
                mod my_module {
                    impl MyStorage {
                        #[ink(constructor)]
                        pub fn my_constructor() -> Self {}
                        #[ink(message)]
                        pub fn my_message(&self) {}
                    }
                }
            },
            "missing ink! storage struct",
        )
    }

    #[test]
    fn multiple_storage_struct_fails() {
        assert_fail(
            syn::parse_quote! {
                mod my_module {
                    #[ink(storage)]
                    pub struct MyFirstStorage {}
                    #[ink(storage)]
                    pub struct MySecondStorage {}
                    impl MyFirstStorage {
                        #[ink(constructor)]
                        pub fn my_constructor() -> Self {}
                        #[ink(message)]
                        pub fn my_message(&self) {}
                    }
                }
            },
            "encountered multiple ink! storage structs, expected exactly one",
        )
    }

    #[test]
    fn missing_constructor_fails() {
        assert_fail(
            syn::parse_quote! {
                mod my_module {
                    #[ink(storage)]
                    pub struct MyStorage {}

                    impl MyStorage {
                        #[ink(message)]
                        pub fn my_message(&self) {}
                    }
                }
            },
            "missing ink! constructor",
        )
    }

    #[test]
    fn missing_message_fails() {
        assert_fail(
            syn::parse_quote! {
                mod my_module {
                    #[ink(storage)]
                    pub struct MyStorage {}

                    impl MyStorage {
                        #[ink(constructor)]
                        pub fn my_constructor() -> Self {}
                    }
                }
            },
            "missing ink! message",
        )
    }

    #[test]
    fn invalid_out_of_line_module_fails() {
        assert_fail(
            syn::parse_quote! {
                mod my_module;
            },
            "out-of-line ink! modules are not supported, use `#[ink::contract] mod name { ... }`",
        )
    }

    #[test]
    fn conflicting_attributes_fails() {
        assert_fail(
            syn::parse_quote! {
                #[ink(namespace = "my_namespace")]
                mod my_module {
                    #[ink(storage)]
                    pub struct MyStorage {}
                    impl MyStorage {
                        #[ink(constructor)]
                        pub fn my_constructor() -> Self {}
                        #[ink(message)]
                        pub fn my_message(&self) {}
                    }
                }
            },
            "encountered invalid ink! attributes on ink! module",
        )
    }

    #[test]
    fn overlapping_messages_fails() {
        assert_fail(
            syn::parse_quote! {
                mod my_module {
                    #[ink(storage)]
                    pub struct MyStorage {}

                    impl MyStorage {
                        #[ink(constructor)]
                        pub fn my_constructor() -> Self {}

                        #[ink(message, selector = "0xDEADBEEF")]
                        pub fn my_message_1(&self) {}
                    }

                    impl MyStorage {
                        #[ink(message, selector = "0xDEADBEEF")]
                        pub fn my_message_2(&self) {}
                    }
                }
            },
            "encountered ink! messages with overlapping selectors (= [DE, AD, BE, EF])\n\
                hint: use #[ink(selector = \"0x...\")] on the callable or \
                #[ink(namespace = \"...\")] on the implementation block to \
                disambiguate overlapping selectors.",
        );
    }

    #[test]
    fn overlapping_constructors_fails() {
        assert_fail(
            syn::parse_quote! {
                mod my_module {
                    #[ink(storage)]
                    pub struct MyStorage {}

                    impl MyStorage {
                        #[ink(constructor, selector = "0xDEADBEEF")]
                        pub fn my_constructor_1() -> Self {}

                        #[ink(message)]
                        pub fn my_message_1(&self) {}
                    }

                    impl MyStorage {
                        #[ink(constructor, selector = "0xDEADBEEF")]
                        pub fn my_constructor_2() -> Self {}
                    }
                }
            },
            "encountered ink! constructors with overlapping selectors (= [DE, AD, BE, EF])\n\
                hint: use #[ink(selector = \"0x...\")] on the callable or \
                #[ink(namespace = \"...\")] on the implementation block to \
                disambiguate overlapping selectors.",
        );
    }

    #[test]
    fn overlapping_trait_impls_fails() {
        assert_fail(
            syn::parse_quote! {
                mod my_module {
                    #[ink(storage)]
                    pub struct MyStorage {}

                    impl first::MyTrait for MyStorage {
                        #[ink(constructor)]
                        fn my_constructor() -> Self {}

                        #[ink(message)]
                        fn my_message(&self) {}
                    }

                    impl second::MyTrait for MyStorage {
                        #[ink(message)]
                        fn my_message(&self) {}
                    }
                }
            },
            "encountered ink! messages with overlapping selectors (= [04, C4, 94, 46])\n\
                hint: use #[ink(selector = \"0x...\")] on the callable or \
                #[ink(namespace = \"...\")] on the implementation block to \
                disambiguate overlapping selectors.",
        );
    }

    #[test]
    fn namespaced_overlapping_trait_impls_works() {
        assert!(
            <ir::ItemMod as TryFrom<syn::ItemMod>>::try_from(syn::parse_quote! {
                mod my_module {
                    #[ink(storage)]
                    pub struct MyStorage {}

                    #[ink(namespace = "first")]
                    impl first::MyTrait for MyStorage {
                        #[ink(constructor)]
                        fn my_constructor() -> Self {}

                        #[ink(message)]
                        fn my_message(&self) {}
                    }

                    impl second::MyTrait for MyStorage {
                        #[ink(message)]
                        fn my_message(&self) {}
                    }
                }
            })
            .is_ok()
        );
    }

    #[test]
    fn allow_overlap_between_messages_and_constructors() {
        assert!(
            <ir::ItemMod as TryFrom<syn::ItemMod>>::try_from(syn::parse_quote! {
                mod my_module {
                    #[ink(storage)]
                    pub struct MyStorage {}

                    impl MyStorage {
                        #[ink(constructor, selector = "0xDEADBEEF")]
                        pub fn my_constructor() -> Self {}

                        #[ink(message, selector = "0xDEADBEEF")]
                        pub fn my_message(&self) {}
                    }
                }
            })
            .is_ok()
        );
    }
}
