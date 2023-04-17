#![cfg_attr(not(feature = "std"), no_std)]

pub use self::accumulator::{Accumulator, AccumulatorRef};

#[ink::contract]
pub mod accumulator {
    /// Holds a simple `i32` value that can be incremented and decremented.
    #[ink(storage)]
    pub struct Accumulator {
        value: i32,
    }

    impl Accumulator {
        /// Initializes the value to the initial value.
        #[ink(constructor)]
        pub fn new(init_value: i32) -> Self {
            Self { value: init_value }
        }

        /// Mutates the internal value.
        #[ink(message)]
        pub fn inc(&mut self, by: i32) {
            // Debug message to check whether the contract gets called by the
            // `adder` or `subber` contract.
            let caller = self.env().caller();
            let message = ink::prelude::format!("got a call from {:?}", caller);
            ink::env::debug_println!("{}", &message);

            self.value += by;
        }

        /// Returns the current state.
        #[ink(message)]
        pub fn get(&self) -> i32 {
            self.value
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[ink::test]
        fn get() {
            let accumulator = Accumulator::new(10);
            assert_eq!(accumulator.value, 10);
            assert_eq!(accumulator.value, accumulator.get());
        }

        #[ink::test]
        fn increase() {
            let value = 10;
            let mut accumulator = Accumulator::new(value);
            assert_eq!(accumulator.value, accumulator.get());
            let increase = 10;
            accumulator.inc(increase);
            assert_eq!(accumulator.value, value + increase);
            assert_eq!(accumulator.value, accumulator.get());
        }

        #[ink::test]
        fn decrease() {
            let value = 10;
            let mut accumulator = Accumulator::new(value);
            assert_eq!(accumulator.value, accumulator.get());
            let decrease = -10;
            accumulator.inc(decrease);
            assert_eq!(accumulator.value, value + decrease);
            assert_eq!(accumulator.value, accumulator.get());
        }
    }

    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        use super::*;
        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        #[ink_e2e::test]
        async fn get(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            let init_value = 10;
            let constructor = AccumulatorRef::new(init_value);
            let contract_account_id = client
                .instantiate("accumulator", &ink_e2e::alice(), constructor, 0, None)
                .await
                .expect("instantiation failed")
                .account_id;

            // Build `get` message and execute
            let get_message = ink_e2e::build_message::<AccumulatorRef>(contract_account_id.clone())
                .call(|accumulator| accumulator.get());
            let get_result = client
                .call_dry_run(&ink_e2e::alice(), &get_message, 0, None)
                .await;
            assert_eq!(get_result.return_value(), init_value);
            Ok(())
        }

        #[ink_e2e::test]
        async fn increase(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            let init_value = 10;
            let constructor = AccumulatorRef::new(init_value);
            let contract_account_id = client
                .instantiate("accumulator", &ink_e2e::alice(), constructor, 0, None)
                .await
                .expect("instantiation failed")
                .account_id;

            // Build `get` message and execute
            let get_message = ink_e2e::build_message::<AccumulatorRef>(contract_account_id.clone())
                .call(|accumulator| accumulator.get());
            let get_result = client
                .call_dry_run(&ink_e2e::alice(), &get_message, 0, None)
                .await;
            assert_eq!(get_result.return_value(), init_value);

            // Build `increase` message and execute
            let increase = 10;
            let increase_message =
                ink_e2e::build_message::<AccumulatorRef>(contract_account_id.clone())
                    .call(|accumulator| accumulator.inc(increase));
            let increase_result = client
                .call(&ink_e2e::alice(), increase_message, 0, None)
                .await;
            assert!(increase_result.is_ok());

            // Build `get` message and execute
            let get_message = ink_e2e::build_message::<AccumulatorRef>(contract_account_id.clone())
                .call(|accumulator| accumulator.get());
            let get_result = client
                .call_dry_run(&ink_e2e::alice(), &get_message, 0, None)
                .await;
            assert_eq!(get_result.return_value(), init_value + increase);
            Ok(())
        }

        #[ink_e2e::test]
        async fn decrease(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            let init_value = 10;
            let constructor = AccumulatorRef::new(init_value);
            let contract_account_id = client
                .instantiate("accumulator", &ink_e2e::alice(), constructor, 0, None)
                .await
                .expect("instantiation failed")
                .account_id;

            // Build `get` message and execute
            let get_message = ink_e2e::build_message::<AccumulatorRef>(contract_account_id.clone())
                .call(|accumulator| accumulator.get());
            let get_result = client
                .call_dry_run(&ink_e2e::alice(), &get_message, 0, None)
                .await;
            assert_eq!(get_result.return_value(), init_value);

            // Build `increase` message and execute
            let decrease = -10;
            let increase_message =
                ink_e2e::build_message::<AccumulatorRef>(contract_account_id.clone())
                    .call(|accumulator| accumulator.inc(decrease));
            let increase_result = client
                .call(&ink_e2e::alice(), increase_message, 0, None)
                .await;
            assert!(increase_result.is_ok());

            // Build `get` message and execute
            let get_message = ink_e2e::build_message::<AccumulatorRef>(contract_account_id.clone())
                .call(|accumulator| accumulator.get());
            let get_result = client
                .call_dry_run(&ink_e2e::alice(), &get_message, 0, None)
                .await;
            assert_eq!(get_result.return_value(), init_value + decrease);
            Ok(())
        }
    }
}
