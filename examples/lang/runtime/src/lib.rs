#![no_std]

use pdsl_core::{
    Address,
    BalancesCall, Call,
    memory::format,
};
use pdsl_lang::contract;

contract! {
    /// A simple contract to test calls into the runtime
    struct Runtime {}

    impl Deploy for Runtime {
        fn deploy(&mut self) {
        }
    }

    impl Runtime {
        /// Transfer the specified amount to the indexed address
        pub(external) fn balance_transfer(&mut self, dest: Address, value: u128) {
            env.println(&format!("Runtime::balance_transfer"));
            env.dispatch_call(Call::Balances(BalancesCall::transfer(dest, value)))
        }
    }
}
