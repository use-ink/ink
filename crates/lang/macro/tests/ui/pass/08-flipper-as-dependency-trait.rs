use ink_lang as ink;

#[ink::contract(compile_as_dependency = true)]
mod flipper {
    #[ink_lang::trait_definition]
    pub trait FlipperTrait {
        #[ink(constructor)]
        fn new() -> Self;

        #[ink(message)]
        fn flip(&mut self);

        #[ink(message)]
        fn get(&self) -> bool;
    }

    #[ink(storage)]
    pub struct Flipper {
        value: bool,
    }

    impl FlipperTrait for Flipper {
        #[ink(constructor)]
        fn new() -> Self {
            Self::default()
        }

        #[ink(message)]
        fn flip(&mut self) {
            self.value = !self.value;
        }

        #[ink(message)]
        fn get(&self) -> bool {
            self.value
        }
    }
}

fn main() {}
