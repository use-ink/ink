use incrementer::Incrementer;
use ink_lang as ink;

#[ink::trait_definition]
pub trait Increment {
    #[ink(message)]
    fn inc(&mut self);

    #[ink(message)]
    fn get(&self) -> i64;
}

#[ink::trait_definition]
pub trait Reset {
    #[ink(message)]
    fn reset(&mut self);
}

#[ink::contract]
mod incrementer {
    use super::{
        Increment,
        Reset,
    };

    #[ink(storage)]
    pub struct Incrementer {
        value: i64,
    }

    impl Incrementer {
        #[ink(constructor)]
        pub fn new(init_value: i64) -> Self {
            Self { value: init_value }
        }

        #[ink(message)]
        pub fn inc_by(&mut self, delta: i64) {
            self.value += delta;
        }
    }

    impl Increment for Incrementer {
        #[ink(message)]
        fn inc(&mut self) {
            self.inc_by(1)
        }

        #[ink(message)]
        fn get(&self) -> i64 {
            self.value
        }
    }

    impl Reset for Incrementer {
        #[ink(message)]
        fn reset(&mut self) {
            self.value = 0;
        }
    }
}

fn main() {
    let mut incrementer = Incrementer::new(0);
    assert_eq!(<Incrementer as Increment>::get(&incrementer), 0);
    incrementer.inc_by(1);
    assert_eq!(<Incrementer as Increment>::get(&incrementer), 1);
    incrementer.inc_by(-1);
    assert_eq!(<Incrementer as Increment>::get(&incrementer), 0);

    <Incrementer as Increment>::inc(&mut incrementer);
    assert_eq!(<Incrementer as Increment>::get(&incrementer), 1);
    <Incrementer as Increment>::inc(&mut incrementer);
    assert_eq!(<Incrementer as Increment>::get(&incrementer), 2);
}
