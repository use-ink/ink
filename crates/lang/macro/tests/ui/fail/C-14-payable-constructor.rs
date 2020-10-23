use ink_lang as ink;

#[ink::contract]
mod noop {
    #[ink(storage)]
    pub struct Noop {}

    impl Noop {
        #[ink(constructor, payable)]
        pub fn new() -> Self {
            Self {}
        }
    }
}

fn main() {}
