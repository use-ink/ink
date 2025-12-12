#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod ensure_test {
    use ink::{U256, ensure};

    /// A simple contract to test the ensure! macro.
    #[ink(storage)]
    #[derive(Default)]
    pub struct EnsureTest {
        balance: U256,
    }

    /// Error types for testing ensure!
    #[derive(Debug, PartialEq, Eq)]
    #[ink::error]
    pub enum Error {
        InsufficientBalance,
        ValueTooLarge,
        ValueMustBePositive,
    }

    /// Result type.
    pub type Result<T> = core::result::Result<T, Error>;

    impl EnsureTest {
        /// Creates a new contract with initial balance.
        #[ink(constructor)]
        pub fn new(initial_balance: U256) -> Self {
            Self {
                balance: initial_balance,
            }
        }

        /// Get the current balance.
        #[ink(message)]
        pub fn balance(&self) -> U256 {
            self.balance
        }

        /// Transfer tokens - uses ensure! to check balance.
        #[ink(message)]
        pub fn transfer(&mut self, amount: U256) -> Result<()> {
            // Test ensure! with positive value check
            ensure!(amount > U256::from(0), Error::ValueMustBePositive);

            // Test ensure! with balance check
            ensure!(self.balance >= amount, Error::InsufficientBalance);

            // Test ensure! with maximum value check
            ensure!(amount <= U256::from(1000), Error::ValueTooLarge);

            self.balance -= amount;
            Ok(())
        }

        /// Deposit tokens - uses ensure! with trailing comma.
        #[ink(message)]
        pub fn deposit(&mut self, amount: U256) -> Result<()> {
            ensure!(amount > U256::from(0), Error::ValueMustBePositive,);
            self.balance += amount;
            Ok(())
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[ink::test]
        fn ensure_works_with_positive_value() {
            let mut contract = EnsureTest::new(U256::from(100));
            assert!(contract.transfer(U256::from(50)).is_ok());
        }

        #[ink::test]
        fn ensure_returns_error_for_zero_value() {
            let mut contract = EnsureTest::new(U256::from(100));
            assert_eq!(
                contract.transfer(U256::from(0)),
                Err(Error::ValueMustBePositive)
            );
        }

        #[ink::test]
        fn ensure_returns_error_for_insufficient_balance() {
            let mut contract = EnsureTest::new(U256::from(50));
            assert_eq!(
                contract.transfer(U256::from(100)),
                Err(Error::InsufficientBalance)
            );
        }

        #[ink::test]
        fn ensure_returns_error_for_value_too_large() {
            let mut contract = EnsureTest::new(U256::from(2000));
            assert_eq!(
                contract.transfer(U256::from(1001)),
                Err(Error::ValueTooLarge)
            );
        }

        #[ink::test]
        fn ensure_works_with_trailing_comma() {
            let mut contract = EnsureTest::new(U256::from(100));
            assert!(contract.deposit(U256::from(50)).is_ok());
        }
    }
}
