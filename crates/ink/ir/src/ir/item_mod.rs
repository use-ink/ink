// Copyright (C) Use Ink (UK) Ltd.
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
    Callable,
    error::ExtError as _,
    ir,
    ir::idents_lint,
};
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
/// # use ink_ir as ir;
/// # <ir::ItemMod as TryFrom<syn::ItemMod>>::try_from(syn::parse_quote! {
/// mod my_contract {
///     #[ink(storage)]
///     pub struct MyStorage {
///         // storage fields
///     }
///
///     #[ink(event)]
///     pub struct MyEvent {
///         // event fields
///     }
///
///     impl MyStorage {
///         #[ink(constructor)]
///         pub fn my_constructor() -> Self {
///             // constructor initialization
///         }
///
///         #[ink(message)]
///         pub fn my_message(&self) {
///             // message statements
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
///     // some Rust item definitions
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
    ///
    /// # Note
    ///
    /// Also ensure that there's only one constructor in "sol" ABI mode,
    /// or that at least one constructor is annotated as the default constructor in "all"
    /// ABI mode.
    ///
    /// See <https://use.ink/docs/v6/basics/abi> for details about ABI modes.
    fn ensure_contains_constructor(
        module_span: Span,
        items: &[ir::Item],
    ) -> Result<(), syn::Error> {
        let all_constructors = || {
            items
                .iter()
                .filter_map(|item| {
                    match item {
                        ir::Item::Ink(ir::InkItem::ImplBlock(item_impl)) => {
                            Some(item_impl.iter_constructors())
                        }
                        _ => None,
                    }
                })
                .flatten()
        };

        let n_constructors = all_constructors().count();
        if n_constructors == 0 {
            return Err(format_err!(module_span, "missing ink! constructor"));
        }

        #[cfg(ink_abi = "sol")]
        if n_constructors > 1 {
            return Err(format_err!(
                module_span,
                "multiple constructors are not supported in Solidity ABI compatibility mode"
            ));
        }

        #[cfg(ink_abi = "all")]
        {
            let has_default_constructor =
                || all_constructors().any(|constructor| constructor.is_default());
            if n_constructors > 1 && !has_default_constructor() {
                return Err(format_err!(
                    module_span,
                    "One constructor used for Solidity ABI encoded instantiation \
                    must be annotated with the `default` attribute argument \
                    in \"all\" ABI mode"
                ));
            }
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
                format_err!(
                    second_span,
                    "encountered ink! {}s with overlapping selectors (= {:02X?})\n\
                     hint: use #[ink(selector = S:u32)] on the callable or \
                     #[ink(namespace = N:string)] on the implementation block to \
                     disambiguate overlapping selectors.",
                    kind,
                    selector.to_bytes(),
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

    /// Ensures that the `#[cfg(…)]` contains only valid attributes.
    ///
    /// # Note
    ///
    /// This restriction was added to prevent contract developers from
    /// adding public constructors/messages that don't show up in the
    /// ink! metadata, but are compiled into the contract binary.
    ///
    /// Or formulated differently: we allow only `#[cfg(…)]`'s that don't
    /// allow differentiating between compiling for PolkaVM vs. native.
    ///
    /// Without this restriction users that view the metadata can be
    /// deceived as to what functions the contract provides to the public.
    fn ensure_only_allowed_cfgs(items: &[ir::Item]) -> Result<(), syn::Error> {
        const ERR_HELP: &str = "Allowed in `#[cfg(…)]`:\n\
               - `test`\n\
               - `feature` (without `std`)\n\
               - `any`\n\
               - `not`\n\
               - `all`";

        fn verify_attr(a: &syn::Attribute) -> Result<(), syn::Error> {
            match &a.meta {
                syn::Meta::List(list) => {
                    if let Some(ident) = list.path.get_ident()
                        && ident.eq("cfg")
                    {
                        return list.parse_nested_meta(verify_cfg_attrs);
                    }
                    unreachable!(
                        "`verify_attr` can only be called for `#[cfg(…)]`, not for other `List`"
                    );
                }
                syn::Meta::Path(_) => {
                    // not relevant, we are only looking at `#[cfg(…)]`
                    unreachable!(
                        "`verify_attr` can only be called for `#[cfg(…)]`, not for `Path`"
                    );
                }
                syn::Meta::NameValue(_) => {
                    // not relevant, we are only looking at `#[cfg(…)]`
                    unreachable!(
                        "`verify_attr` can only be called for `#[cfg(…)]`, not for `NameValue`"
                    );
                }
            }
        }

        fn verify_cfg_attrs(meta: syn::meta::ParseNestedMeta) -> Result<(), syn::Error> {
            if meta.path.is_ident("test") {
                return Ok(());
            }
            if meta.path.is_ident("any")
                || meta.path.is_ident("all")
                || meta.path.is_ident("not")
            {
                return meta.parse_nested_meta(verify_cfg_attrs);
            }

            if meta.path.is_ident("feature") {
                let value = meta.value()?;
                let value: syn::LitStr = value.parse()?;
                if value.value().eq("std") {
                    return Err(format_err_spanned!(
                        meta.path,
                        "The feature `std` is not allowed in `cfg`.\n\n{ERR_HELP}"
                    ))
                }
                return Ok(());
            }

            Err(format_err_spanned!(
                meta.path,
                "This `cfg` attribute is not allowed.\n\n{ERR_HELP}"
            ))
        }

        for item_impl in items
            .iter()
            .filter_map(ir::Item::map_ink_item)
            .filter_map(ir::InkItem::filter_map_impl_block)
        {
            for message in item_impl.iter_messages() {
                for a in message.get_cfg_syn_attrs() {
                    verify_attr(&a)?;
                }
            }
            for constructor in item_impl.iter_constructors() {
                for a in constructor.get_cfg_syn_attrs() {
                    verify_attr(&a)?;
                }
            }
        }
        Ok(())
    }

    /// Ensures that:
    /// - At most one wildcard selector exists among ink! messages, as well as ink!
    ///   constructors.
    /// - Where a wildcard selector is defined for a message, at most one other message is
    ///   defined which must have a well known selector.
    fn ensure_valid_wildcard_selector_usage(
        items: &[ir::Item],
    ) -> Result<(), syn::Error> {
        let mut wildcard_selector: Option<&ir::Message> = None;
        let mut other_messages = Vec::new();
        for item_impl in items
            .iter()
            .filter_map(ir::Item::map_ink_item)
            .filter_map(ir::InkItem::filter_map_impl_block)
        {
            for message in item_impl.iter_messages() {
                if !message.has_wildcard_selector() {
                    other_messages.push(message);
                    continue
                }
                match wildcard_selector {
                    None => wildcard_selector = Some(message.callable()),
                    Some(overlap) => {
                        let err = format_err!(
                            message.callable().span(),
                            "encountered ink! messages with overlapping wildcard selectors",
                        );
                        let overlap_err = format_err!(
                            overlap.span(),
                            "first ink! message with overlapping wildcard selector here",
                        );
                        return Err(err.into_combine(overlap_err))
                    }
                }
            }

            if let Some(wildcard) = wildcard_selector {
                match other_messages.len() as u32 {
                    0 => {
                        return Err(format_err!(
                            wildcard.span(),
                            "missing definition of another message with TODO in tandem with a wildcard \
                        selector",
                        ))
                    }
                    1 => {
                        if !other_messages[0]
                            .callable()
                            .has_wildcard_complement_selector()
                        {
                            return Err(format_err!(
                                other_messages[0].callable().span(),
                                "when using a wildcard selector `selector = _` for an ink! message \
                                then the other message must use the wildcard complement `selector = @`"
                            ))
                        }
                    }
                    2.. => {
                        let mut combined = format_err!(
                            wildcard.span(),
                            "exactly one other message must be defined together with a wildcard selector",
                        );
                        for message in &other_messages {
                            if !message.callable().has_wildcard_complement_selector() {
                                combined.combine(
                                    format_err!(
                                        message.callable().span(),
                                        "additional message not permitted together with a wildcard selector",
                                    )
                                )
                            }
                        }
                        return Err(combined)
                    }
                }
            } else {
                for message in &other_messages {
                    if message.callable().has_wildcard_complement_selector() {
                        return Err(format_err!(
                            message.callable().span(),
                            "encountered ink! message with wildcard complement `selector = @` but no \
                             wildcard `selector = _` defined"
                        ));
                    }
                }
            }

            let mut wildcard_selector: Option<&ir::Constructor> = None;
            for constructor in item_impl.iter_constructors() {
                if !constructor.has_wildcard_selector() {
                    continue
                }
                match wildcard_selector {
                    None => wildcard_selector = Some(constructor.callable()),
                    Some(overlap) => {
                        return Err(format_err!(
                            constructor.callable().span(),
                            "encountered ink! constructor with overlapping wildcard selectors",
                        )
                            .into_combine(format_err!(
                            overlap.span(),
                            "first ink! constructor with overlapping wildcard selector here",
                        )))
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
        Self::ensure_valid_wildcard_selector_usage(&items)?;
        Self::ensure_only_allowed_cfgs(&items)?;
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

    /// Returns all (ink! and non-ink! specific) item definitions of the ink! inline
    /// module.
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
    /// # use ink_ir as ir;
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
    ///         // message implementation
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
    /// # use ink_ir as ir;
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
    ///         // method implementation
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
    pub fn impls(&self) -> IterItemImpls<'_> {
        IterItemImpls::new(self)
    }

    /// Returns an iterator yielding all event definitions in this ink! module.
    pub fn events(&self) -> IterEvents<'_> {
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

                        #[ink(message, selector = 0xDEADBEEF)]
                        pub fn my_message_1(&self) {}
                    }

                    impl MyStorage {
                        #[ink(message, selector = 0xDEADBEEF)]
                        pub fn my_message_2(&self) {}
                    }
                }
            },
            "encountered ink! messages with overlapping selectors (= [DE, AD, BE, EF])\n\
            hint: use #[ink(selector = S:u32)] on the callable or #[ink(namespace = N:string)] \
            on the implementation block to disambiguate overlapping selectors.",
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
                        #[ink(constructor, selector = 0xDEADBEEF)]
                        pub fn my_constructor_1() -> Self {}

                        #[ink(message)]
                        pub fn my_message_1(&self) {}
                    }

                    impl MyStorage {
                        #[ink(constructor, selector = 0xDEADBEEF)]
                        pub fn my_constructor_2() -> Self {}
                    }
                }
            },
            "encountered ink! constructors with overlapping selectors (= [DE, AD, BE, EF])\n\
            hint: use #[ink(selector = S:u32)] on the callable or #[ink(namespace = N:string)] \
            on the implementation block to disambiguate overlapping selectors.",
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
            hint: use #[ink(selector = S:u32)] on the callable or #[ink(namespace = N:string)] \
            on the implementation block to disambiguate overlapping selectors.",
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
                        #[ink(constructor, selector = 0xDEADBEEF)]
                        pub fn my_constructor() -> Self {}

                        #[ink(message, selector = 0xDEADBEEF)]
                        pub fn my_message(&self) {}
                    }
                }
            })
            .is_ok()
        );
    }

    #[test]
    fn overlapping_wildcard_selectors_fails() {
        assert_fail(
            syn::parse_quote! {
                mod my_module {
                    #[ink(storage)]
                    pub struct MyStorage {}

                    impl MyStorage {
                        #[ink(constructor)]
                        pub fn my_constructor() -> Self {}

                        #[ink(message, selector = _)]
                        pub fn my_message1(&self) {}

                        #[ink(message, selector = _)]
                        pub fn my_message2(&self) {}
                    }
                }
            },
            "encountered ink! messages with overlapping wildcard selectors",
        );
    }

    #[test]
    fn wildcard_selector_on_constructor_works() {
        assert!(
            <ir::ItemMod as TryFrom<syn::ItemMod>>::try_from(syn::parse_quote! {
                mod my_module {
                    #[ink(storage)]
                    pub struct MyStorage {}

                    impl MyStorage {
                        #[ink(constructor, selector = _)]
                        pub fn my_constructor() -> Self {}

                        #[ink(message)]
                        pub fn my_message(&self) {}
                    }
                }
            })
            .is_ok()
        );
    }

    #[test]
    fn overlap_between_wildcard_selector_and_composed_selector_fails() {
        assert_fail(
            syn::parse_quote! {
                mod my_module {
                    #[ink(storage)]
                    pub struct MyStorage {}

                    impl MyStorage {
                        #[ink(constructor)]
                        pub fn my_constructor() -> Self {}

                        #[ink(message, selector = _, selector = 0xCAFEBABE)]
                        pub fn my_message(&self) {}
                    }
                }
            },
            "encountered ink! attribute arguments with equal kinds",
        );
    }

    #[test]
    fn wildcard_selector_and_one_other_message_with_well_known_selector_works() {
        assert!(
            <ir::ItemMod as TryFrom<syn::ItemMod>>::try_from(syn::parse_quote! {
                mod my_module {
                    #[ink(storage)]
                    pub struct MyStorage {}

                    impl MyStorage {
                        #[ink(constructor)]
                        pub fn my_constructor() -> Self {}

                        #[ink(message, selector = _)]
                        pub fn fallback(&self) {}

                        #[ink(message, selector = 0x9BAE9D5E)]
                        pub fn wildcard_complement_message(&self) {}
                    }
                }
            })
            .is_ok()
        );
    }

    #[test]
    fn wildcard_selector_and_one_other_message_with_wildcard_complement_selector_works() {
        assert!(
            <ir::ItemMod as TryFrom<syn::ItemMod>>::try_from(syn::parse_quote! {
                mod my_module {
                    #[ink(storage)]
                    pub struct MyStorage {}

                    impl MyStorage {
                        #[ink(constructor)]
                        pub fn my_constructor() -> Self {}

                        #[ink(message, selector = _)]
                        pub fn fallback(&self) {}

                        #[ink(message, selector = @)]
                        pub fn wildcard_complement_message(&self) {}
                    }
                }
            })
            .is_ok()
        );
    }

    #[test]
    fn wildcard_selector_without_other_message_fails() {
        assert_fail(
            syn::parse_quote! {
                mod my_module {
                    #[ink(storage)]
                    pub struct MyStorage {}

                    impl MyStorage {
                        #[ink(constructor)]
                        pub fn my_constructor() -> Self {}

                        #[ink(message, selector = _)]
                        pub fn fallback(&self) {}
                    }
                }
            },
            "missing definition of another message with TODO in tandem with a wildcard selector",
        )
    }

    #[test]
    fn wildcard_selector_and_one_other_message_without_well_known_selector_fails() {
        assert_fail(
            syn::parse_quote! {
                mod my_module {
                    #[ink(storage)]
                    pub struct MyStorage {}

                    impl MyStorage {
                        #[ink(constructor)]
                        pub fn my_constructor() -> Self {}

                        #[ink(message, selector = _)]
                        pub fn fallback(&self) {}

                        #[ink(message)]
                        pub fn other_message_without_well_known_selector(&self) {}
                    }
                }
            },
            "when using a wildcard selector `selector = _` for an ink! message then the other \
            message must use the wildcard complement `selector = @`",
        );
    }

    #[test]
    fn wildcard_selector_with_two_other_messages() {
        assert_fail(
            syn::parse_quote! {
                mod my_module {
                    #[ink(storage)]
                    pub struct MyStorage {}

                    impl MyStorage {
                        #[ink(constructor)]
                        pub fn my_constructor() -> Self {}

                        #[ink(message, selector = _)]
                        pub fn fallback(&self) {}

                        #[ink(message, selector = 0x00000000)]
                        pub fn wildcard_complement_message(&self) {}

                        #[ink(message)]
                        pub fn another_message_not_allowed(&self) {}
                    }
                }
            },
            "exactly one other message must be defined together with a wildcard selector",
        );
    }

    #[test]
    fn wildcard_selector_with_many_other_messages() {
        assert_fail(
            syn::parse_quote! {
                mod my_module {
                    #[ink(storage)]
                    pub struct MyStorage {}

                    impl MyStorage {
                        #[ink(constructor)]
                        pub fn my_constructor() -> Self {}

                        #[ink(message, selector = _)]
                        pub fn fallback(&self) {}

                        #[ink(message, selector = @)]
                        pub fn wildcard_complement(&self) {}

                        #[ink(message)]
                        pub fn another_message_not_allowed1(&self) {}

                        #[ink(message)]
                        pub fn another_message_not_allowed2(&self) {}

                        #[ink(message)]
                        pub fn another_message_not_allowed3(&self) {}
                    }
                }
            },
            "exactly one other message must be defined together with a wildcard selector",
        );
    }

    #[test]
    fn wildcard_complement_used_without_wildcard_fails() {
        assert_fail(
            syn::parse_quote! {
                mod my_module {
                    #[ink(storage)]
                    pub struct MyStorage {}

                    impl MyStorage {
                        #[ink(constructor)]
                        pub fn my_constructor() -> Self {}

                        #[ink(message, selector = @)]
                        pub fn uses_reserved_wildcard_other_message_selector(&self) {}
                    }
                }
            },
            "encountered ink! message with wildcard complement `selector = @` but no \
            wildcard `selector = _` defined",
        )
    }

    #[test]
    fn wildcard_reserved_selector_used_without_wildcard_fails() {
        assert_fail(
            syn::parse_quote! {
                mod my_module {
                    #[ink(storage)]
                    pub struct MyStorage {}

                    impl MyStorage {
                        #[ink(constructor)]
                        pub fn my_constructor() -> Self {}

                        #[ink(message, selector = 0x9BAE9D5E)]
                        pub fn uses_reserved_wildcard_other_message_selector(&self) {}
                    }
                }
            },
            "encountered ink! message with wildcard complement `selector = @` but no \
            wildcard `selector = _` defined",
        )
    }

    #[test]
    fn cfg_feature_std_not_allowed() {
        let item_mod = syn::parse_quote! {
            mod my_module {
                #[ink(storage)]
                pub struct MyStorage {}

                impl MyStorage {
                    #[ink(constructor)]
                    pub fn my_constructor() -> Self {}

                    #[ink(message)]
                    #[cfg(feature = "std")]
                    pub fn not_allowed(&self) {}
                }
            }
        };
        let res = <ir::ItemMod as TryFrom<syn::ItemMod>>::try_from(item_mod)
            .map_err(|err| err.to_string());
        assert!(res.is_err());
        assert!(
            res.unwrap_err()
                .starts_with("The feature `std` is not allowed in `cfg`.")
        );
    }

    #[test]
    fn cfg_feature_other_than_std_allowed() {
        let item_mod = syn::parse_quote! {
            mod my_module {
                #[ink(storage)]
                pub struct MyStorage {}

                impl MyStorage {
                    #[ink(constructor)]
                    pub fn my_constructor() -> Self {}

                    #[ink(message)]
                    pub fn not_allowed(&self) {}
                }
            }
        };
        let res = <ir::ItemMod as TryFrom<syn::ItemMod>>::try_from(item_mod)
            .map_err(|err| err.to_string());
        assert!(res.is_ok());
    }

    #[test]
    fn cfg_test_allowed() {
        let item_mod = syn::parse_quote! {
            mod my_module {
                #[ink(storage)]
                pub struct MyStorage {}

                impl MyStorage {
                    #[ink(constructor)]
                    pub fn my_constructor() -> Self {}

                    #[ink(message)]
                    #[cfg(test)]
                    pub fn not_allowed(&self) {}
                }
            }
        };
        let res = <ir::ItemMod as TryFrom<syn::ItemMod>>::try_from(item_mod)
            .map_err(|err| err.to_string());
        assert!(res.is_ok());
    }

    #[test]
    fn cfg_nested_forbidden_must_be_found() {
        let item_mod = syn::parse_quote! {
            mod my_module {
                #[ink(storage)]
                pub struct MyStorage {}

                impl MyStorage {
                    #[ink(constructor)]
                    #[cfg(any(not(target_os = "wasm")))]
                    pub fn my_constructor() -> Self {}

                    #[ink(message)]
                    pub fn not_allowed(&self) {}
                }
            }
        };
        let res = <ir::ItemMod as TryFrom<syn::ItemMod>>::try_from(item_mod)
            .map_err(|err| err.to_string());
        assert!(res.is_err());
        assert!(
            res.unwrap_err()
                .starts_with("This `cfg` attribute is not allowed.")
        );
    }
}
