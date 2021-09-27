use ink_lang as ink;
use incrementer::Incrementer;

#[ink::contract]
mod incrementer {
    #[ink(storage)]
    pub struct Incrementer {
        value: i64,
    }

    impl Incrementer {
        #[ink(constructor)]
        pub fn new(init_value: i64) -> Self {
            Self {
                value: init_value,
            }
        }

        #[ink(message)]
        pub fn inc_by(&mut self, delta: i64) {
            self.value += delta;
        }

        #[ink(message)]
        pub fn get(&self) -> i64 {
            self.value
        }
    }
}

fn main() {
    let mut incrementer = Incrementer::new(0);
    assert_eq!(incrementer.get(), 0);
    incrementer.inc_by(1);
    assert_eq!(incrementer.get(), 1);
    incrementer.inc_by(-1);
    assert_eq!(incrementer.get(), 0);
}
