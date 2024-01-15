// Copyright (C) Parity Technologies (UK) Ltd.
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
    ast,
    error::ExtError,
    ir,
    ir::idents_lint,
};
use core::slice::Iter as SliceIter;
use proc_macro2::TokenStream as TokenStream2;
use std::collections::HashMap;
use syn::{
    spanned::Spanned as _,
    Result,
};

/// An ink! chain extension.
#[derive(Debug, PartialEq, Eq)]
pub struct ChainExtension {
    item: syn::ItemTrait,
    config: Config,
    error_code: syn::TraitItemType,
    methods: Vec<ChainExtensionMethod>,
}

impl ChainExtension {
    /// Returns the Rust attributes of the ink! chain extension.
    pub fn attrs(&self) -> Vec<syn::Attribute> {
        let (_, attrs) = ir::partition_attributes(self.item.attrs.iter().cloned())
            .unwrap_or_else(|err| panic!("encountered unexpected invalid attributes for ink! chain extension: {err}"));
        attrs
    }

    /// Returns the span of the ink! chain extension.
    pub fn span(&self) -> proc_macro2::Span {
        self.item.span()
    }

    /// Returns the identifier of the ink! chain extension.
    pub fn ident(&self) -> &proc_macro2::Ident {
        &self.item.ident
    }

    /// Returns a slice over all the chain extension methods.
    pub fn iter_methods(&self) -> SliceIter<ChainExtensionMethod> {
        self.methods.iter()
    }

    /// Returns the type of the error code of the chain extension.
    pub fn error_code(&self) -> &syn::Type {
        self.error_code
            .default
            .as_ref()
            .map(|(_token, ty)| ty)
            .expect("unexpected missing default type for error code")
    }
}

/// The chain extension configuration.
#[derive(Default, Debug, PartialEq, Eq)]
pub struct Config {
    ext_id: ExtensionId,
}

impl TryFrom<ast::AttributeArgs> for Config {
    type Error = syn::Error;

    fn try_from(args: ast::AttributeArgs) -> Result<Self> {
        let mut ext_id: Option<ExtensionId> = None;

        for arg in args.clone().into_iter() {
            if arg.name.is_ident("extension") {
                if ext_id.is_some() {
                    return Err(format_err_spanned!(
                        arg.value,
                        "encountered duplicate ink! contract `extension` configuration argument",
                    ))
                }

                if let Some(lit_int) = arg.value.as_lit_int() {
                    let id = lit_int.base10_parse::<u16>()
                        .map_err(|error| {
                            format_err_spanned!(
                                        lit_int,
                                        "could not parse `N` in `extension = N` into a `u16` integer: {}", error)
                        })?;
                    ext_id = Some(ExtensionId::from_u16(id));
                } else {
                    return Err(format_err_spanned!(
                        arg.value,
                        "expected `u16` integer type for `N` in `extension = N`",
                    ))
                }
            } else {
                return Err(format_err_spanned!(
                    arg,
                    "encountered unknown or unsupported chain extension configuration argument",
                ))
            }
        }

        if let Some(ext_id) = ext_id {
            Ok(Config { ext_id })
        } else {
            Err(format_err_spanned!(
                args,
                "missing required `extension = N: u16` argument on ink! chain extension",
            ))
        }
    }
}

impl Config {
    /// Returns the chain extension identifier.
    pub fn ext_id(&self) -> ExtensionId {
        self.ext_id
    }
}

/// An ink! chain extension method.
#[derive(Debug, PartialEq, Eq)]
pub struct ChainExtensionMethod {
    /// The underlying validated AST of the chain extension method.
    item: syn::TraitItemFn,
    /// The unique identifier of the chain extension method.
    id: GlobalMethodId,
    /// If `false` the `u32` status code of the chain extension method call is going to
    /// be ignored and assumed to be always successful. The output buffer in this
    /// case is going to be queried and decoded into the chain extension method's
    /// output type.
    ///
    /// If `true` the returned `u32` status code `code` is queried and
    /// `<Self::ErrorCode as ink::FromStatusCode>::from_status_code(code)` is called.
    /// The call to `from_status_code` returns `Result<(), Self::ErrorCode>`. If `Ok(())`
    /// the output buffer is queried and decoded as described above.
    /// If `Err(Self::ErrorCode)` the `Self::ErrorCode` is converted into `E` of
    /// `Result<T, E>` if the chain extension method returns a `Result` type.
    /// In case the chain extension method does _NOT_ return a `Result` type the call
    /// returns `Result<T, Self::ErrorCode>` where `T` is the chain extension
    /// method's return type.
    ///
    /// The default for this flag is `true`.
    handle_status: bool,
}

impl ChainExtensionMethod {
    /// Returns the Rust attributes of the ink! chain extension method.
    pub fn attrs(&self) -> Vec<syn::Attribute> {
        let (_, attrs) = ir::partition_attributes(self.item.attrs.iter().cloned())
            .expect(
            "encountered unexpected invalid attributes for ink! chain extension method",
        );
        attrs
    }

    /// Returns the span of the ink! chain extension method.
    pub fn span(&self) -> proc_macro2::Span {
        self.item.span()
    }

    /// Returns the identifier of the ink! chain extension method.
    pub fn ident(&self) -> &proc_macro2::Ident {
        &self.item.sig.ident
    }

    /// Returns the method signature of the ink! chain extension method.
    pub fn sig(&self) -> &syn::Signature {
        &self.item.sig
    }

    /// Returns the unique ID of the chain extension method.
    pub fn id(&self) -> GlobalMethodId {
        self.id
    }

    /// Returns an iterator over the inputs of the chain extension method.
    pub fn inputs(&self) -> ChainExtensionMethodInputs {
        ChainExtensionMethodInputs {
            iter: self.item.sig.inputs.iter(),
        }
    }

    /// Returns `true` if the chain extension method was flagged with
    /// `#[ink(handle_status)]`.
    pub fn handle_status(&self) -> bool {
        self.handle_status
    }
}

pub struct ChainExtensionMethodInputs<'a> {
    iter: syn::punctuated::Iter<'a, syn::FnArg>,
}

impl<'a> Iterator for ChainExtensionMethodInputs<'a> {
    type Item = &'a syn::PatType;

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.iter.next()?;
        match item {
            syn::FnArg::Receiver(receiver) => {
                panic!("encountered unexpected receiver in chain extension method input: {receiver:?}")
            }
            syn::FnArg::Typed(pat_type) => Some(pat_type),
        }
    }
}

/// The unique ID of an chain extension.
///
/// # Note
///
/// The ink! attribute `#[ink::chain_extension(extension = N: u16)]` for chain extension.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ExtensionId {
    index: u16,
}

impl ExtensionId {
    /// Creates a new chain extension ID from the given `u16`.
    pub fn from_u16(index: u16) -> Self {
        Self { index }
    }

    /// Returns the underlying raw `u16` index.
    pub fn into_u16(self) -> u16 {
        self.index
    }
}

/// The unique ID of the method within the chain extension.
///
/// # Note
///
/// The ink! attribute `#[ink(function = N: u16)]` for chain extension methods.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FunctionId {
    index: u16,
}

impl FunctionId {
    /// Creates a new chain extension function ID from the given `u16`.
    pub fn from_u16(index: u16) -> Self {
        Self { index }
    }

    /// Returns the underlying raw `u16` index.
    pub fn into_u16(self) -> u16 {
        self.index
    }
}

/// The unique ID of a chain extension method across all chain extensions.
///
/// # Note
///
/// It is a combination of the [`ExtensionId`] and [`FunctionId`].
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GlobalMethodId {
    index: u32,
}

impl GlobalMethodId {
    /// Creates a new chain extension method global ID.
    pub fn new(ext_id: ExtensionId, func_id: FunctionId) -> Self {
        Self {
            index: ((ext_id.index as u32) << 16) | func_id.index as u32,
        }
    }

    /// Returns the identifier of the function within the chain extension.
    pub fn func_id(&self) -> FunctionId {
        FunctionId::from_u16((self.index & 0x0000FFFF) as u16)
    }

    /// Returns the identifier of the chain extension.
    pub fn ext_id(&self) -> ExtensionId {
        ExtensionId::from_u16((self.index >> 16) as u16)
    }

    /// Returns the underlying raw `u32` index.
    pub fn into_u32(self) -> u32 {
        self.index
    }
}

impl ChainExtension {
    /// Creates a new ink! chain extension from the given configuration and trait item.
    ///
    /// # Errors
    ///
    /// Returns an error if some of the chain extension rules are violated.
    pub fn try_from(
        item_trait: syn::ItemTrait,
        config: Config,
    ) -> core::result::Result<Self, syn::Error> {
        idents_lint::ensure_no_ink_identifiers(&item_trait)?;
        Self::analyse_properties(&item_trait)?;
        let (error_code, methods) = Self::analyse_items(config.ext_id, &item_trait)?;
        Ok(Self {
            item: item_trait,
            config,
            error_code,
            methods,
        })
    }
}

impl ChainExtension {
    /// Returns `Ok` if the trait matches all requirements for an ink! chain extension.
    pub fn new(attr: TokenStream2, input: TokenStream2) -> Result<Self> {
        let args = syn::parse2::<ast::AttributeArgs>(attr)?;
        let config = Config::try_from(args)?;
        let item_trait = syn::parse2::<syn::ItemTrait>(input)?;
        ChainExtension::try_from(item_trait, config)
    }

    /// Analyses the properties of the ink! chain extension.
    ///
    /// # Errors
    ///
    /// - If the input trait has been defined as `unsafe`.
    /// - If the input trait is an automatically implemented trait (`auto trait`).
    /// - If the input trait is generic over some set of types.
    /// - If the input trait's visibility is not public (`pub`).
    /// - If the input trait has super-traits.
    fn analyse_properties(item_trait: &syn::ItemTrait) -> Result<()> {
        if let Some(unsafety) = &item_trait.unsafety {
            return Err(format_err_spanned!(
                unsafety,
                "ink! chain extensions cannot be unsafe"
            ))
        }
        if let Some(auto) = &item_trait.auto_token {
            return Err(format_err_spanned!(
                auto,
                "ink! chain extensions cannot be automatically implemented traits"
            ))
        }
        if !item_trait.generics.params.is_empty() {
            return Err(format_err_spanned!(
                item_trait.generics.params,
                "ink! chain extensions must not be generic"
            ))
        }
        if !matches!(item_trait.vis, syn::Visibility::Public(_)) {
            return Err(format_err_spanned!(
                item_trait.vis,
                "ink! chain extensions must have public visibility"
            ))
        }
        if !item_trait.supertraits.is_empty() {
            return Err(format_err_spanned!(
                item_trait.supertraits,
                "ink! chain extensions with super-traits are not supported, yet"
            ))
        }
        Ok(())
    }

    /// Checks if the associated trait item type is a proper chain extension error code.
    ///
    /// # Errors
    ///
    /// - If the associated type is not called `ErrorCode`.
    /// - If the associated type is generic, has where bounds or has a default type.
    /// - If there are multiple associated `ErrorCode` types.
    fn analyse_error_code(
        item_type: &syn::TraitItemType,
        previous: &mut Option<syn::TraitItemType>,
    ) -> Result<()> {
        if item_type.ident != "ErrorCode" {
            return Err(format_err_spanned!(
                item_type.ident,
                "chain extensions expect an associated type with name `ErrorCode` but found {}",
                item_type.ident,
            ));
        }
        if !item_type.generics.params.is_empty() {
            return Err(format_err_spanned!(
                item_type.generics,
                "generic chain extension `ErrorCode` types are not supported",
            ))
        }
        if !item_type.bounds.is_empty() {
            return Err(format_err_spanned!(
                item_type.bounds,
                "bounded chain extension `ErrorCode` types are not supported",
            ))
        }
        if item_type.default.is_none() {
            return Err(format_err_spanned!(
                item_type,
                "expected a default type for the ink! chain extension ErrorCode",
            ))
        }
        match previous {
            Some(previous_error_code) => {
                return Err(format_err_spanned!(
                    item_type,
                    "encountered duplicate `ErrorCode` associated types for the chain extension",
                )).map_err(|err| err.into_combine(format_err_spanned!(
                    previous_error_code,
                    "first `ErrorCode` associated type here",
                )))
            }
            None => {
                *previous = Some(item_type.clone());
            }
        }
        Ok(())
    }

    /// Returns `Ok` if all trait items respect the requirements for an ink! chain
    /// extension.
    ///
    /// # Errors
    ///
    /// - If the trait contains an unsupported trait item such as
    ///     - associated constants (`const`)
    ///     - associated types (`type`)
    ///     - macros definitions or usages
    ///     - unknown token sequences (`Verbatim`s)
    ///     - methods with default implementations
    /// - If the trait contains methods which do not respect the ink! trait definition
    ///   requirements:
    ///     - All trait methods must not have a `self` receiver.
    ///     - All trait methods must have an `#[ink(function = N: u16)]` attribute that is
    ///       the ID that corresponds with the function ID of the respective chain
    ///       extension call.
    ///
    /// # Note
    ///
    /// The input Rust trait item is going to be replaced with a concrete chain extension
    /// type definition as a result of this procedural macro invocation.
    fn analyse_items(
        ext_id: ExtensionId,
        item_trait: &syn::ItemTrait,
    ) -> Result<(syn::TraitItemType, Vec<ChainExtensionMethod>)> {
        let mut methods = Vec::new();
        let mut seen_ids = HashMap::new();
        let mut error_code = None;
        for trait_item in &item_trait.items {
            match trait_item {
                syn::TraitItem::Const(const_trait_item) => {
                    return Err(format_err_spanned!(
                        const_trait_item,
                        "associated constants in ink! chain extensions are not supported, yet"
                    ))
                }
                syn::TraitItem::Macro(macro_trait_item) => {
                    return Err(format_err_spanned!(
                        macro_trait_item,
                        "macros in ink! chain extensions are not supported"
                    ))
                }
                syn::TraitItem::Type(type_trait_item) => {
                    Self::analyse_error_code(type_trait_item, &mut error_code)?;
                }
                syn::TraitItem::Verbatim(verbatim) => {
                    return Err(format_err_spanned!(
                        verbatim,
                        "encountered unsupported item in ink! chain extensions"
                    ))
                }
                syn::TraitItem::Fn(fn_trait_item) => {
                    let method = Self::analyse_methods(ext_id, fn_trait_item)?;
                    let method_id = method.id();
                    if let Some(previous) = seen_ids.get(&method_id) {
                        return Err(format_err!(
                            method.span(),
                            "encountered duplicate extension identifiers for the same chain extension",
                        ).into_combine(format_err!(
                            *previous,
                            "previous duplicate extension identifier here",
                        )))
                    }
                    seen_ids.insert(method_id, method.span());
                    methods.push(method);
                }
                unknown => {
                    return Err(format_err_spanned!(
                        unknown,
                        "encountered unknown or unsupported item in ink! chain extensions"
                    ))
                }
            }
        }
        let error_code = match error_code {
            Some(error_code) => error_code,
            None => {
                return Err(format_err_spanned!(
                    item_trait,
                    "missing ErrorCode associated type from ink! chain extension",
                ))
            }
        };
        Ok((error_code, methods))
    }

    /// Analyses a chain extension method.
    ///
    /// # Errors
    ///
    /// - If the method is missing the `#[ink(function = N: u16)]` attribute.
    /// - If the method has a `self` receiver.
    /// - If the method declared as `unsafe`, `const` or `async`.
    /// - If the method has some explicit API.
    /// - If the method is variadic or has generic parameters.
    fn analyse_methods(
        ext_id: ExtensionId,
        method: &syn::TraitItemFn,
    ) -> Result<ChainExtensionMethod> {
        if let Some(default_impl) = &method.default {
            return Err(format_err_spanned!(
                default_impl,
                "ink! chain extension methods with default implementations are not supported"
            ));
        }
        if let Some(constness) = &method.sig.constness {
            return Err(format_err_spanned!(
                constness,
                "const ink! chain extension methods are not supported"
            ))
        }
        if let Some(asyncness) = &method.sig.asyncness {
            return Err(format_err_spanned!(
                asyncness,
                "async ink! chain extension methods are not supported"
            ))
        }
        if let Some(unsafety) = &method.sig.unsafety {
            return Err(format_err_spanned!(
                unsafety,
                "unsafe ink! chain extension methods are not supported"
            ))
        }
        if let Some(abi) = &method.sig.abi {
            return Err(format_err_spanned!(
                abi,
                "ink! chain extension methods with non default ABI are not supported"
            ))
        }
        if let Some(variadic) = &method.sig.variadic {
            return Err(format_err_spanned!(
                variadic,
                "variadic ink! chain extension methods are not supported"
            ))
        }
        if !method.sig.generics.params.is_empty() {
            return Err(format_err_spanned!(
                method.sig.generics.params,
                "generic ink! chain extension methods are not supported"
            ))
        }
        match ir::first_ink_attribute(&method.attrs)?
                .map(|attr| attr.first().kind().clone()) {
            Some(ir::AttributeArg::Function(func_id)) => {
                Self::analyse_chain_extension_method(method, ext_id, func_id)
            }
            Some(_unsupported) => {
                Err(format_err_spanned!(
                    method,
                    "encountered unsupported ink! attribute for ink! chain extension method. expected #[ink(function = N: u16)] attribute"
                ))
            }
            None => {
                Err(format_err_spanned!(
                    method,
                    "missing #[ink(function = N: u16)] flag on ink! chain extension method"
                ))
            }
        }
    }

    /// Analyses the properties of an ink! chain extension method.
    ///
    /// # Errors
    ///
    /// - If the chain extension method has a `self` receiver as first argument.
    fn analyse_chain_extension_method(
        item_method: &syn::TraitItemFn,
        ext_id: ExtensionId,
        func_id: FunctionId,
    ) -> Result<ChainExtensionMethod> {
        let (ink_attrs, _) = ir::sanitize_attributes(
            item_method.span(),
            item_method.attrs.clone(),
            &ir::AttributeArgKind::Function,
            |arg| {
                match arg.kind() {
                    ir::AttributeArg::Function(_) | ir::AttributeArg::HandleStatus(_) => {
                        Ok(())
                    }
                    _ => Err(None),
                }
            },
        )?;
        if let Some(receiver) = item_method.sig.receiver() {
            return Err(format_err_spanned!(
                receiver,
                "ink! chain extension method must not have a `self` receiver",
            ))
        }
        let result = ChainExtensionMethod {
            id: GlobalMethodId::new(ext_id, func_id),
            item: item_method.clone(),
            handle_status: ink_attrs.is_handle_status(),
        };
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Checks if the token stream in `$chain_extension` results in the expected error
    /// message.
    macro_rules! assert_ink_chain_extension_eq_err {
        ( error: $err_str:literal, $($chain_extension:tt)* ) => {
            assert_eq!(
                ChainExtension::try_from(
                    syn::parse_quote! {
                        $( $chain_extension )*
                    },
                    Config::default()
                )
                .map_err(|err| err.to_string()),
                Err(
                    $err_str.to_string()
                )
            )
        };
    }

    #[test]
    fn unsafe_chain_extension_is_denied() {
        assert_ink_chain_extension_eq_err!(
            error: "ink! chain extensions cannot be unsafe",
            pub unsafe trait MyChainExtension {}
        );
    }

    #[test]
    fn auto_chain_extension_is_denied() {
        assert_ink_chain_extension_eq_err!(
            error: "ink! chain extensions cannot be automatically implemented traits",
            pub auto trait MyChainExtension {}
        );
    }

    #[test]
    fn non_pub_chain_extension_is_denied() {
        assert_ink_chain_extension_eq_err!(
            error: "ink! chain extensions must have public visibility",
            trait MyChainExtension {}
        );
        assert_ink_chain_extension_eq_err!(
            error: "ink! chain extensions must have public visibility",
            pub(crate) trait MyChainExtension {}
        );
    }

    #[test]
    fn generic_chain_extension_is_denied() {
        assert_ink_chain_extension_eq_err!(
            error: "ink! chain extensions must not be generic",
            pub trait MyChainExtension<T> {}
        );
    }

    #[test]
    fn chain_extension_with_supertraits_is_denied() {
        assert_ink_chain_extension_eq_err!(
            error: "ink! chain extensions with super-traits are not supported, yet",
            pub trait MyChainExtension: SuperChainExtension {}
        );
    }

    #[test]
    fn chain_extension_containing_const_item_is_denied() {
        assert_ink_chain_extension_eq_err!(
            error: "associated constants in ink! chain extensions are not supported, yet",
            pub trait MyChainExtension {
                const T: i32;
            }
        );
    }

    #[test]
    fn chain_extension_containing_invalid_associated_type_is_denied() {
        assert_ink_chain_extension_eq_err!(
            error: "chain extensions expect an associated type with name `ErrorCode` but found Type",
            pub trait MyChainExtension {
                type Type;
            }
        );
    }

    #[test]
    fn chain_extension_with_invalid_error_code() {
        assert_ink_chain_extension_eq_err!(
            error: "chain extensions expect an associated type with name `ErrorCode` but found IncorrectName",
            pub trait MyChainExtension {
                type IncorrectName = ();
            }
        );
        assert_ink_chain_extension_eq_err!(
            error: "generic chain extension `ErrorCode` types are not supported",
            pub trait MyChainExtension {
                type ErrorCode<T> = ();
            }
        );
        assert_ink_chain_extension_eq_err!(
            error: "bounded chain extension `ErrorCode` types are not supported",
            pub trait MyChainExtension {
                type ErrorCode: Copy = ();
            }
        );
        assert_ink_chain_extension_eq_err!(
            error: "expected a default type for the ink! chain extension ErrorCode",
            pub trait MyChainExtension {
                type ErrorCode;
            }
        );
        assert_ink_chain_extension_eq_err!(
            error: "encountered duplicate `ErrorCode` associated types for the chain extension",
            pub trait MyChainExtension {
                type ErrorCode = ();
                type ErrorCode = ();
            }
        );
    }

    #[test]
    fn chain_extension_containing_macro_is_denied() {
        assert_ink_chain_extension_eq_err!(
            error: "macros in ink! chain extensions are not supported",
            pub trait MyChainExtension {
                my_macro_call!();
            }
        );
    }

    #[test]
    fn chain_extension_containing_non_flagged_method_is_denied() {
        assert_ink_chain_extension_eq_err!(
            error: "missing #[ink(function = N: u16)] flag on ink! chain extension method",
            pub trait MyChainExtension {
                fn non_flagged_1(&self);
            }
        );
        assert_ink_chain_extension_eq_err!(
            error: "missing #[ink(function = N: u16)] flag on ink! chain extension method",
            pub trait MyChainExtension {
                fn non_flagged_2(&mut self);
            }
        );
        assert_ink_chain_extension_eq_err!(
            error: "missing #[ink(function = N: u16)] flag on ink! chain extension method",
            pub trait MyChainExtension {
                fn non_flagged_3() -> Self;
            }
        );
    }

    #[test]
    fn chain_extension_containing_default_implemented_methods_is_denied() {
        assert_ink_chain_extension_eq_err!(
            error: "ink! chain extension methods with default implementations are not supported",
            pub trait MyChainExtension {
                #[ink(constructor)]
                fn default_implemented() -> Self {}
            }
        );
    }

    #[test]
    fn chain_extension_containing_const_methods_is_denied() {
        assert_ink_chain_extension_eq_err!(
            error: "const ink! chain extension methods are not supported",
            pub trait MyChainExtension {
                #[ink(function = 1)]
                const fn const_constructor() -> Self;
            }
        );
    }

    #[test]
    fn chain_extension_containing_async_methods_is_denied() {
        assert_ink_chain_extension_eq_err!(
            error: "async ink! chain extension methods are not supported",
            pub trait MyChainExtension {
                #[ink(function = 1)]
                async fn const_constructor() -> Self;
            }
        );
    }

    #[test]
    fn chain_extension_containing_unsafe_methods_is_denied() {
        assert_ink_chain_extension_eq_err!(
            error: "unsafe ink! chain extension methods are not supported",
            pub trait MyChainExtension {
                #[ink(function = 1)]
                unsafe fn const_constructor() -> Self;
            }
        );
    }

    #[test]
    fn chain_extension_containing_methods_using_explicit_abi_is_denied() {
        assert_ink_chain_extension_eq_err!(
            error: "ink! chain extension methods with non default ABI are not supported",
            pub trait MyChainExtension {
                #[ink(function = 1)]
                extern fn const_constructor() -> Self;
            }
        );
    }

    #[test]
    fn chain_extension_containing_variadic_methods_is_denied() {
        assert_ink_chain_extension_eq_err!(
            error: "variadic ink! chain extension methods are not supported",
            pub trait MyChainExtension {
                #[ink(function = 1)]
                fn const_constructor(...) -> Self;
            }
        );
    }

    #[test]
    fn chain_extension_containing_generic_methods_is_denied() {
        assert_ink_chain_extension_eq_err!(
            error: "generic ink! chain extension methods are not supported",
            pub trait MyChainExtension {
                #[ink(function = 1)]
                fn const_constructor<T>() -> Self;
            }
        );
    }

    #[test]
    fn chain_extension_containing_method_with_unsupported_ink_attribute_is_denied() {
        assert_ink_chain_extension_eq_err!(
            error: "\
                encountered unsupported ink! attribute for ink! chain extension method. \
                expected #[ink(function = N: u16)] attribute",
            pub trait MyChainExtension {
                #[ink(message)]
                fn unsupported_ink_attribute(&self);
            }
        );
        assert_ink_chain_extension_eq_err!(
            error: "encountered unknown ink! attribute argument: unknown",
            pub trait MyChainExtension {
                #[ink(unknown)]
                fn unknown_ink_attribute(&self);
            }
        );
    }

    #[test]
    fn chain_extension_containing_method_with_invalid_marker() {
        assert_ink_chain_extension_eq_err!(
            error: "could not parse `N` in `#[ink(function = N)]` into a `u16` integer: \
            invalid digit found in string",
            pub trait MyChainExtension {
                #[ink(function = -1)]
                fn has_self_receiver();
            }
        );
        let too_large = (u16::MAX as u64) + 1;
        assert_ink_chain_extension_eq_err!(
            error: "could not parse `N` in `#[ink(function = N)]` into a `u16` integer: \
            number too large to fit in target type",
            pub trait MyChainExtension {
                #[ink(function = #too_large)]
                fn has_self_receiver();
            }
        );
        assert_ink_chain_extension_eq_err!(
            error: "expected `u16` integer type for `N` in #[ink(function = N)]",
            pub trait MyChainExtension {
                #[ink(function = "Hello, World!")]
                fn has_self_receiver();
            }
        );
        assert_ink_chain_extension_eq_err!(
            error: "encountered #[ink(function)] that is missing its `id` parameter. \
                    Did you mean #[ink(function = id: u16)] ?",
            pub trait MyChainExtension {
                #[ink(function)]
                fn has_self_receiver();
            }
        );

        assert_ink_chain_extension_eq_err!(
            error: "encountered duplicate ink! attribute",
            pub trait MyChainExtension {
                #[ink(function = 42)]
                #[ink(function = 42)]
                fn duplicate_attributes() -> Self;
            }
        );
        assert_ink_chain_extension_eq_err!(
            error: "encountered ink! attribute arguments with equal kinds",
            pub trait MyChainExtension {
                #[ink(function = 1)]
                #[ink(function = 2)]
                fn duplicate_attributes() -> Self;
            }
        );
        assert_ink_chain_extension_eq_err!(
            error: "encountered conflicting ink! attribute argument",
            pub trait MyChainExtension {
                #[ink(function = 1)]
                #[ink(message)]
                fn conflicting_attributes() -> Self;
            }
        );
    }

    #[test]
    fn chain_extension_containing_method_with_self_receiver_is_denied() {
        assert_ink_chain_extension_eq_err!(
            error: "ink! chain extension method must not have a `self` receiver",
            pub trait MyChainExtension {
                type ErrorCode = ();

                #[ink(function = 1)]
                fn has_self_receiver(&self) -> Self;
            }
        );
        assert_ink_chain_extension_eq_err!(
            error: "ink! chain extension method must not have a `self` receiver",
            pub trait MyChainExtension {
                type ErrorCode = ();

                #[ink(function = 1)]
                fn has_self_receiver(&mut self) -> Self;
            }
        );
        assert_ink_chain_extension_eq_err!(
            error: "ink! chain extension method must not have a `self` receiver",
            pub trait MyChainExtension {
                type ErrorCode = ();

                #[ink(function = 1)]
                fn has_self_receiver(self) -> Self;
            }
        );
        assert_ink_chain_extension_eq_err!(
            error: "ink! chain extension method must not have a `self` receiver",
            pub trait MyChainExtension {
                type ErrorCode = ();

                #[ink(function = 1)]
                fn has_self_receiver(self: &Self) -> Self;
            }
        );
        assert_ink_chain_extension_eq_err!(
            error: "ink! chain extension method must not have a `self` receiver",
            pub trait MyChainExtension {
                type ErrorCode = ();

                #[ink(function = 1)]
                fn has_self_receiver(self: Self) -> Self;
            }
        );
    }

    #[test]
    fn chain_extension_with_overlapping_extension_ids() {
        assert_ink_chain_extension_eq_err!(
            error: "encountered duplicate extension identifiers for the same chain extension",
            pub trait MyChainExtension {
                #[ink(function = 1)]
                fn same_id_1();
                #[ink(function = 1)]
                fn same_id_2();
            }
        );
    }

    #[test]
    fn chain_extension_is_ok() {
        let chain_extension = ChainExtension::try_from(syn::parse_quote! {
                pub trait MyChainExtension {
                    type ErrorCode = ();

                    #[ink(function = 1)]
                    fn extension_1();
                    #[ink(function = 2)]
                    fn extension_2(input: i32);
                    #[ink(function = 3)]
                    fn extension_3() -> i32;
                    #[ink(function = 4)]
                    fn extension_4(input: i32) -> i32;
                    #[ink(function = 5)]
                    fn extension_5(in1: i8, in2: i16, in3: i32, in4: i64) -> (u8, u16, u32, u64);
                }
            }, Config::default()).unwrap();
        assert_eq!(chain_extension.methods.len(), 5);
        for (actual, expected) in chain_extension
            .methods
            .iter()
            .map(|method| method.id())
            .zip(1..=5u32)
        {
            assert_eq!(actual.index, expected);
        }
        for (actual, expected) in chain_extension
            .methods
            .iter()
            .map(|method| method.ident().to_string())
            .zip(
                [
                    "extension_1",
                    "extension_2",
                    "extension_3",
                    "extension_4",
                    "extension_5",
                ]
                .iter()
                .map(ToString::to_string),
            )
        {
            assert_eq!(actual, expected);
        }
    }

    #[test]
    fn chain_extension_with_params_is_ok() {
        let chain_extension = ChainExtension::try_from(
            syn::parse_quote! {
                pub trait MyChainExtension {
                    type ErrorCode = ();

                    #[ink(function = 1, handle_status = false)]
                    fn extension_a();
                    #[ink(function = 2)]
                    fn extension_b();
                    #[ink(function = 3, handle_status = false)]
                    fn extension_c();
                    #[ink(function = 4)]
                    #[ink(handle_status = false)]
                    fn extension_d();
                    #[ink(function = 5)]
                    fn extension_e();
                    #[ink(function = 6)]
                    #[ink(handle_status = false)]
                    fn extension_f();
                }
            },
            Config::default(),
        )
        .unwrap();
        let expected_methods = 6;
        assert_eq!(chain_extension.methods.len(), expected_methods);
        for (actual, expected) in chain_extension
            .methods
            .iter()
            .map(|method| method.id())
            .zip(1..=expected_methods as u32)
        {
            assert_eq!(actual.index, expected);
        }
        for (actual, expected) in chain_extension
            .methods
            .iter()
            .map(|method| method.ident().to_string())
            .zip(
                [
                    "extension_a",
                    "extension_b",
                    "extension_c",
                    "extension_d",
                    "extension_e",
                    "extension_f",
                ]
                .iter()
                .map(ToString::to_string),
            )
        {
            assert_eq!(actual, expected);
        }
    }

    /// Asserts that the given input configuration attribute argument are converted
    /// into the expected ink! configuration or yields the expected error message.
    fn assert_config(
        input: ast::AttributeArgs,
        expected: core::result::Result<Config, &'static str>,
    ) {
        assert_eq!(
            <Config as TryFrom<ast::AttributeArgs>>::try_from(input)
                .map_err(|err| err.to_string()),
            expected.map_err(ToString::to_string),
        );
    }

    #[test]
    fn empty_config_fails() {
        assert_config(
            syn::parse_quote! {},
            Err("missing required `extension = N: u16` argument on ink! chain extension"),
        )
    }

    #[test]
    fn extension_works() {
        assert_config(
            syn::parse_quote! {
                extension = 13
            },
            Ok(Config {
                ext_id: ExtensionId::from_u16(13),
            }),
        )
    }

    #[test]
    fn extension_invalid_value_fails() {
        assert_config(
            syn::parse_quote! { extension = "invalid" },
            Err("expected `u16` integer type for `N` in `extension = N`"),
        );
    }

    #[test]
    fn unknown_arg_fails() {
        assert_config(
            syn::parse_quote! { unknown = argument },
            Err("encountered unknown or unsupported chain extension configuration argument"),
        );
    }

    #[test]
    fn duplicate_args_fails() {
        assert_config(
            syn::parse_quote! {
                extension = 13,
                extension = 123,
            },
            Err("encountered duplicate ink! contract `extension` configuration argument"),
        );
    }
}
