use ink_lang as ink;
use ink_lang::{
    reflect::{
        ContractConstructorExecutor,
        ContractMessageExecutor,
        ExecuteDispatchable,
    },
    selector_bytes,
};
use scale::Encode;

#[ink::contract]
pub mod contract {
    #[ink(storage)]
    pub struct Contract {}

    impl Contract {
        #[ink(constructor)]
        pub fn constructor(_input_1: bool, _input_2: i32) -> Self {
            Self {}
        }

        #[ink(message)]
        pub fn message(&self, _input_1: bool, _input_2: i32) {}
    }
}

use contract::Contract;
use std::panic;

fn panic_eq<F: FnOnce() -> R + panic::UnwindSafe, R>(f: F, message: &'static str) {
    let prev_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            assert_eq!(s.as_str(), message);
        } else if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            assert_eq!(s, &message);
        } else {
            panic!("Unknown panic");
        }
    }));
    let result = panic::catch_unwind(f);
    assert!(result.is_err());
    panic::set_hook(prev_hook);
}

fn main() {
    constructor_executor_works();
    message_executor_works()
}

fn constructor_executor_works() {
    // Valid call to `constructor`:
    {
        let mut input_bytes = Vec::new();
        input_bytes.extend(selector_bytes!("constructor"));
        input_bytes.extend(true.encode());
        input_bytes.extend(42i32.encode());
        assert!(
            <<Contract as ContractConstructorExecutor>::Type as ExecuteDispatchable>
                ::execute_dispatchable(&input_bytes[..]).is_ok()
        );
    }
    // Invalid call with invalid selector (or empty input).
    {
        let input_bytes = Vec::new();
        panic_eq(
            || {
                <<Contract as ContractConstructorExecutor>::Type
                    as ExecuteDispatchable>::execute_dispatchable(&input_bytes[..])
            },
            "dispatching ink! constructor failed: unable to decode selector",
        );
    }
    // Invalid call to `message` with unknown selector.
    {
        let mut input_bytes = Vec::new();
        input_bytes.extend(selector_bytes!("unknown_selector"));
        panic_eq(
            || {
                <<Contract as ContractConstructorExecutor>::Type
                    as ExecuteDispatchable>::execute_dispatchable(&input_bytes[..])
            },
            "dispatching ink! constructor failed: encountered unknown selector",
        );
    }
    // Invalid call to `message` with invalid (or missing) parameters.
    {
        let mut input_bytes = Vec::new();
        input_bytes.extend(selector_bytes!("constructor"));
        panic_eq(
            || {
                <<Contract as ContractConstructorExecutor>::Type
                    as ExecuteDispatchable>::execute_dispatchable(&input_bytes[..])
            },
            "dispatching ink! constructor failed: unable to decode input",
        );
    }
}

fn message_executor_works() {
    // Valid case at the end of the function because it will exit from the program

    // Invalid call with invalid selector (or empty input).
    {
        let input_bytes = Vec::new();
        panic_eq(
            || {
                <<Contract as ContractMessageExecutor>::Type
                    as ExecuteDispatchable>::execute_dispatchable(&input_bytes[..])
            },
            "dispatching ink! message failed: unable to decode selector",
        );
    }
    // Invalid call to `message` with unknown selector.
    {
        let mut input_bytes = Vec::new();
        input_bytes.extend(selector_bytes!("unknown_selector"));
        panic_eq(
            || {
                <<Contract as ContractMessageExecutor>::Type
                    as ExecuteDispatchable>::execute_dispatchable(&input_bytes[..])
            },
            "dispatching ink! message failed: encountered unknown selector",
        );
    }
    // Invalid call to `message` with invalid (or missing) parameters.
    {
        let mut input_bytes = Vec::new();
        input_bytes.extend(selector_bytes!("message"));
        panic_eq(
            || {
                <<Contract as ContractMessageExecutor>::Type
                    as ExecuteDispatchable>::execute_dispatchable(&input_bytes[..])
            },
            "dispatching ink! message failed: unable to decode input",
        );
    }
    // Valid call to `message`:
    {
        let mut input_bytes = Vec::new();
        input_bytes.extend(selector_bytes!("message"));
        input_bytes.extend(true.encode());
        input_bytes.extend(42i32.encode());
        assert!(
            <<Contract as ContractMessageExecutor>::Type as ExecuteDispatchable>
            ::execute_dispatchable(&input_bytes[..]).is_ok()
        );
    }
}
