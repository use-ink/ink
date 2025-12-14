#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod ensure_test {
    use ink::{
        U256,
        ensure,
    };

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
    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        use super::*;
        use ink_e2e::ContractsBackend;
        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;
        #[ink_e2e::test]
        async fn e2e_transfer_succeeds(mut client: Client) -> E2EResult<()> {
            // Deploy the contract
            let mut constructor = EnsureTestRef::new(U256::from(1000));
            let contract = client
                .instantiate("ensure_test", &ink_e2e::alice(), &mut constructor)
                .submit()
                .await
                .expect("instantiate failed");
            let mut call_builder = contract.call_builder::<EnsureTest>();
            let transfer = call_builder.transfer(U256::from(500));
            let result = client
                .call(&ink_e2e::alice(), &transfer)
                .submit()
                .await
                .expect("transfer should succeed");
            assert!(result.return_value().is_ok());

            Ok(())
        }
        #[ink_e2e::test]
        async fn e2e_transfer_fails_with_value_must_be_positive(
            mut client: Client,
        ) -> E2EResult<()> {
            // Deploy contract
            let mut constructor = EnsureTestRef::new(U256::from(1000));
            let contract = client
                .instantiate("ensure_test", &ink_e2e::alice(), &mut constructor)
                .submit()
                .await
                .expect("instantiate failed");
            let mut call_builder = contract.call_builder::<EnsureTest>();

            // Try to transfer zero - should fail
            let transfer = call_builder.transfer(U256::from(0));
            let result = client.call(&ink_e2e::alice(), &transfer).dry_run().await?;

            // Check it reverted and decode the error
            assert!(result.did_revert(), "should revert");
            let error_data = result.return_data();
            let decoded_error =
                <Error as ink::scale::Decode>::decode(&mut &error_data[..])?;
            assert_eq!(decoded_error, Error::ValueMustBePositive);

            Ok(())
        }
        #[ink_e2e::test]
        async fn e2e_transfer_fails_with_insufficient_balance(
            mut client: Client,
        ) -> E2EResult<()> {
            let mut constructor = EnsureTestRef::new(U256::from(100));
            let contract = client
                .instantiate("ensure_test", &ink_e2e::alice(), &mut constructor)
                .submit()
                .await
                .expect("instantiate failed");
            let mut call_builder = contract.call_builder::<EnsureTest>();

            // Try to transfer more than balance (200 when balance is 100)
            let transfer = call_builder.transfer(U256::from(200));
            let result = client.call(&ink_e2e::alice(), &transfer).dry_run().await?;

            assert!(result.did_revert());
            let error_data = result.return_data();
            let decoded_error =
                <Error as ink::scale::Decode>::decode(&mut &error_data[..])?;
            assert_eq!(decoded_error, Error::InsufficientBalance);

            Ok(())
        }
        #[ink_e2e::test]
        async fn e2e_transfer_fails_with_value_too_large(
            mut client: Client,
        ) -> E2EResult<()> {
            let mut constructor = EnsureTestRef::new(U256::from(2000));
            let contract = client
                .instantiate("ensure_test", &ink_e2e::alice(), &mut constructor)
                .submit()
                .await
                .expect("instantiate failed");
            let mut call_builder = contract.call_builder::<EnsureTest>();

            // Try to transfer more than max (1001 when max is 1000)
            let transfer = call_builder.transfer(U256::from(1001));
            let result = client.call(&ink_e2e::alice(), &transfer).dry_run().await?;

            assert!(result.did_revert());
            let error_data = result.return_data();
            let decoded_error =
                <Error as ink::scale::Decode>::decode(&mut &error_data[..])?;
            assert_eq!(decoded_error, Error::ValueTooLarge);

            Ok(())
        }
    }
}
