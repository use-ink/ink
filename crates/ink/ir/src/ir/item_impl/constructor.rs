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

use super::{
    Callable,
    CallableKind,
    InputsIter,
    Visibility,
    ensure_callable_invariants,
};
use crate::{
    ir,
    ir::{
        attrs::SelectorOrWildcard,
        utils::{
            extract_cfg_attributes,
            extract_cfg_syn_attributes,
        },
    },
};
use proc_macro2::{
    Ident,
    Span,
    TokenStream,
};
use syn::spanned::Spanned as _;

/// An ink! constructor definition.
///
/// # Example
///
/// ## Inherent implementation constructor:
///
/// ```
/// # let event = <ink_ir::ItemImpl as TryFrom<syn::ItemImpl>>::try_from(syn::parse_quote! {
/// impl MyStorage {
///     #[ink(constructor)]
///     pub fn new(init_value: i32) -> Self {
///         /* contract initialization goes here */
/// #       unimplemented!()
///     }
/// }
/// # }).unwrap();
/// ```
///
/// ## Trait implementation constructor:
///
/// ```
/// # <ink_ir::ItemImpl as TryFrom<syn::ItemImpl>>::try_from(syn::parse_quote! {
/// impl MyTrait for MyStorage {
///     #[ink(constructor)]
///     fn new(init_value: i32) -> Self {
///         // contract initialization goes here
/// #       unimplemented!()
///     }
/// }
/// # }).unwrap();
/// ```
#[derive(Debug, PartialEq, Eq)]
pub struct Constructor {
    /// The underlying Rust method item.
    pub(super) item: syn::ImplItemFn,
    /// If the ink! constructor can receive funds.
    is_payable: bool,
    /// If the ink! constructor is default.
    is_default: bool,
    /// An optional user provided selector.
    ///
    /// # Note
    ///
    /// This overrides the computed selector, even when using a manual namespace
    /// for the parent implementation block.
    selector: Option<SelectorOrWildcard>,
    /// An optional function name override.
    ///
    /// # Note
    ///
    /// - Useful for defining overloaded interfaces.
    /// - If provided, the name must be a valid "identifier-like" string.
    name: Option<String>,
}

impl quote::ToTokens for Constructor {
    /// We mainly implement this trait for this ink! type to have a derived
    /// [`Spanned`](`syn::spanned::Spanned`) implementation for it.
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.item.to_tokens(tokens)
    }
}

impl Constructor {
    /// Ensure that the constructor has a return.
    /// Returns an error otherwise.
    ///
    /// # Errors
    ///
    /// If the ink! constructor is missing a return type.
    fn ensure_return(method_item: &syn::ImplItemFn) -> Result<(), syn::Error> {
        if let syn::ReturnType::Default = &method_item.sig.output {
            return Err(format_err_spanned!(
                &method_item.sig,
                "missing return for ink! constructor",
            ))
        }
        Ok(())
    }

    /// Ensures that the ink! constructor has no `self` receiver.
    ///
    /// Returns an appropriate error otherwise.
    ///
    /// # Errors
    ///
    /// If the ink! constructor has a `&self`, `&mut self`, `self` or any other
    /// kind of a `self` receiver as first argument.
    fn ensure_no_self_receiver(method_item: &syn::ImplItemFn) -> Result<(), syn::Error> {
        match method_item.sig.inputs.iter().next() {
            None | Some(syn::FnArg::Typed(_)) => (),
            Some(syn::FnArg::Receiver(receiver)) => {
                return Err(format_err_spanned!(
                    receiver,
                    "ink! constructors must have no `self` receiver",
                ))
            }
        }
        Ok(())
    }

    /// Sanitizes the attributes for the ink! constructor.
    ///
    /// Returns a tuple of ink! attributes and non-ink! attributes.
    fn sanitize_attributes(
        method_item: &syn::ImplItemFn,
    ) -> Result<(ir::InkAttribute, Vec<syn::Attribute>), syn::Error> {
        ir::sanitize_attributes(
            method_item.span(),
            method_item.attrs.clone(),
            &ir::AttributeArgKind::Constructor,
            |arg| {
                match arg.kind() {
                    ir::AttributeArg::Constructor
                    | ir::AttributeArg::Payable
                    | ir::AttributeArg::Default
                    | ir::AttributeArg::Selector(_)
                    | ir::AttributeArg::Name(_) => Ok(()),
                    _ => Err(None),
                }
            },
        )
    }
}

impl TryFrom<syn::ImplItemFn> for Constructor {
    type Error = syn::Error;

    fn try_from(method_item: syn::ImplItemFn) -> Result<Self, Self::Error> {
        ensure_callable_invariants(&method_item, CallableKind::Constructor)?;
        Self::ensure_return(&method_item)?;
        Self::ensure_no_self_receiver(&method_item)?;
        let (ink_attrs, other_attrs) = Self::sanitize_attributes(&method_item)?;
        let is_payable = ink_attrs.is_payable();
        let is_default = ink_attrs.is_default();
        let selector = ink_attrs.selector();
        let name = ink_attrs.name();
        #[cfg(ink_abi = "sol")]
        if selector.is_some() {
            let selector_span = ink_attrs.args().find_map(|arg| {
                matches!(arg.kind(), ir::AttributeArg::Selector(_)).then_some(arg.span())
            });
            return Err(format_err!(
                selector_span.unwrap_or_else(|| method_item.span()),
                "constructor `selector` attributes are not supported in Solidity ABI compatibility mode",
            ));
        }
        Ok(Constructor {
            selector,
            is_payable,
            is_default,
            name,
            item: syn::ImplItemFn {
                attrs: other_attrs,
                ..method_item
            },
        })
    }
}

impl Callable for Constructor {
    fn kind(&self) -> CallableKind {
        CallableKind::Constructor
    }

    fn ident(&self) -> &Ident {
        &self.item.sig.ident
    }

    fn user_provided_selector(&self) -> Option<&ir::Selector> {
        if let Some(SelectorOrWildcard::UserProvided(selector)) = self.selector.as_ref() {
            return Some(selector)
        }
        None
    }

    fn has_wildcard_selector(&self) -> bool {
        matches!(self.selector, Some(SelectorOrWildcard::Wildcard))
    }

    fn has_wildcard_complement_selector(&self) -> bool {
        self.selector == Some(SelectorOrWildcard::wildcard_complement())
    }

    fn is_payable(&self) -> bool {
        self.is_payable
    }

    fn is_default(&self) -> bool {
        self.is_default
    }

    fn visibility(&self) -> Visibility {
        match &self.item.vis {
            syn::Visibility::Public(vis_public) => Visibility::Public(*vis_public),
            syn::Visibility::Inherited => Visibility::Inherited,
            _ => unreachable!("encountered invalid visibility for ink! constructor"),
        }
    }

    fn inputs(&self) -> InputsIter<'_> {
        InputsIter::from(self)
    }

    fn inputs_span(&self) -> Span {
        self.item.sig.inputs.span()
    }

    fn statements(&self) -> &[syn::Stmt] {
        &self.item.block.stmts
    }

    fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }
}

impl Constructor {
    /// Returns a slice of all non-ink! attributes of the ink! constructor.
    pub fn attrs(&self) -> &[syn::Attribute] {
        &self.item.attrs
    }

    /// Returns a list of `cfg` attributes if any.
    pub fn get_cfg_attrs(&self, span: Span) -> Vec<TokenStream> {
        extract_cfg_attributes(self.attrs(), span)
    }

    /// Returns a list of `cfg` attributes as `syn::Attribute` if any.
    pub fn get_cfg_syn_attrs(&self) -> Vec<syn::Attribute> {
        extract_cfg_syn_attributes(self.attrs())
    }

    /// Returns the return type of the ink! constructor if any.
    pub fn output(&self) -> Option<&syn::Type> {
        match &self.item.sig.output {
            syn::ReturnType::Default => None,
            syn::ReturnType::Type(_, return_type) => Some(return_type),
        }
    }

    /// Returns the function name override (if any).
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inputs_works() {
        macro_rules! expected_inputs {
            ( $( $name:ident: $ty:ty ),* ) => {{
                vec![
                    $(
                        syn::parse_quote! {
                            $name: $ty
                        }
                    ),*
                ]
            }};
        }
        let test_inputs: Vec<(Vec<syn::FnArg>, syn::ImplItemFn)> = vec![
            (
                // No inputs:
                expected_inputs!(),
                syn::parse_quote! {
                    #[ink(constructor)]
                    fn my_constructor() -> Self {}
                },
            ),
            (
                // Single input:
                expected_inputs!(a: i32),
                syn::parse_quote! {
                    #[ink(constructor)]
                    fn my_constructor(a: i32) -> Self {}
                },
            ),
            (
                // Some inputs:
                expected_inputs!(a: i32, b: u64, c: [u8; 32]),
                syn::parse_quote! {
                    #[ink(constructor)]
                    fn my_constructor(a: i32, b: u64, c: [u8; 32]) -> Self {}
                },
            ),
        ];
        for (expected_inputs, item_method) in test_inputs {
            let actual_inputs = <ir::Constructor as TryFrom<_>>::try_from(item_method)
                .unwrap()
                .inputs()
                .cloned()
                .map(syn::FnArg::Typed)
                .collect::<Vec<_>>();
            assert_eq!(actual_inputs, expected_inputs);
        }
    }

    #[test]
    fn is_payable_works() {
        let test_inputs: Vec<(bool, syn::ImplItemFn)> = vec![
            // Not payable.
            (
                false,
                syn::parse_quote! {
                    #[ink(constructor)]
                    fn my_constructor() -> Self {}
                },
            ),
            // Normalized ink! attribute.
            (
                true,
                syn::parse_quote! {
                    #[ink(constructor, payable)]
                    pub fn my_constructor() -> Self {}
                },
            ),
            // Different ink! attributes.
            (
                true,
                syn::parse_quote! {
                    #[ink(constructor)]
                    #[ink(payable)]
                    pub fn my_constructor() -> Self {}
                },
            ),
            // Another ink! attribute, separate and normalized attribute.
            (
                true,
                syn::parse_quote! {
                    #[ink(constructor)]
                    #[ink(selector = 0xDEADBEEF, payable)]
                    pub fn my_constructor() -> Self {}
                },
            ),
        ];
        for (expect_payable, item_method) in test_inputs {
            let is_payable = <ir::Constructor as TryFrom<_>>::try_from(item_method)
                .unwrap()
                .is_payable();
            assert_eq!(is_payable, expect_payable);
        }
    }

    #[test]
    fn is_default_works() {
        let test_inputs: Vec<(bool, syn::ImplItemFn)> = vec![
            // Not default.
            (
                false,
                syn::parse_quote! {
                    #[ink(constructor)]
                    fn my_constructor() -> Self {}
                },
            ),
            // Default constructor.
            (
                true,
                syn::parse_quote! {
                    #[ink(constructor, default)]
                    pub fn my_constructor() -> Self {}
                },
            ),
        ];
        for (expect_default, item_method) in test_inputs {
            let is_default = <ir::Constructor as TryFrom<_>>::try_from(item_method)
                .unwrap()
                .is_default();
            assert_eq!(is_default, expect_default);
        }
    }

    #[test]
    fn name_override_works() {
        let test_inputs: Vec<(Option<&str>, syn::ImplItemFn)> = vec![
            // No name override.
            (
                None,
                syn::parse_quote! {
                    #[ink(constructor)]
                    fn my_constructor() -> Self {}
                },
            ),
            // Name override.
            (
                Some("myConstructor"),
                syn::parse_quote! {
                    #[ink(constructor, name = "myConstructor")]
                    pub fn my_constructor() -> Self {}
                },
            ),
        ];
        for (expected_name, item_method) in test_inputs {
            let ctor = <ir::Constructor as TryFrom<_>>::try_from(item_method).unwrap();
            assert_eq!(ctor.name(), expected_name);
        }
    }

    #[test]
    fn visibility_works() {
        let test_inputs: Vec<(bool, syn::ImplItemFn)> = vec![
            // inherited
            (
                false,
                syn::parse_quote! {
                    #[ink(constructor)]
                    fn my_constructor() -> Self {}
                },
            ),
            // public
            (
                true,
                syn::parse_quote! {
                    #[ink(constructor)]
                    pub fn my_constructor() -> Self {}
                },
            ),
        ];
        for (is_pub, item_method) in test_inputs {
            let visibility = <ir::Constructor as TryFrom<_>>::try_from(item_method)
                .unwrap()
                .visibility();
            assert_eq!(visibility.is_pub(), is_pub);
            assert_eq!(visibility.is_inherited(), !is_pub);
        }
    }

    #[test]
    fn try_from_works() {
        let item_methods: Vec<syn::ImplItemFn> = vec![
            // simple + inherited visibility
            syn::parse_quote! {
                #[ink(constructor)]
                fn my_constructor() -> Self {}
            },
            // simple + public visibility
            syn::parse_quote! {
                #[ink(constructor)]
                pub fn my_constructor() -> Self {}
            },
            // many inputs
            syn::parse_quote! {
                #[ink(constructor)]
                fn my_constructor(input1: i32, input2: i64, input3: u32, input4: u64) -> Self {}
            },
            // Result return type
            syn::parse_quote! {
                #[ink(constructor)]
                pub fn my_constructor() -> Result<Self, ()> {}
            },
        ];
        for item_method in item_methods {
            assert!(<ir::Constructor as TryFrom<_>>::try_from(item_method).is_ok());
        }
    }

    fn assert_try_from_fails(item_method: syn::ImplItemFn, expected_err: &str) {
        assert_eq!(
            <ir::Constructor as TryFrom<_>>::try_from(item_method)
                .map_err(|err| err.to_string()),
            Err(expected_err.to_string()),
        );
    }

    #[test]
    fn try_from_missing_return_fails() {
        let item_methods: Vec<syn::ImplItemFn> = vec![
            syn::parse_quote! {
                #[ink(constructor)]
                fn my_constructor() {}
            },
            syn::parse_quote! {
                #[ink(constructor)]
                pub fn my_constructor() {}
            },
        ];
        for item_method in item_methods {
            assert_try_from_fails(item_method, "missing return for ink! constructor")
        }
    }

    #[test]
    fn try_from_invalid_self_receiver_fails() {
        let item_methods: Vec<syn::ImplItemFn> = vec![
            syn::parse_quote! {
                #[ink(constructor)]
                fn my_constructor(&self) -> Self {}
            },
            syn::parse_quote! {
                #[ink(constructor)]
                pub fn my_constructor(&mut self) -> Self {}
            },
            syn::parse_quote! {
                #[ink(constructor)]
                pub fn my_constructor(self) -> Self {}
            },
            syn::parse_quote! {
                #[ink(constructor)]
                pub fn my_constructor(mut self) -> Self {}
            },
        ];
        for item_method in item_methods {
            assert_try_from_fails(
                item_method,
                "ink! constructors must have no `self` receiver",
            )
        }
    }

    #[test]
    fn try_from_generics_fails() {
        let item_methods: Vec<syn::ImplItemFn> = vec![
            syn::parse_quote! {
                #[ink(constructor)]
                fn my_constructor<T>() -> Self {}
            },
            syn::parse_quote! {
                #[ink(constructor)]
                pub fn my_constructor<T>() -> Self {}
            },
        ];
        for item_method in item_methods {
            assert_try_from_fails(item_method, "ink! constructors must not be generic")
        }
    }

    #[test]
    fn try_from_const_fails() {
        let item_methods: Vec<syn::ImplItemFn> = vec![
            syn::parse_quote! {
                #[ink(constructor)]
                const fn my_constructor() -> Self {}
            },
            syn::parse_quote! {
                #[ink(constructor)]
                pub const fn my_constructor() -> Self {}
            },
        ];
        for item_method in item_methods {
            assert_try_from_fails(item_method, "ink! constructors must not be const")
        }
    }

    #[test]
    fn try_from_async_fails() {
        let item_methods: Vec<syn::ImplItemFn> = vec![
            syn::parse_quote! {
                #[ink(constructor)]
                async fn my_constructor() -> Self {}
            },
            syn::parse_quote! {
                #[ink(constructor)]
                async fn my_constructor() -> Self {}
            },
        ];
        for item_method in item_methods {
            assert_try_from_fails(item_method, "ink! constructors must not be async")
        }
    }

    #[test]
    fn try_from_unsafe_fails() {
        let item_methods: Vec<syn::ImplItemFn> = vec![
            syn::parse_quote! {
                #[ink(constructor)]
                unsafe fn my_constructor() -> Self {}
            },
            syn::parse_quote! {
                #[ink(constructor)]
                unsafe fn my_constructor() -> Self {}
            },
        ];
        for item_method in item_methods {
            assert_try_from_fails(item_method, "ink! constructors must not be unsafe")
        }
    }

    #[test]
    fn try_from_explicit_abi_fails() {
        let item_methods: Vec<syn::ImplItemFn> = vec![
            syn::parse_quote! {
                #[ink(constructor)]
                extern "C" fn my_constructor() -> Self {}
            },
            syn::parse_quote! {
                #[ink(constructor)]
                extern "C" fn my_constructor() -> Self {}
            },
        ];
        for item_method in item_methods {
            assert_try_from_fails(
                item_method,
                "ink! constructors must not have explicit ABI",
            )
        }
    }

    #[test]
    fn try_from_variadic_fails() {
        let item_methods: Vec<syn::ImplItemFn> = vec![
            syn::parse_quote! {
                #[ink(constructor)]
                fn my_constructor(...) -> Self {}
            },
            syn::parse_quote! {
                #[ink(constructor)]
                fn my_constructor(...) -> Self {}
            },
        ];
        for item_method in item_methods {
            assert_try_from_fails(item_method, "ink! constructors must not be variadic")
        }
    }

    #[test]
    fn try_from_visibility_fails() {
        let item_methods: Vec<syn::ImplItemFn> = vec![
            syn::parse_quote! {
                #[ink(constructor)]
                pub(crate) fn my_constructor() -> Self {}
            },
            syn::parse_quote! {
                #[ink(constructor)]
                pub(in my::path) fn my_constructor() -> Self {}
            },
        ];
        for item_method in item_methods {
            assert_try_from_fails(
                item_method,
                "ink! constructors must have public or inherited visibility",
            )
        }
    }

    #[test]
    fn conflicting_attributes_fails() {
        let item_methods: Vec<syn::ImplItemFn> = vec![
            // storage
            syn::parse_quote! {
                #[ink(constructor, storage)]
                fn my_constructor() -> Self {}
            },
            // namespace
            syn::parse_quote! {
                #[ink(constructor, namespace = "my_namespace")]
                fn my_constructor() -> Self {}
            },
            // event + multiple attributes
            syn::parse_quote! {
                #[ink(constructor)]
                #[ink(event)]
                fn my_constructor() -> Self {}
            },
        ];
        for item_method in item_methods {
            assert_try_from_fails(
                item_method,
                "encountered conflicting ink! attribute argument",
            )
        }
    }

    #[test]
    fn try_from_wildcard_constructor_works() {
        let item: syn::ImplItemFn = syn::parse_quote! {
            #[ink(constructor, selector = _)]
            pub fn my_constructor() -> Self {}
        };
        assert!(<ir::Constructor as TryFrom<_>>::try_from(item).is_ok());
    }
}
