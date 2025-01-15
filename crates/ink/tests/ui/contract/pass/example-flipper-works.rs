#![allow(unexpected_cfgs)]

use flipper::Flipper;

#[ink::contract]
mod flipper {
    #[ink(storage)]
    pub struct Flipper {
        value: bool,
    }

    impl Flipper {
        #[ink(constructor)]
        pub fn new(init_value: bool) -> Self {
            Self { value: init_value }
        }

        #[ink(message)]
        pub fn flip(&mut self) {
            self.value = !self.value;
        }

        #[ink(message)]
        pub fn get(&self) -> bool {
            self.value
        }
    }
}

fn main() {
    let mut flipper = Flipper::new(false);
    assert!(!flipper.get());
    flipper.flip();
    assert!(flipper.get());
}
