use ink_lang as ink;

#[ink::contract(keep_attr = "foo, bar")]
mod contract {
    #[ink(storage)]
    pub struct Contract {}

    impl Contract {
        #[ink(constructor)]
        pub fn constructor() -> Self {
            Self {}
        }

        #[ink(message)]
        #[allow(non_snake_case)]
        pub fn meSSage(&self) {}
    }
}

fn main() {}
