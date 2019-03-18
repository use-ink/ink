mod utils;
mod noop;
mod incrementer;

pub(crate) use quote::quote;
pub(crate) use utils::{
    assert_eq_tokenstreams,
    assert_failure,
};
pub(crate) use crate::contract_gen_impl2;

#[test]
fn empty_contract_input() {
    assert!(contract_gen_impl2(quote!{}).is_err());
}

#[test]
fn using_self_val_in_message() {
    assert_failure(
        quote!{
            struct TestContract {}
            impl Deploy for TestContract {
                fn deploy(&mut self) {}
            }
            impl TestContract {
                pub(external) fn with_self_value(self) {}
            }
        },
        "contract messages must start with `&self` or `&mut self`"
    )
}

#[test]
fn using_self_val_in_deploy() {
    assert_failure(
        quote!{
            struct TestContract {}
            impl Deploy for TestContract {
                fn deploy(self) {}
            }
            impl TestContract {}
        },
        "the deploy implementation must operate on `&mut self`"
    )
}

#[test]
fn using_self_ref_in_deploy() {
    assert_failure(
        quote!{
            struct TestContract {}
            impl Deploy for TestContract {
                fn deploy(&self) {}
            }
            impl TestContract {}
        },
        "the deploy implementation must operate on `&mut self`"
    )
}

#[test]
fn missing_state_in_contract() {
    assert_failure(
        quote!{
            impl Deploy for TestContract {
                fn deploy(self) {}
            }
            impl TestContract {}
        },
        "couldn't find a contract state `struct`"
    )
}

#[test]
fn missing_deploy_impl_block() {
    assert_failure(
        quote! {
            struct TestContract {}
            impl TestContract {}
        },
        "couldn't find a contract deploy implementation; requires exactly one"
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
        "the deploy implementation must not contain an argument named `env`"
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
        "expected parentheses" // The check for this is built into the parser.
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
        "the deploy implementation must not have a return type"
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
        "contract messages must not contain an argument called `env`"
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
        "contract messages must not be named `deploy`"
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
        "contract methods must not be named `deploy`"
    )
}
