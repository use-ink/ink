#![cfg_attr(not(feature = "std"), no_main)]

#[ink::contract]
pub mod strict_balance_equality {
    use ink::U256;

    #[ink(storage)]
    pub struct StrictU256Equality {}

    impl StrictU256Equality {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {}
        }

        // Return value tainted with balance
        fn get_balance_1(&self) -> U256 {
            self.env().balance()
        }
        fn get_balance_2(&self) -> U256 {
            let tmp = self.env().balance();
            tmp
        }
        fn get_balance_3(&self) -> U256 {
            let tmp = self.env().balance();
            tmp + 42
        }
        fn get_balance_recursive(&self, acc: &U256) -> U256 {
            if acc < &U256::from(10) {
                self.get_balance_recursive(&(acc + 1))
            } else {
                self.env().balance()
            }
        }

        // Return the result of comparison with balance
        fn cmp_balance_1(&self, value: &U256) -> bool {
            *value == self.env().balance()
        }
        fn cmp_balance_2(&self, value: &U256, threshold: &U256) -> bool {
            value != threshold
        }
        fn cmp_balance_3(&self, value: U256, threshold: U256) -> bool {
            value != threshold
        }

        // Tainted `&mut` input argument
        fn get_balance_arg_1(&self, value: &mut U256) {
            *value = self.env().balance();
        }
        fn get_balance_arg_indirect(&self, value: &mut U256) {
            self.get_balance_arg_1(value)
        }

        #[ink(message)]
        pub fn do_nothing(&mut self) {
            let threshold = U256::from(100);
            let value: U256 = self.env().balance();

            // Bad: Strict equality with balance
            if self.env().balance() == U256::from(10) { /* ... */ }
            if value == 11.into() { /* ... */ }
            if self.env().balance() == threshold { /* ... */ }

            // Bad: Strict equality in function call: return value
            if self.get_balance_1() == 10.into() { /* ... */ }
            if self.get_balance_2() == U256::from(10) { /* ... */ }
            if self.get_balance_3() == U256::from(10) { /* ... */ }
            if self.get_balance_recursive(&10.into()) == 10.into() { /* ... */ }

            // Bad: Strict equality in function call: return value contains the result of
            // comparison
            if self.cmp_balance_1(&U256::from(10)) { /* ... */ }
            if self.cmp_balance_2(&self.env().balance(), &threshold) { /* ... */ }
            if self.cmp_balance_3(self.env().balance(), threshold) { /* ... */ }

            // Bad: Strict equality in function: tainted arguments
            let mut res_1 = U256::zero();
            self.get_balance_arg_1(&mut res_1);
            if res_1 == U256::from(10) { /* ... */ }
            let mut res_2 = U256::from(0);
            self.get_balance_arg_indirect(&mut res_2);
            if res_2 == 10.into() { /* ... */ }
        }
    }
}

fn main() {}
