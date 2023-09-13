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
            // Bad: Strict equality
            value != threshold
        }

        fn get_balance_1(&self) -> Balance {
            self.env().balance()
        }
        fn get_balance_2(&self) -> Balance {
            let tmp = self.env().balance();
            tmp
        }
        fn get_balance_3(&self) -> Balance {
            let tmp = self.env().balance();
            tmp + 1
        }
        fn get_balance_recursive(&self, acc: &Balance) -> Balance {
            if acc < &10_u128 {
                self.get_balance_recursive(&(acc + 1))
            } else {
                self.env().balance()
            }
        }

        fn cmp_balance(&self, value: &Balance) -> bool {
            *value == self.env().balance()
        }

        #[ink(message)]
        pub fn do_nothing(&mut self) {
            let threshold: Balance = 100;
            let value: Balance = self.env().balance();

            // Bad: Strict equality
            if self.env().balance() == 10 { /* ... */ }
            if value == 11 { /* ... */ }
            if self.env().balance() == threshold { /* ... */ }

            // Bad: Strict equality in function call: return value
            if self.get_balance_1() == 10 { /* ... */ }
            if self.get_balance_2() == 10 { /* ... */ }
            //if self.get_balance_3() == 10 { /* ... */ } // TODO: false negative
            if self.get_balance_recursive(&10) == 10 { /* ... */ }

            // Bad: Strict equality in function: tainted arguments
            //if self.cmp_balance(&10) {}
            // if self.check_balance(&self.env().balance(), &threshold) {
            //     // Do nothing
            // }
            // let res = self.check_balance(&self.env().balance(), &threshold);
            // if res {
            //     // Do nothing
            // }
            // if self.check_balance(&value, &threshold) {
            //     // Do nothing
            // }
        }
    }
}

fn main() {}
