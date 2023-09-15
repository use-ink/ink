#[ink::contract]
pub mod strict_balance_equality {
    #[ink(storage)]
    pub struct StrictBalanceEquality {}

    impl StrictBalanceEquality {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {}
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
            tmp + 42
        }
        fn get_balance_recursive(&self, acc: &Balance) -> Balance {
            if acc < &10_u128 {
                self.get_balance_recursive(&(acc + 1))
            } else {
                self.env().balance()
            }
        }
        fn cmp_balance_1(&self, value: &Balance) -> bool {
            *value == self.env().balance()
        }
        fn cmp_balance_2(&self, value: &Balance, threshold: &Balance) -> bool {
            value != threshold
        }

        fn get_balance_arg_1(&self, value: &mut Balance) {
            *value = self.env().balance();
        }
        fn get_balance_arg_indirect(&self, value: &mut Balance) {
            self.get_balance_arg_1(value)
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
            if self.get_balance_3() == 10 { /* ... */ }
            if self.get_balance_recursive(&10) == 10 { /* ... */ }
            // if self.cmp_balance_1(&10) { /* ... */ } // TODO: false negative
            // if self.cmp_balance_2(&self.env().balance(), &threshold) { /* ... */ } // TODO: false negative

            // Bad: Strict equality in function: tainted arguments
            let mut res_1 = 0_u128;
            self.get_balance_arg_1(&mut res_1);
            if res_1 == 10 { /* ... */ }
            let mut res_2 = 0_u128;
            self.get_balance_arg_indirect(&mut res_2);
            if res_2 == 10 { /* ... */ }
        }
    }
}

fn main() {}
