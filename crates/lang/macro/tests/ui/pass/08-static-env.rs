use ink_lang as ink;

#[ink::contract]
mod static_env {
    #[ink(storage)]
    pub struct UsesStaticEnv {}

    impl UsesStaticEnv {
        #[ink(constructor)]
        pub fn new() -> Self {
            assert!(Self::env().balance() > 0);
            Self {}
        }

        #[ink(message)]
        pub fn gas_left(&mut self) -> Balance {
            Self::env().gas_left()
        }
    }
}

fn main() {}
