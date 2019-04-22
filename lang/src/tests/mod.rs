// Copyright 2018-2019 Parity Technologies (UK) Ltd.
// This file is part of ink!.
//
// ink! is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// ink! is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with ink!.  If not, see <http://www.gnu.org/licenses/>.

mod flipper;
mod incrementer;
mod noop;
mod utils;

pub(crate) use crate::contract_gen_impl2;
pub(crate) use quote::quote;
pub(crate) use utils::{
    assert_eq_tokenstreams,
    assert_failure,
};

#[test]
fn empty_contract_input() {
    assert!(contract_gen_impl2(quote! {}).is_err());
}

#[test]
fn using_self_val_in_message() {
    assert_failure(
        quote! {
            struct TestContract {}
            impl Deploy for TestContract {
                fn deploy(&mut self) {}
            }
            impl TestContract {
                pub(external) fn with_self_value(self) {}
            }
        },
        "contract messages must operate on `&self` or `&mut self`",
    )
}

#[test]
fn using_non_self_in_message() {
    assert_failure(
        quote! {
            struct TestContract {}
            impl Deploy for TestContract {
                fn deploy(&mut self) {}
            }
            impl TestContract {
                pub(external) fn with_self_value(not_self: u32) {}
            }
        },
        "contract messages must operate on `&self` or `&mut self`",
    )
}

#[test]
fn using_empty_message_args() {
    assert_failure(
        quote! {
            struct TestContract {}
            impl Deploy for TestContract {
                fn deploy(&mut self) {}
            }
            impl TestContract {
                pub(external) fn with_self_value() {}
            }
        },
        "contract messages must operate on `&self` or `&mut self`",
    )
}

#[test]
fn using_self_val_in_deploy() {
    assert_failure(
        quote! {
            struct TestContract {}
            impl Deploy for TestContract {
                fn deploy(self) {}
            }
            impl TestContract {}
        },
        "the deploy implementation must operate on `&mut self`",
    )
}

#[test]
fn using_self_ref_in_deploy() {
    assert_failure(
        quote! {
            struct TestContract {}
            impl Deploy for TestContract {
                fn deploy(&self) {}
            }
            impl TestContract {}
        },
        "the deploy implementation must operate on `&mut self`",
    )
}

#[test]
fn missing_state_in_contract() {
    assert_failure(
        quote! {
            impl Deploy for TestContract {
                fn deploy(self) {}
            }
            impl TestContract {}
        },
        "couldn't find a contract state `struct`",
    )
}

#[test]
fn missing_deploy_impl_block() {
    assert_failure(
        quote! {
            struct TestContract {}
            impl TestContract {}
        },
        "couldn't find a contract deploy implementation; requires exactly one",
    )
}

#[test]
fn env_as_deploy_handler_arg() {
    assert_failure(
        quote! {
            struct TestContract {}
            impl Deploy for TestContract {
                fn deploy(&mut self, env: u32) {}
            }
        },
        "the deploy implementation must not contain an argument named `env`",
    )
}

#[test]
fn generic_deploy_handler() {
    assert_failure(
        quote! {
            struct TestContract {}
            impl Deploy for TestContract {
                fn deploy<T>(&mut self, generic_param: T) {}
            }
        },
        "expected parentheses", // The check for this is built into the parser.
    )
}

#[test]
fn deploy_handler_with_return_type() {
    assert_failure(
        quote! {
            struct TestContract {}
            impl Deploy for TestContract {
                fn deploy(&mut self) -> u32 {}
            }
        },
        "the deploy implementation must not have a return type",
    )
}

#[test]
fn env_as_message_arg() {
    assert_failure(
        quote! {
            struct TestContract {}
            impl Deploy for TestContract {
                fn deploy(&mut self) {}
            }
            impl TestContract {
                pub(external) fn test_message(&self, env: u32) {}
            }
        },
        "contract messages must not contain an argument called `env`",
    )
}

#[test]
fn message_called_deploy() {
    assert_failure(
        quote! {
            struct TestContract {}
            impl Deploy for TestContract {
                fn deploy(&mut self) {}
            }
            impl TestContract {
                pub(external) fn deploy(&mut self) {}
            }
        },
        "contract messages must not be named `deploy`",
    )
}

#[test]
fn method_called_deploy() {
    assert_failure(
        quote! {
            struct TestContract {}
            impl Deploy for TestContract {
                fn deploy(&mut self) {}
            }
            impl TestContract {
                fn deploy(&mut self) {}
            }
        },
        "contract methods must not be named `deploy`",
    )
}

#[test]
fn multiple_states() {
    assert_failure(
        quote! {
            struct TestContract1 {}
            struct TestContract2 {}
            impl Deploy for TestContract1 {
                fn deploy(&mut self) {}
            }
            impl Deploy for TestContract2 {
                fn deploy(&mut self) {}
            }
        },
        "requires exactly one contract state `struct`; found 2",
    )
}

#[test]
fn multiple_deploy_handlers() {
    assert_failure(
        quote! {
            struct TestContract {}
            impl Deploy for TestContract {
                fn deploy(&mut self) {}
            }
            impl Deploy for TestContract {
                fn deploy(&mut self) {}
            }
        },
        "found more than one contract deploy implementation for TestContract",
    )
}
