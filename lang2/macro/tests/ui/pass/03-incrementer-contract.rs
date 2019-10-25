#![feature(proc_macro_hygiene)]

use ink_core::storage;
use ink_lang2 as ink;

#[ink::contract(version = "0.1.0")]
mod incrementer {
    #[ink(storage)]
    struct Incrementer {
        value: storage::Value<i64>,
    }

    #[ink(event)]
    struct Incremented {
        #[indexed]
        caller: AccountId,
        #[indexed]
        by: i32,
    }

    impl Flipper {
        #[ink(constructor)]
        fn new(&mut self, init_value: i32) {
            self.value.set(init_value as i64);
        }

        #[ink(constructor)]
        fn default(&mut self) {
            self.new(0)
        }

        #[ink(message)]
        fn inc_by(&mut self, by: i32) {
            self.env().emit_event(Flipper {
                caller: self.env().caller(),
                by,
            });
            *self.value += by as i64;
        }

        #[ink(message)]
        fn get(&self) -> i64 {
            *self.value
        }
    }
}

fn main() {}
