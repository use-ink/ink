use ink_lang as ink;

#[ink::trait_definition]
pub trait Foo {
    #[ink(constructor)]
    fn new() -> Self;
}

#[ink::contract]
mod noop {
    #[ink(storage)]
    pub struct Noop {}

    impl Foo for Noop {
        #[ink(constructor, payable)]
        fn new() -> Self {
            Self {}
        }
    }
}

fn main() {}
