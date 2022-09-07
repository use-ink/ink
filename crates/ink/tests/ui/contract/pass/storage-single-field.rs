use ink_lang as ink;

#[ink::contract]
mod contract {
    #[ink(storage)]
    pub struct Contract {
        field: i8,
    }

    impl Contract {
        #[ink(constructor)]
        pub fn constructor() -> Self {
            Self { field: 0 }
        }

        #[ink(message)]
        pub fn message(&self) {}
    }
}

fn main() {}
