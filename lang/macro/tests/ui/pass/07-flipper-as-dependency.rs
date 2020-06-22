use ink_lang as ink;

#[ink::contract(
    version = "0.1.0",
    compile_as_dependency = true,
)]
mod flipper {
    #[ink(storage)]
    struct Flipper {
        value: bool,
    }

    impl Flipper {
        #[ink(constructor)]
        fn new(init_value: bool) -> Self {
            Self {
                value: init_value,
            }
        }

        #[ink(constructor)]
        fn default() -> Self {
            Self::new(false)
        }

        #[ink(message)]
        fn flip(&mut self) {
            self.value = !self.value;
        }

        #[ink(message)]
        fn get(&self) -> bool {
            self.value
        }
    }
}

fn main() {}
