use ink_lang as ink;

#[ink::contract]
mod noop {
    #[ink(storage)]
    pub struct Noop {}

    impl Noop {
        #[ink(constructor)]
        pub fn invalid_self_val(self) -> Self {
            Self {}
        }

        #[ink(message)]
        pub fn noop(&self) {}
    }
}

fn main() {}
