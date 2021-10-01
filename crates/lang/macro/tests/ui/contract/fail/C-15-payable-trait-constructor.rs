use ink_lang as ink;

#[ink::trait_definition]
pub trait Constructor {
    #[ink(constructor)]
    fn constructor() -> Self;
}

#[ink::contract]
mod noop {
    #[ink(storage)]
    pub struct Noop {}

    impl Constructor for Noop {
        #[ink(constructor, payable)]
        fn constructor() -> Self {
            Self {}
        }
    }

    impl Noop {
        #[ink(message)]
        pub fn noop(&self) {}
    }
}

fn main() {}
