// Copyright 2018-2019 Parity Technologies (UK) Ltd.
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

mod events;
mod flipper;
mod incrementer;
mod noop;
mod utils;

pub(crate) use crate::generate_or_err;
pub(crate) use quote::quote;
pub(crate) use utils::{
    assert_eq_tokenstreams,
    assert_failure,
};

#[test]
fn empty_contract_input() {
    assert!(generate_or_err(quote! {}).is_err());
}

#[test]
fn missing_env_types_meta() {
    assert_failure(
        quote! {
            struct TestContract {}
            impl Deploy for TestContract {
                fn deploy(&mut self) {}
            }
            impl TestContract {}
        },
        "couldn\'t find an `#![env = <EnvTypesImpl>]` attribute",
    )
}

#[test]
fn multiple_env_types_meta() {
    assert_failure(
        quote! {
            #![env = ink_core::env::DefaultSrmlTypes]
            #![env = ink_core::env::DefaultSrmlTypes]
            struct TestContract {}
            impl Deploy for TestContract {
                fn deploy(&mut self) {}
            }
            impl TestContract {}
        },
        "requires exactly one `#![env = <EnvTypesImpl>]` attribute; found 2",
    )
}

#[test]
fn env_types_meta_wrong_attr_name() {
    assert_failure(
        quote! {
            #![not_env = ink_core::env::DefaultSrmlTypes]
            struct TestContract {}
            impl Deploy for TestContract {
                fn deploy(&mut self) {}
            }
            impl TestContract {}
        },
        "unknown env attribute \'not_env\'",
    )
}

#[test]
fn using_self_val_in_message() {
    assert_failure(
        quote! {
            #![env = ink_core::env::DefaultSrmlTypes]
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
            #![env = ink_core::env::DefaultSrmlTypes]
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
            #![env = ink_core::env::DefaultSrmlTypes]
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
            #![env = ink_core::env::DefaultSrmlTypes]
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
            #![env = ink_core::env::DefaultSrmlTypes]
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
            #![env = ink_core::env::DefaultSrmlTypes]
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
            #![env = ink_core::env::DefaultSrmlTypes]
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
            #![env = ink_core::env::DefaultSrmlTypes]
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
            #![env = ink_core::env::DefaultSrmlTypes]
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
            #![env = ink_core::env::DefaultSrmlTypes]
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
            #![env = ink_core::env::DefaultSrmlTypes]
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
            #![env = ink_core::env::DefaultSrmlTypes]
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
            #![env = ink_core::env::DefaultSrmlTypes]
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
            #![env = ink_core::env::DefaultSrmlTypes]
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
            #![env = ink_core::env::DefaultSrmlTypes]
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
