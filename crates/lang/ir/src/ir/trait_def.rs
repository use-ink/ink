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
    TokenStream as TokenStream2,
};
use syn::{
    spanned::Spanned as _,
    Result,
};

/// A checked ink! trait definition.
#[derive(Debug, PartialEq, Eq)]
pub struct InkTrait {
    item: syn::ItemTrait,
}

impl TryFrom<syn::ItemTrait> for InkTrait {
    type Error = syn::Error;

    fn try_from(item_trait: syn::ItemTrait) -> core::result::Result<Self, Self::Error> {
        idents_lint::ensure_no_ink_identifiers(&item_trait)?;
        Self::analyse_properties(&item_trait)?;
        Self::analyse_items(&item_trait)?;
        Ok(Self { item: item_trait })
    }
}

impl InkTrait {
    /// Returns the hash to verify that the trait definition has been checked.
    pub fn compute_verify_hash<C, M>(
        trait_name: &Ident,
        constructors: C,
        messages: M,
    ) -> [u8; 32]
    where
        // Name and number of inputs.
        C: Iterator<Item = (Ident, usize)>,
        // Name, number of inputs and true if message may mutate storage.
        M: Iterator<Item = (Ident, usize, bool)>,
    {
        let mut constructors = constructors
            .map(|(name, len_inputs)| {
                [name.to_string(), len_inputs.to_string()].join(":")
            })
            .collect::<Vec<_>>();
        let mut messages = messages
            .map(|(name, len_inputs, mutability)| {
                let mutability = match mutability {
                    true => "w",
                    false => "r",
                };
                [
                    name.to_string(),
                    len_inputs.to_string(),
                    mutability.to_string(),
                ]
                .join(":")
            })
            .collect::<Vec<_>>();
        constructors.sort_unstable();
        messages.sort_unstable();
        let joined_constructors = constructors.join(",");
        let joined_messages = messages.join(",");
        let mut buffer = vec!["__ink_trait".to_string(), trait_name.to_string()];
        if !joined_constructors.is_empty() {
            buffer.push(joined_constructors);
        }
        if !joined_messages.is_empty() {
            buffer.push(joined_messages);
        }
        let buffer = buffer.join("::").into_bytes();
        use blake2::digest::generic_array::sequence::Split as _;
        let (head_32, _rest) =
            <blake2::Blake2b as blake2::Digest>::digest(&buffer).split();
        head_32.into()
    }

    /// Returns the hash to verify that the trait definition has been checked.
    pub fn verify_hash(&self) -> [u8; 32] {
        let trait_name = self.ident();
        Self::compute_verify_hash(
            trait_name,
            self.iter_items()
                .flat_map(InkTraitItem::filter_map_constructor)
                .map(|constructor| {
                    let name = constructor.sig().ident.clone();
                    let len_inputs = constructor.sig().inputs.len();
                    (name, len_inputs)
                }),
            self.iter_items()
                .flat_map(InkTraitItem::filter_map_message)
                .map(|message| {
                    let name = message.sig().ident.clone();
                    let len_inputs = message.sig().inputs.len();
                    let mutability = message.mutates();
                    (name, len_inputs, mutability)
                }),
        )
    }
}

/// Iterator over all the ink! trait items of an ink! trait definition.
pub struct IterInkTraitItems<'a> {
    iter: core::slice::Iter<'a, syn::TraitItem>,
}

impl<'a> IterInkTraitItems<'a> {
    /// Creates a new iterator yielding ink! trait items.
    fn new(item_trait: &'a InkTrait) -> Self {
        Self {
            iter: item_trait.item.items.iter(),
        }
    }
}

impl<'a> Iterator for IterInkTraitItems<'a> {
    type Item = InkTraitItem<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        'outer: loop {
            match self.iter.next() {
                None => return None,
                Some(syn::TraitItem::Method(method)) => {
                    let first_attr = ir::first_ink_attribute(&method.attrs)
                        .ok()
                        .flatten()
                        .expect("unexpected missing ink! attribute for trait method")
                        .first()
                        .kind()
                        .clone();
                    match first_attr {
                        ir::AttributeArg::Constructor => {
                            return Some(InkTraitItem::Constructor(InkTraitConstructor {
                                item: method,
                            }))
                        }
                        ir::AttributeArg::Message => {
                            return Some(InkTraitItem::Message(InkTraitMessage {
                                item: method,
                            }))
                        }
                        _ => continue 'outer,
                    }
                }
                Some(_) => continue 'outer,
            }
        }
    }
}

/// An ink! item within an ink! trait definition.
#[derive(Debug, Clone)]
pub enum InkTraitItem<'a> {
    Constructor(InkTraitConstructor<'a>),
    Message(InkTraitMessage<'a>),
}

impl<'a> InkTraitItem<'a> {
    /// Returns `Some` if the ink! trait item is a constructor.
    pub fn filter_map_constructor(self) -> Option<InkTraitConstructor<'a>> {
        match self {
            Self::Constructor(ink_trait_constructor) => Some(ink_trait_constructor),
            _ => None,
        }
    }

    /// Returns `Some` if the ink! trait item is a message.
    pub fn filter_map_message(self) -> Option<InkTraitMessage<'a>> {
        match self {
            Self::Message(ink_trait_message) => Some(ink_trait_message),
            _ => None,
        }
    }
}

/// Returns all non-ink! attributes.
///
/// # Panics
///
/// If there are malformed ink! attributes in the input.
fn extract_rust_attributes(attributes: &[syn::Attribute]) -> Vec<syn::Attribute> {
    let (_ink_attrs, rust_attrs) = ir::partition_attributes(attributes.to_vec())
        .expect("encountered unexpected invalid ink! attributes");
    rust_attrs
}

/// A checked ink! constructor of an ink! trait definition.
#[derive(Debug, Clone)]
pub struct InkTraitConstructor<'a> {
    item: &'a syn::TraitItemMethod,
}

impl<'a> InkTraitConstructor<'a> {
    /// Returns all non-ink! attributes.
    pub fn attrs(&self) -> Vec<syn::Attribute> {
        extract_rust_attributes(&self.item.attrs)
    }

    /// Returns the original signature of the ink! constructor.
    pub fn sig(&self) -> &syn::Signature {
        &self.item.sig
    }

    /// Returns the span of the ink! constructor.
    pub fn span(&self) -> Span {
        self.item.span()
    }
}

/// A checked ink! message of an ink! trait definition.
#[derive(Debug, Clone)]
pub struct InkTraitMessage<'a> {
    item: &'a syn::TraitItemMethod,
}

impl<'a> InkTraitMessage<'a> {
    /// Returns all non-ink! attributes.
    pub fn attrs(&self) -> Vec<syn::Attribute> {
        extract_rust_attributes(&self.item.attrs)
    }

    /// Returns the original signature of the ink! message.
    pub fn sig(&self) -> &syn::Signature {
        &self.item.sig
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

impl InkTrait {
    /// Returns `Ok` if the trait matches all requirements for an ink! trait definition.
    pub fn new(attr: TokenStream2, input: TokenStream2) -> Result<Self> {
        if !attr.is_empty() {
            return Err(format_err_spanned!(
                attr,
                "unexpected attribute input for ink! trait definition"
            ))
        }
        let item_trait = syn::parse2::<syn::ItemTrait>(input)?;
        InkTrait::try_from(item_trait)
    }

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

    /// Returns an iterator yielding the ink! specific items of the ink! trait definition.
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
                "ink! trait definitions with super-traits are not supported, yet"
            ))
        }
        Ok(())
    }

    /// Returns `Ok` if all trait items respects the requirements for an ink! trait definition.
    ///
    /// # Errors
    ///
    /// - If the trait contains an unsupported trait item such as
    ///     - associated constants (`const`)
    ///     - associated types (`type`)
    ///     - macros definitions or usages
    ///     - unknown token sequences (`Verbatim`'s)
    ///     - methods with default implementations
    /// - If the trait contains methods which do not respect the ink! trait definition requirements:
    ///     - All trait methods need to be declared as either `#[ink(message)]` or `#[ink(constructor)]`
    ///       and need to respect their respective rules.
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
                    Self::analyse_methods(method_trait_item)?;
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

    /// Analyses an ink! method that can be either an ink! message or constructor.
    ///
    /// # Errors
    ///
    /// - If the method declared as `unsafe`, `const` or `async`.
    /// - If the method has some explicit API.
    /// - If the method is variadic or has generic parameters.
    /// - If the method does not respect the properties of either an
    ///   ink! message or ink! constructor.
    fn analyse_methods(method: &syn::TraitItemMethod) -> Result<()> {
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
                        Self::analyse_message(method)?;
                    }
                    ir::AttributeArg::Constructor => {
                        Self::analyse_constructor(method)?;
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

    /// Analyses the properties of an ink! constructor.
    ///
    /// # Errors
    ///
    /// - If the constructor has a `self` receiver as first argument.
    /// - If the constructor has no `Self` return type.
    fn analyse_constructor(constructor: &syn::TraitItemMethod) -> Result<()> {
        ir::sanitize_attributes(
            constructor.span(),
            constructor.attrs.clone(),
            &ir::AttributeArgKind::Constructor,
            |arg| {
                match arg.kind() {
                    ir::AttributeArg::Constructor => Ok(()),
                    _ => Err(None),
                }
            },
        )?;
        if let Some(receiver) = constructor.sig.receiver() {
            return Err(format_err_spanned!(
                receiver,
                "ink! constructors must not have a `self` receiver",
            ))
        }
        match &constructor.sig.output {
            syn::ReturnType::Default => {
                return Err(format_err_spanned!(
                    constructor.sig,
                    "ink! constructors must return Self"
                ))
            }
            syn::ReturnType::Type(_, ty) => {
                match &**ty {
                    syn::Type::Path(type_path) => {
                        if !type_path.path.is_ident("Self") {
                            return Err(format_err_spanned!(
                                type_path.path,
                                "ink! constructors must return Self"
                            ))
                        }
                    }
                    unknown => {
                        return Err(format_err_spanned!(
                            unknown,
                            "ink! constructors must return Self"
                        ))
                    }
                }
            }
        }
        Ok(())
    }

    /// Analyses the properties of an ink! message.
    ///
    /// # Errors
    ///
    /// - If the message has no `&self` or `&mut self` receiver.
    fn analyse_message(message: &syn::TraitItemMethod) -> Result<()> {
        ir::sanitize_attributes(
            message.span(),
            message.attrs.clone(),
            &ir::AttributeArgKind::Message,
            |arg| {
                match arg.kind() {
                    ir::AttributeArg::Message => Ok(()),
                    _ => Err(None),
                }
            },
        )?;
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
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Checks if the token stream in `$trait_def` results in the expected error message.
    macro_rules! assert_ink_trait_eq_err {
        ( error: $err_str:literal, $($trait_def:tt)* ) => {
            assert_eq!(
                <InkTrait as TryFrom<syn::ItemTrait>>::try_from(syn::parse_quote! {
                    $( $trait_def )*
                })
                .map_err(|err| err.to_string()),
                Err(
                    $err_str.to_string()
                )
            )
        };
    }

    #[test]
    fn unsafe_trait_def_is_denied() {
        assert_ink_trait_eq_err!(
            error: "ink! trait definitions cannot be unsafe",
            pub unsafe trait MyTrait {}
        );
    }

    #[test]
    fn auto_trait_def_is_denied() {
        assert_ink_trait_eq_err!(
            error: "ink! trait definitions cannot be automatically implemented traits",
            pub auto trait MyTrait {}
        );
    }

    #[test]
    fn non_pub_trait_def_is_denied() {
        assert_ink_trait_eq_err!(
            error: "ink! trait definitions must have public visibility",
            trait MyTrait {}
        );
        assert_ink_trait_eq_err!(
            error: "ink! trait definitions must have public visibility",
            pub(crate) trait MyTrait {}
        );
    }

    #[test]
    fn generic_trait_def_is_denied() {
        assert_ink_trait_eq_err!(
            error: "ink! trait definitions must not be generic",
            pub trait MyTrait<T> {}
        );
    }

    #[test]
    fn trait_def_with_supertraits_is_denied() {
        assert_ink_trait_eq_err!(
            error: "ink! trait definitions with super-traits are not supported, yet",
            pub trait MyTrait: SuperTrait {}
        );
    }

    #[test]
    fn trait_def_containing_const_item_is_denied() {
        assert_ink_trait_eq_err!(
            error: "associated constants in ink! trait definitions are not supported, yet",
            pub trait MyTrait {
                const T: i32;
            }
        );
    }

    #[test]
    fn trait_def_containing_associated_type_is_denied() {
        assert_ink_trait_eq_err!(
            error: "associated types in ink! trait definitions are not supported, yet",
            pub trait MyTrait {
                type Type;
            }
        );
    }

    #[test]
    fn trait_def_containing_macro_is_denied() {
        assert_ink_trait_eq_err!(
            error: "macros in ink! trait definitions are not supported",
            pub trait MyTrait {
                my_macro_call!();
            }
        );
    }

    #[test]
    fn trait_def_containing_non_flagged_method_is_denied() {
        assert_ink_trait_eq_err!(
            error: "missing #[ink(message)] or #[ink(constructor)] flags on ink! trait method",
            pub trait MyTrait {
                fn non_flagged_1(&self);
            }
        );
        assert_ink_trait_eq_err!(
            error: "missing #[ink(message)] or #[ink(constructor)] flags on ink! trait method",
            pub trait MyTrait {
                fn non_flagged_2(&mut self);
            }
        );
        assert_ink_trait_eq_err!(
            error: "missing #[ink(message)] or #[ink(constructor)] flags on ink! trait method",
            pub trait MyTrait {
                fn non_flagged_3() -> Self;
            }
        );
    }

    #[test]
    fn trait_def_containing_default_implemented_methods_is_denied() {
        assert_ink_trait_eq_err!(
            error: "ink! trait methods with default implementations are not supported",
            pub trait MyTrait {
                #[ink(constructor)]
                fn default_implemented() -> Self {}
            }
        );
        assert_ink_trait_eq_err!(
            error: "ink! trait methods with default implementations are not supported",
            pub trait MyTrait {
                #[ink(message)]
                fn default_implemented(&self) {}
            }
        );
    }

    #[test]
    fn trait_def_containing_const_methods_is_denied() {
        assert_ink_trait_eq_err!(
            error: "const ink! trait methods are not supported",
            pub trait MyTrait {
                #[ink(constructor)]
                const fn const_constructor() -> Self;
            }
        );
        assert_ink_trait_eq_err!(
            error: "const ink! trait methods are not supported",
            pub trait MyTrait {
                #[ink(message)]
                const fn const_message(&self);
            }
        );
    }

    #[test]
    fn trait_def_containing_async_methods_is_denied() {
        assert_ink_trait_eq_err!(
            error: "async ink! trait methods are not supported",
            pub trait MyTrait {
                #[ink(constructor)]
                async fn const_constructor() -> Self;
            }
        );
        assert_ink_trait_eq_err!(
            error: "async ink! trait methods are not supported",
            pub trait MyTrait {
                #[ink(message)]
                async fn const_message(&self);
            }
        );
    }

    #[test]
    fn trait_def_containing_unsafe_methods_is_denied() {
        assert_ink_trait_eq_err!(
            error: "unsafe ink! trait methods are not supported",
            pub trait MyTrait {
                #[ink(constructor)]
                unsafe fn const_constructor() -> Self;
            }
        );
        assert_ink_trait_eq_err!(
            error: "unsafe ink! trait methods are not supported",
            pub trait MyTrait {
                #[ink(message)]
                unsafe fn const_message(&self);
            }
        );
    }

    #[test]
    fn trait_def_containing_methods_using_explicit_abi_is_denied() {
        assert_ink_trait_eq_err!(
            error: "ink! trait methods with non default ABI are not supported",
            pub trait MyTrait {
                #[ink(constructor)]
                extern fn const_constructor() -> Self;
            }
        );
        assert_ink_trait_eq_err!(
            error: "ink! trait methods with non default ABI are not supported",
            pub trait MyTrait {
                #[ink(message)]
                extern fn const_message(&self);
            }
        );
    }

    #[test]
    fn trait_def_containing_variadic_methods_is_denied() {
        assert_ink_trait_eq_err!(
            error: "variadic ink! trait methods are not supported",
            pub trait MyTrait {
                #[ink(constructor)]
                fn const_constructor(...) -> Self;
            }
        );
        assert_ink_trait_eq_err!(
            error: "variadic ink! trait methods are not supported",
            pub trait MyTrait {
                #[ink(message)]
                fn const_message(&self, ...);
            }
        );
    }

    #[test]
    fn trait_def_containing_generic_methods_is_denied() {
        assert_ink_trait_eq_err!(
            error: "generic ink! trait methods are not supported",
            pub trait MyTrait {
                #[ink(constructor)]
                fn const_constructor<T>() -> Self;
            }
        );
        assert_ink_trait_eq_err!(
            error: "generic ink! trait methods are not supported",
            pub trait MyTrait {
                #[ink(message)]
                fn const_message<T>(&self);
            }
        );
    }

    #[test]
    fn trait_def_containing_method_with_unsupported_ink_attribute_is_denied() {
        assert_ink_trait_eq_err!(
            error: "encountered unsupported ink! attribute for ink! trait method",
            pub trait MyTrait {
                #[ink(payable)]
                fn unsupported_ink_attribute(&self);
            }
        );
        assert_ink_trait_eq_err!(
            error: "unknown ink! attribute (path)",
            pub trait MyTrait {
                #[ink(unknown)]
                fn unknown_ink_attribute(&self);
            }
        );
    }

    #[test]
    fn trait_def_containing_invalid_constructor_is_denied() {
        assert_ink_trait_eq_err!(
            error: "ink! constructors must not have a `self` receiver",
            pub trait MyTrait {
                #[ink(constructor)]
                fn has_self_receiver(&self) -> Self;
            }
        );
        assert_ink_trait_eq_err!(
            error: "ink! constructors must not have a `self` receiver",
            pub trait MyTrait {
                #[ink(constructor)]
                fn has_self_receiver(&mut self) -> Self;
            }
        );
        assert_ink_trait_eq_err!(
            error: "ink! constructors must not have a `self` receiver",
            pub trait MyTrait {
                #[ink(constructor)]
                fn has_self_receiver(self) -> Self;
            }
        );
        assert_ink_trait_eq_err!(
            error: "ink! constructors must not have a `self` receiver",
            pub trait MyTrait {
                #[ink(constructor)]
                fn has_self_receiver(self: &Self) -> Self;
            }
        );
        assert_ink_trait_eq_err!(
            error: "ink! constructors must not have a `self` receiver",
            pub trait MyTrait {
                #[ink(constructor)]
                fn has_self_receiver(self: Self) -> Self;
            }
        );
        assert_ink_trait_eq_err!(
            error: "ink! constructors must return Self",
            pub trait MyTrait {
                #[ink(constructor)]
                fn does_not_return_self();
            }
        );
        assert_ink_trait_eq_err!(
            error: "ink! constructors must return Self",
            pub trait MyTrait {
                #[ink(constructor)]
                fn does_not_return_self() -> i32;
            }
        );
    }

    #[test]
    fn trait_def_containing_invalid_message_is_denied() {
        assert_ink_trait_eq_err!(
            error: "missing or malformed `&self` or `&mut self` receiver for ink! message",
            pub trait MyTrait {
                #[ink(message)]
                fn does_not_return_self();
            }
        );
        assert_ink_trait_eq_err!(
            error: "missing or malformed `&self` or `&mut self` receiver for ink! message",
            pub trait MyTrait {
                #[ink(message)]
                fn does_not_return_self(self: &Self);
            }
        );
        assert_ink_trait_eq_err!(
            error: "self receiver of ink! message must be `&self` or `&mut self`",
            pub trait MyTrait {
                #[ink(message)]
                fn does_not_return_self(self);
            }
        );
    }

    #[test]
    fn trait_def_containing_constructor_with_invalid_ink_attributes_is_denied() {
        assert_ink_trait_eq_err!(
            error: "encountered duplicate ink! attribute",
            pub trait MyTrait {
                #[ink(constructor)]
                #[ink(constructor)]
                fn does_not_return_self() -> Self;
            }
        );
        assert_ink_trait_eq_err!(
            error: "encountered conflicting ink! attribute argument",
            pub trait MyTrait {
                #[ink(constructor)]
                #[ink(message)]
                fn does_not_return_self() -> Self;
            }
        );
        assert_ink_trait_eq_err!(
            error: "encountered conflicting ink! attribute argument",
            pub trait MyTrait {
                #[ink(constructor)]
                #[ink(payable)]
                fn does_not_return_self() -> Self;
            }
        );
    }

    #[test]
    fn trait_def_containing_message_with_invalid_ink_attributes_is_denied() {
        assert_ink_trait_eq_err!(
            error: "encountered duplicate ink! attribute",
            pub trait MyTrait {
                #[ink(message)]
                #[ink(message)]
                fn does_not_return_self(&self);
            }
        );
        assert_ink_trait_eq_err!(
            error: "encountered conflicting ink! attribute argument",
            pub trait MyTrait {
                #[ink(message)]
                #[ink(constructor)]
                fn does_not_return_self(&self);
            }
        );
        assert_ink_trait_eq_err!(
            error: "encountered conflicting ink! attribute argument",
            pub trait MyTrait {
                #[ink(message)]
                #[ink(payable)]
                fn does_not_return_self(&self);
            }
        );
    }

    #[test]
    fn trait_def_is_ok() {
        assert!(
            <InkTrait as TryFrom<syn::ItemTrait>>::try_from(syn::parse_quote! {
                pub trait MyTrait {
                    #[ink(constructor)]
                    fn my_constructor() -> Self;
                    #[ink(message)]
                    fn my_message(&self);
                    #[ink(message)]
                    fn my_message_mut(&mut self);
                }
            })
            .is_ok()
        )
    }

    #[test]
    fn iter_constructors_works() {
        let ink_trait =
            <InkTrait as TryFrom<syn::ItemTrait>>::try_from(syn::parse_quote! {
                pub trait MyTrait {
                    #[ink(constructor)]
                    fn constructor_1() -> Self;
                    #[ink(constructor)]
                    fn constructor_2() -> Self;
                    #[ink(message)]
                    fn message_1(&self);
                    #[ink(message)]
                    fn message_2(&mut self);
                 }
            })
            .unwrap();
        let actual = ink_trait
            .iter_items()
            .flat_map(|item| {
                item.filter_map_constructor()
                    .map(|constructor| constructor.sig().ident.to_string())
            })
            .collect::<Vec<_>>();
        let expected = vec!["constructor_1".to_string(), "constructor_2".to_string()];
        assert_eq!(actual, expected);
    }

    #[test]
    fn iter_messages_works() {
        let ink_trait =
            <InkTrait as TryFrom<syn::ItemTrait>>::try_from(syn::parse_quote! {
                pub trait MyTrait {
                    #[ink(constructor)]
                    fn constructor_1() -> Self;
                    #[ink(constructor)]
                    fn constructor_2() -> Self;
                    #[ink(message)]
                    fn message_1(&self);
                    #[ink(message)]
                    fn message_2(&mut self);
                }
            })
            .unwrap();
        let actual = ink_trait
            .iter_items()
            .flat_map(|item| {
                item.filter_map_message()
                    .map(|message| message.sig().ident.to_string())
            })
            .collect::<Vec<_>>();
        let expected = vec!["message_1".to_string(), "message_2".to_string()];
        assert_eq!(actual, expected);
    }

    fn assert_verify_hash2_works_with(ink_trait: InkTrait, expected: &str) {
        let expected = expected.to_string().into_bytes();
        let actual = ink_trait.verify_hash();
        let expected: [u8; 32] = {
            use blake2::digest::generic_array::sequence::Split as _;
            let (head_32, _rest) =
                <blake2::Blake2b as blake2::Digest>::digest(&expected).split();
            head_32.into()
        };
        assert_eq!(actual, expected);
    }

    macro_rules! ink_trait {
        ( $($tt:tt)* ) => {{
            <InkTrait as TryFrom<syn::ItemTrait>>::try_from(syn::parse_quote! {
                $( $tt )*
            })
            .unwrap()
        }};
    }

    #[test]
    fn verify_hash_works() {
        let ink_trait = ink_trait! {
            pub trait MyTrait {
                #[ink(constructor)]
                fn constructor_1() -> Self;
                #[ink(constructor)]
                fn constructor_2(a: i32, b: i32) -> Self;
                #[ink(message)]
                fn message_1(&self);
                #[ink(message)]
                fn message_2(&mut self, a: i32, b: i32) -> i32;
            }
        };
        assert_verify_hash2_works_with(
            ink_trait,
            "__ink_trait::MyTrait::constructor_1:0,constructor_2:2::message_1:1:r,message_2:3:w"
        );
    }

    #[test]
    fn verify_hash_works_without_constructors() {
        let ink_trait = ink_trait! {
            pub trait MyTrait {
                #[ink(message)]
                fn message_1(&self);
                #[ink(message)]
                fn message_2(&mut self, a: i32, b: i32) -> i32;
            }
        };
        assert_verify_hash2_works_with(
            ink_trait,
            "__ink_trait::MyTrait::message_1:1:r,message_2:3:w",
        );
    }

    #[test]
    fn verify_hash_works_without_messages() {
        let ink_trait = ink_trait! {
            pub trait MyTrait {
                #[ink(constructor)]
                fn constructor_1() -> Self;
                #[ink(constructor)]
                fn constructor_2(a: i32, b: i32) -> Self;
            }
        };
        assert_verify_hash2_works_with(
            ink_trait,
            "__ink_trait::MyTrait::constructor_1:0,constructor_2:2",
        );
    }
}
