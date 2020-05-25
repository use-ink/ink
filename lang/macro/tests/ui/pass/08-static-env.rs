use ink_lang as ink;

#[ink::contract(version = "0.1.0")]
mod static_env {
    #[ink(storage)]
    struct StaticEnv {}

    impl StaticEnv {
        #[ink(constructor)]
        fn new(&mut self) {
        }

        #[ink(message)]
        fn gas_left(&mut self) -> Balance {
            Self::env().gas_left()
        }
    }
}

fn main() {}
