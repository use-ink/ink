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
        pub fn message(&self) {
            self.message_impl();
        }
    }

    #[ink(impl)]
    impl Contract {
        fn constructor_impl() -> Self {
            let _ = Self::env().caller();
            Self {}
        }

        fn message_impl(&self) {
            let _ = self.env().caller();
        }
    }
}

fn main() {}
