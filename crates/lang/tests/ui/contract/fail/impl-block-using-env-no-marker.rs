use ink_lang as ink;

#[ink::contract]
mod contract {
    #[ink(storage)]
    pub struct Contract {}

    impl Contract {
        #[ink(constructor)]
        pub fn constructor() -> Self {
            Self {}
        }
    }

    impl Contract {
        #[ink(message)]
        pub fn message(&self) {
            self.message_impl();
        }
    }

    impl Contract {
        fn message_impl(&self) {
            let _ = self.env().caller();
        }
    }
}

fn main() {}
