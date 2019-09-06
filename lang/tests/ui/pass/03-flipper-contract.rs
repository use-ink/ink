#![feature(proc_macro_hygiene)]

use ink_lang as ink;

#[ink::contract]
mod flipper {
    #![ink(env = DefaultSrmlTypes)]

    #[ink(storage)]
    struct Flipper {
        value: storage::Value<bool>,
    }

    #[ink(event)]
    struct Flipped {
        #[indexed]
        current: bool,
        #[indexed]
        by: AccountId,
    }

    impl Flipper {
        #[ink(constructor)]
        fn new(&mut self, init_value: bool) {
            self.value.set(init_value);
        }

        #[ink(constructor)]
        fn default(&mut self) {
            self.new(false)
        }

        #[ink(message)]
        fn flip(&mut self) {
            let current = !self.get();
            let by = self.env.caller();
            self.env.emit_event(Flipper { current, by });
            *self.value = current;
        }

        #[ink(message)]
        fn get(&self) -> bool {
            *self.value
        }
    }
}

fn main() {}
