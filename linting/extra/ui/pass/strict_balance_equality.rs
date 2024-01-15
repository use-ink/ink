#![cfg_attr(not(feature = "std"), no_main)]

#[ink::contract]
pub mod strict_balance_equality {
    #[ink(storage)]
    pub struct StrictBalanceEquality {}

    impl StrictBalanceEquality {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {}
        }

        // Return value tainted with balance
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

        // Return the result of non-strict comparison with balance
        fn cmp_balance_1(&self, value: &Balance) -> bool {
            *value < self.env().balance()
        }
        fn cmp_balance_2(&self, value: &Balance, threshold: &Balance) -> bool {
            value > threshold
        }
        fn cmp_balance_3(&self, value: Balance, threshold: Balance) -> bool {
            value >= threshold
        }

        // `&mut` input argument gets the balance value
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

            // Good: Non-strict equality with balance
            if self.env().balance() < 10 { /* ... */ }
            if value > 11 { /* ... */ }
            if self.env().balance() < threshold { /* ... */ }

            // Good: Non-strict equality in function call: return value
            if self.get_balance_1() < 10 { /* ... */ }
            if self.get_balance_2() > 10 { /* ... */ }
            if self.get_balance_3() >= 10 { /* ... */ }
            if self.get_balance_recursive(&10) <= 10 { /* ... */ }

            // Good: Non-strict equality in function call: return value contains the
            // result of comparison
            if self.cmp_balance_1(&10) { /* ... */ }
            if self.cmp_balance_2(&self.env().balance(), &threshold) { /* ... */ }
            if self.cmp_balance_3(self.env().balance(), threshold) { /* ... */ }

            // Good: Non-strict equality in function: tainted arguments
            let mut res_1 = 0_u128;
            self.get_balance_arg_1(&mut res_1);
            if res_1 < 10 { /* ... */ }
            let mut res_2 = 0_u128;
            self.get_balance_arg_indirect(&mut res_2);
            if res_2 > 10 { /* ... */ }

            // Good: warning is suppressed
            #[cfg_attr(dylint_lib = "ink_linting", allow(strict_balance_equality))]
            if self.env().balance() == 10 { /* ... */ }
        }
    }
}

fn main() {}
