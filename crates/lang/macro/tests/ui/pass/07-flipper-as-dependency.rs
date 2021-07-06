use ink_lang as ink;

#[ink::trait_definition]
pub trait FlipperTrait {
    #[ink(constructor)]
    fn new() -> Self;

    #[ink(message)]
    fn flip(&mut self);

    #[ink(message)]
    fn get(&self) -> bool;
}

#[ink::contract(compile_as_dependency = true)]
mod flipper {
    use super::FlipperTrait;

    #[ink(storage)]
    pub struct Flipper {
        value: bool,
    }

    impl Flipper {
        #[ink(constructor)]
        pub fn new(init_value: bool) -> Self {
            Self { value: init_value }
        }

        #[ink(constructor)]
        pub fn default() -> Self {
            Self::new(false)
        }

        #[ink(message)]
        pub fn flip2(&mut self) {
            self.value = !self.value;
        }

        #[ink(message)]
        pub fn get2(&self) -> bool {
            self.value
        }
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
