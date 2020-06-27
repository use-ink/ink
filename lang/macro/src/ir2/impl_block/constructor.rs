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

use super::{
    ensure_callable_invariants,
    CallableKind,
    Visibility,
};
use crate::ir2;
use core::convert::TryFrom;
use syn::spanned::Spanned as _;

/// An ink! constructor definition.
#[derive(Debug, PartialEq, Eq)]
pub struct Constructor {
    /// The underlying Rust method item.
    item: syn::ImplItemMethod,
    /// If the ink! constructor can receive funds.
    is_payable: bool,
    /// An optional user provided selector.
    ///
    /// # Note
    ///
    /// This overrides the computed selector, even when using a manual salt
    /// for the parent implementation block.
    selector: Option<ir2::Selector>,
}

impl Constructor {
    /// Returns `true` if the given type is `Self`.
    fn type_is_self_val(ty: &syn::Type) -> bool {
        matches!(ty, syn::Type::Path(syn::TypePath {
            qself: None,
            path
        }) if path.is_ident("Self"))
    }

    /// Ensures that the return type of the ink! constructor is `Self`.
    ///
    /// Returns an appropriate error otherwise.
    ///
    /// # Errors
    ///
    /// If the ink! constructor does not return `Self` or is missing a return
    /// type entirely.
    fn ensure_valid_return_type(
        method_item: &syn::ImplItemMethod,
    ) -> Result<(), syn::Error> {
        match &method_item.sig.output {
            syn::ReturnType::Default => {
                return Err(format_err!(
                    &method_item.sig,
                    "missing return for ink! constructor",
                ))
            }
            syn::ReturnType::Type(_, return_type) => {
                if !Self::type_is_self_val(return_type.as_ref()) {
                    return Err(format_err!(
                        return_type,
                        "ink! constructors must return Self",
                    ))
                }
            }
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
    fn ensure_no_self_receiver(
        method_item: &syn::ImplItemMethod,
    ) -> Result<(), syn::Error> {
        match method_item.sig.inputs.iter().next() {
            None | Some(syn::FnArg::Typed(_)) => (),
            Some(syn::FnArg::Receiver(receiver)) => {
                return Err(format_err!(
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
        method_item: &syn::ImplItemMethod,
    ) -> Result<(ir2::InkAttribute, Vec<syn::Attribute>), syn::Error> {
        ir2::sanitize_attributes(
            method_item.span(),
            method_item.attrs.clone(),
            &ir2::AttributeArgKind::Constructor,
            |kind| {
                match kind {
                    ir2::AttributeArgKind::Constructor
                    | ir2::AttributeArgKind::Payable
                    | ir2::AttributeArgKind::Salt(_)
                    | ir2::AttributeArgKind::Selector(_) => false,
                    _ => false,
                }
            },
        )
    }
}

impl TryFrom<syn::ImplItemMethod> for Constructor {
    type Error = syn::Error;

    fn try_from(method_item: syn::ImplItemMethod) -> Result<Self, Self::Error> {
        let method_span = method_item.span();
        ensure_callable_invariants(&method_item, CallableKind::Constructor)?;
        Self::ensure_valid_return_type(&method_item)?;
        Self::ensure_no_self_receiver(&method_item)?;
        let (ink_attrs, other_attrs) = Self::sanitize_attributes(&method_item)?;
        let is_payable = false; // TODO
        let selector = None; // TODO
        Ok(Constructor {
            is_payable,
            selector,
            item: syn::ImplItemMethod {
                attrs: other_attrs,
                ..method_item
            },
        })
    }
}

impl Constructor {
    /// Returns the visibility of the constructor.
    pub fn visibility(&self) -> Visibility {
        match &self.item.vis {
            syn::Visibility::Public(vis_public) => Visibility::Public(vis_public.clone()),
            syn::Visibility::Inherited => Visibility::Inherited,
            _ => unreachable!("encountered invalid visibility for ink! constructor"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn visibility_works() {
        let test_inputs: Vec<(bool, syn::ImplItemMethod)> = vec![
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
            let visibility = <ir2::Constructor as TryFrom<_>>::try_from(item_method)
                .unwrap()
                .visibility();
            assert_eq!(visibility.is_pub(), is_pub);
            assert_eq!(visibility.is_inherited(), !is_pub);
        }
    }

    #[test]
    fn try_from_works() {
        let item_methods: Vec<syn::ImplItemMethod> = vec![
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
        ];
        for item_method in item_methods {
            assert!(<ir2::Constructor as TryFrom<_>>::try_from(item_method).is_ok());
        }
    }

    fn assert_try_from_fails(item_method: syn::ImplItemMethod, expected_err: &str) {
        assert_eq!(
            <ir2::Constructor as TryFrom<_>>::try_from(item_method)
                .map_err(|err| err.to_string()),
            Err(expected_err.to_string()),
        );
    }

    #[test]
    fn try_from_missing_return_fails() {
        let item_methods: Vec<syn::ImplItemMethod> = vec![
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
    fn try_from_invalid_return_fails() {
        let item_methods: Vec<syn::ImplItemMethod> = vec![
            syn::parse_quote! {
                #[ink(constructor)]
                fn my_constructor() -> &Self {}
            },
            syn::parse_quote! {
                #[ink(constructor)]
                pub fn my_constructor() -> &mut Self {}
            },
            syn::parse_quote! {
                #[ink(constructor)]
                pub fn my_constructor() -> i32 {}
            },
            syn::parse_quote! {
                #[ink(constructor)]
                pub fn my_constructor() -> Result<Self, ()> {}
            },
        ];
        for item_method in item_methods {
            assert_try_from_fails(item_method, "ink! constructors must return Self")
        }
    }

    #[test]
    fn try_from_invalid_self_receiver_fails() {
        let item_methods: Vec<syn::ImplItemMethod> = vec![
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
        let item_methods: Vec<syn::ImplItemMethod> = vec![
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
        let item_methods: Vec<syn::ImplItemMethod> = vec![
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
        let item_methods: Vec<syn::ImplItemMethod> = vec![
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
        let item_methods: Vec<syn::ImplItemMethod> = vec![
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
        let item_methods: Vec<syn::ImplItemMethod> = vec![
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
            assert_try_from_fails(item_method, "ink! constructors must have explicit ABI")
        }
    }

    #[test]
    fn try_from_variadic_fails() {
        let item_methods: Vec<syn::ImplItemMethod> = vec![
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
        let item_methods: Vec<syn::ImplItemMethod> = vec![
            syn::parse_quote! {
                #[ink(constructor)]
                crate fn my_constructor() -> Self {}
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
}
