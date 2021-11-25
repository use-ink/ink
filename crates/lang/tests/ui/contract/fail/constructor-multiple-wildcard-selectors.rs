use ink_lang as ink;

#[ink::contract]
mod contract {
    #[ink(storage)]
    pub struct Contract {}

    impl Contract {
        #[ink(constructor, selector = _)]
        pub fn constructor1() -> Self {
            Self {}
        }

        #[ink(constructor, selector = _)]
        pub fn constructor2() -> Self {
            Self {}
        }

        #[ink(message)]
        pub fn message(&self) {}
    }
}

fn main() {}
