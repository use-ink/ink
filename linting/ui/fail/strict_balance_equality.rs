#[ink::contract]
pub mod strict_balance_equality {
    #[ink(storage)]
    pub struct StrictBalanceEquality {}

    impl StrictBalanceEquality {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {}
        }

        // fn check_balance(&self, value: &Balance, threshold: &Balance) -> bool {
        //     // Bad: Strict equality
        //     return value != threshold
        // }

        #[ink(message)]
        pub fn do_nothing(&mut self) {
            // let threshold: Balance = 100;
            let value: Balance = self.env().balance();

            if self.env().balance() == 10 { // Bad
                // Do nothing
            }

            // Bad: Strict equality
            if value == 11 {
                // Do nothing
            }

            //
            // // Bad: Strict equality
            // if self.env().balance() == threshold {
            //     // Do nothing
            // }
            //
            // if self.check_balance(&self.env().balance(), &threshold) {
            //     // Do nothing
            // }
            //
            // if self.check_balance(&value, &threshold) {
            //     // Do nothing
            // }
        }
    }
}

fn main() {}
