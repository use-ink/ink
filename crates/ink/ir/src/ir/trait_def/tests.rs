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

use super::*;

/// Checks if the token stream in `$trait_def` results in the expected error message.
macro_rules! assert_ink_trait_eq_err {
    ( error: $err_str:literal, $($trait_def:tt)* ) => {
        assert_eq!(
            <InkItemTrait as TryFrom<syn::ItemTrait>>::try_from(syn::parse_quote! {
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
        error: "ink! trait definitions with supertraits are not supported, yet",
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
        error: "encountered unknown ink! attribute argument: unknown",
        pub trait MyTrait {
            #[ink(unknown)]
            fn unknown_ink_attribute(&self);
        }
    );
}

#[test]
fn trait_def_containing_invalid_message_is_denied() {
    assert_ink_trait_eq_err!(
        error: "missing `&self` or `&mut self` receiver for ink! message",
        pub trait MyTrait {
            #[ink(message)]
            fn does_not_return_self();
        }
    );
    assert_ink_trait_eq_err!(
        error: "self receiver of ink! message must be `&self` or `&mut self`",
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
            #[ink(anonymous)]
            fn does_not_return_self(&self);
        }
    );
}

#[test]
fn trait_def_is_ok() {
    assert!(
        <InkItemTrait as TryFrom<syn::ItemTrait>>::try_from(syn::parse_quote! {
            pub trait MyTrait {
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
fn trait_def_with_namespace_is_ok() {
    assert!(
        <InkItemTrait as TryFrom<syn::ItemTrait>>::try_from(syn::parse_quote! {
            #[ink(namespace = "my_namespace")]
            pub trait MyTrait {
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
fn trait_def_with_selectors_ok() {
    assert!(
        <InkItemTrait as TryFrom<syn::ItemTrait>>::try_from(syn::parse_quote! {
            pub trait MyTrait {
                #[ink(message, selector = 0xDEADBEEF)]
                fn my_message(&self);
                #[ink(message, selector = 0xC0FEFEED)]
                fn my_message_mut(&mut self);
            }
        })
        .is_ok()
    )
}

#[test]
fn trait_def_with_payable_ok() {
    assert!(
        <InkItemTrait as TryFrom<syn::ItemTrait>>::try_from(syn::parse_quote! {
            pub trait MyTrait {
                #[ink(message, payable)]
                fn my_message_mut(&mut self);
            }
        })
        .is_ok()
    )
}

#[test]
fn trait_def_with_everything_combined_ok() {
    assert!(
        <InkItemTrait as TryFrom<syn::ItemTrait>>::try_from(syn::parse_quote! {
            #[ink(namespace = "my_namespace")]
            pub trait MyTrait {
                #[ink(message)]
                fn my_message_1(&self);
                #[ink(message, selector = 0xDEADBEEF)]
                fn my_message_2(&self);
                #[ink(message)]
                fn my_message_mut_1(&mut self);
                #[ink(message, payable)]
                fn my_message_mut_2(&mut self);
                #[ink(message, payable, selector = 0xC0DEBEEF)]
                fn my_message_mut_3(&mut self);
            }
        })
        .is_ok()
    )
}

#[test]
fn trait_def_with_overlapping_selectors() {
    assert_ink_trait_eq_err!(
        error: "encountered duplicate selector ([c0, de, ca, fe]) \
                in the same ink! trait definition",
        pub trait MyTrait {
            #[ink(message, selector = 0xC0DECAFE)]
            fn my_message(&self);
            #[ink(message, selector = 0xC0DECAFE)]
            fn my_message_mut(&mut self);
        }
    );
}

#[test]
fn iter_messages_works() {
    let ink_trait =
        <InkItemTrait as TryFrom<syn::ItemTrait>>::try_from(syn::parse_quote! {
            pub trait MyTrait {
                #[ink(message)]
                fn message_1(&self);
                #[ink(message)]
                fn message_2(&mut self);
            }
        })
        .unwrap();
    let actual = ink_trait
        .iter_items()
        .map(|(item, _)| item)
        .flat_map(|item| {
            item.filter_map_message()
                .map(|message| message.sig().ident.to_string())
        })
        .collect::<Vec<_>>();
    let expected = vec!["message_1".to_string(), "message_2".to_string()];
    assert_eq!(actual, expected);
}
