use ink_lang as ink;

#[ink::contract(version = "0.1.0")]
mod static_env {
    #[ink(storage)]
    struct StaticEnv {}

    impl StaticEnv {
        #[ink(constructor)]
        fn new() -> Self {
            assert!(Self::env().balance() > 0);
            Self {}
        }

        #[ink(message)]
        fn gas_left(&mut self) -> Balance {
            Self::env().gas_left()
        }
    }
}

fn main() {}
