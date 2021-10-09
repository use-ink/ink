use flipper::Flipper;
use ink_lang as ink;

#[ink::trait_definition]
pub trait Flip {
    #[ink(message)]
    fn flip(&mut self);

    #[ink(message)]
    fn get(&self) -> bool;
}

#[ink::contract]
mod flipper {
    use super::Flip;

    #[ink(storage)]
    pub struct Flipper {
        value: bool,
    }

    impl Flipper {
        #[ink(constructor)]
        pub fn new(init_value: bool) -> Self {
            Self { value: init_value }
        }
    }

    impl Flip for Flipper {
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

fn main() {
    // We use the verbose universal call syntax for trait methods
    // in order to make sure that the trait flipper example actually
    // implements its messages as Rust traits.
    let mut flipper = Flipper::new(false);
    assert!(!<Flipper as Flip>::get(&flipper));
    <Flipper as Flip>::flip(&mut flipper);
    assert!(<Flipper as Flip>::get(&flipper));
}
