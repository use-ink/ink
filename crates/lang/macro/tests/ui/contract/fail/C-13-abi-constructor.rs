use ink_lang as ink;

#[ink::contract]
mod noop {
    #[ink(storage)]
    pub struct Noop {}

    impl Noop {
        #[ink(constructor)]
        pub extern "C" fn abi_constructor() -> Self {
            Self {}
        }

        #[ink(message)]
        pub fn noop(&self) {}
    }
}

fn main() {}
