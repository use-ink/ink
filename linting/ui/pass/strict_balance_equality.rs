#[ink::contract]
pub mod strict_balance_equality {
    #[ink(storage)]
    pub struct StrictBalanceEquality {}

    impl StrictBalanceEquality {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {}
        }

        fn check_balance(&self, value: &Balance, threshold: &Balance) -> bool {
            // Good: Non-strict equality
            return value > threshold
        }

        fn get_balance(&self, value: &mut Balance) {
            value = self.env().balance()
        }

        #[ink(message)]
        pub fn do_nothing(&mut self) {
            let threshold: Balance = 100;
            let value: Balance = self.env().balance();
            let value_plus_one: Balance = value + 1; // TODO

            let mut value_indirect: Balance = 0;
            self.get_balance(&mut value_indirect);

            // Good: Non-strict equality
            if self.env().balance() < threshold {
                // Do nothing
            }

            // Good: Non-strict equality
            let equal: bool = self.env().balance() < threshold;
            if equal {
                // Do nothing
            }

            if self.check_balance(&self.env().balance(), &threshold) {
                // Do nothing
            }

            if self.check_balance(&value, &threshold) {
                // Do nothing
            }
        }
    }
}

fn main() {}
