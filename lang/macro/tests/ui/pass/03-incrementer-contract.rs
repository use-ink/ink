use ink_lang as ink;

#[ink::contract(version = "0.1.0")]
mod incrementer {
    use ink_core::storage;

    #[ink(storage)]
    struct Incrementer {
        value: storage::Value<i64>,
    }

    #[ink(event)]
    struct Incremented {
        #[ink(topic)]
        caller: AccountId,
        #[ink(topic)]
        by: i32,
    }

    impl Incrementer {
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
            let caller = self.env().caller();
            self.env().emit_event(Incremented { caller, by });
            *self.value += by as i64;
        }

        #[ink(message)]
        fn get(&self) -> i64 {
            *self.value
        }
    }
}

fn main() {}
