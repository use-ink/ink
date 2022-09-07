use ink_lang as ink;

#[ink::contract]
mod contract {
    #[ink(storage)]
    pub struct Contract {}

    impl Contract {
        #[ink(constructor)]
        pub fn constructor_1() -> Self {
            Self {}
        }

        #[ink(message)]
        pub fn message_1(&self) {}
    }

    pub struct NonContract {}

    impl NonContract {
        #[ink(constructor)]
        pub fn constructor_2() -> Self {
            Self {}
        }

        #[ink(message)]
        pub fn message_2(&self) {}
    }
}

fn main() {}
