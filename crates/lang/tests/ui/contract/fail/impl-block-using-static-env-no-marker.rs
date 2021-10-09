use ink_lang as ink;

#[ink::contract]
mod contract {
    #[ink(storage)]
    pub struct Contract {}

    impl Contract {
        #[ink(constructor)]
        pub fn constructor() -> Self {
            Self::constructor_impl()
        }
    }

    impl Contract {
        #[ink(message)]
        pub fn message(&self) {}
    }

    impl Contract {
        fn constructor_impl() -> Self {
            let _ = Self::env().caller();
            Self {}
        }
    }
}

fn main() {}
