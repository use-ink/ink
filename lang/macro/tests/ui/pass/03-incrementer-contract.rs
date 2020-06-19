use ink_lang as ink;

#[ink::contract(version = "0.1.0")]
mod incrementer {
    #[ink(storage)]
    struct Incrementer {
        value: i64,
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
        fn new(init_value: i32) -> Self {
            Self {
                value: init_value as i64,
            }
        }

        #[ink(constructor)]
        fn default() -> Self {
            Self::new(0)
        }

        #[ink(message)]
        fn inc_by(&mut self, by: i32) {
            let caller = self.env().caller();
            self.env().emit_event(Incremented { caller, by });
            self.value += by as i64;
        }

        #[ink(message)]
        fn get(&self) -> i64 {
            self.value
        }
    }
}

fn main() {}
