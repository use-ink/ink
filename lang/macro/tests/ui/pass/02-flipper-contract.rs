use ink_lang as ink;

#[ink::contract(version = "0.1.0")]
mod flipper {
    #[ink(storage)]
    struct Flipper {
        value: bool,
    }

    impl Flipper {
        #[ink(constructor)]
        fn new(init_value: bool) -> Self {
            Self {
                value: init_value,
            }
        }

        #[ink(constructor)]
        fn default() -> Self {
            Self::new(false)
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

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn it_works() {
            let mut flipper = Flipper::new(false);
            assert_eq!(flipper.get(), false);
            flipper.flip();
            assert_eq!(flipper.get(), true);
        }
    }
}

fn main() {}
