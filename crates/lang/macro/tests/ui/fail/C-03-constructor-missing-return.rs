use ink_lang as ink;

#[ink::contract]
mod noop {
    #[ink(storage)]
    pub struct Noop {}

    impl Noop {
        #[ink(constructor)]
        pub fn missing_return() {}

        #[ink(message)]
        pub fn noop(&self) {}
    }
}

fn main() {}
