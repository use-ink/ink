use ink_lang as ink;

#[ink::contract]
mod incrementer {
    #[ink(storage)]
    pub struct Incrementer {
        value: i64,
    }

    #[ink(event)]
    pub struct Incremented {
        #[ink(topic)]
        caller: AccountId,
        #[ink(topic)]
        by: i32,
    }

    impl Incrementer {
        #[ink(constructor)]
        pub fn new(init_value: i32) -> Self {
            Self {
                value: init_value as i64,
            }
        }

        #[ink(constructor)]
        pub fn default() -> Self {
            Self::new(0)
        }

        #[ink(message)]
        pub fn inc_by(&mut self, by: i32) {
            let caller = self.env().caller();
            self.env().emit_event(Incremented { caller, by });
            self.value += by as i64;
        }

        #[ink(message)]
        pub fn get(&self) -> i64 {
            self.value
        }
    }
}

fn main() {}
