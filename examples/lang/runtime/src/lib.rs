#![no_std]

use pdsl_core::{
    Address, BalancesCall, Call,
    storage,
    memory::format,
};
use pdsl_lang::contract;

contract! {
    /// A simple contract to test calls into the runtime
    struct Runtime;

    impl Deploy for Incrementer {
        fn deploy(&mut self, init_value: u32) {
        }
    }

    impl Runtime {
        /// Transfer the specified amount to the indexed address
        pub(external) fn balance_transfer(&mut self, dest: u64, value: u64) {
            env.println(&format!("Runtime::balance_transfer"));
            env.dispatch_call(Call::Balances(BalancesCall::transfer(Address::Index(dest), value)))
        }
    }
}
