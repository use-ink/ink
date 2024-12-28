#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod contract_xcm {
    use ink::{
        env::Error as EnvError,
        xcm::prelude::*,
    };

    /// A smart contract example using the XCM API for cross-chain communication.
    #[ink(storage)]
    #[derive(Default)]
    pub struct ContractXcm;

    /// Enumeration of runtime errors for the contract.
    #[derive(Debug, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    pub enum RuntimeError {
        XcmExecuteFailed,
        XcmSendFailed,
        UnexpectedEnvError,
    }

    impl From<EnvError> for RuntimeError {
        fn from(e: EnvError) -> Self {
            match e {
                EnvError::ReturnError(code) => match code {
                    ink::env::ReturnErrorCode::XcmExecutionFailed => RuntimeError::XcmExecuteFailed,
                    ink::env::ReturnErrorCode::XcmSendFailed => RuntimeError::XcmSendFailed,
                    _ => RuntimeError::UnexpectedEnvError,
                },
                _ => RuntimeError::UnexpectedEnvError,
            }
        }
    }

    impl ContractXcm {
        /// The constructor is `payable`, allowing the contract to receive initial tokens.
        #[ink(constructor, payable)]
        pub fn new() -> Self {
            Default::default()
        }

        /// Helper function to build an XCM message.
        ///
        /// # Arguments
        /// * `receiver` - The target account to receive the assets.
        /// * `value` - The amount of tokens to transfer.
        /// * `fee` - Optional fee for the XCM execution.
        fn build_xcm_message(
            &self,
            receiver: AccountId32,
            value: Balance,
            fee: Option<Balance>,
        ) -> Xcm<()> {
            let asset: Asset = (Here, value).into();
            let mut builder = Xcm::builder()
                .withdraw_asset(asset.clone())
                .deposit_asset(asset.clone(), receiver);
            if let Some(fee) = fee {
                builder = builder.buy_execution((Here, fee), WeightLimit::Unlimited);
            }
            builder.build()
        }

        /// Transfers funds through XCM to the given receiver.
        ///
        /// Fails if:
        /// - XCM execution fails.
        /// - Insufficient funds.
        /// - Unsupported environment or runtime configuration.
        #[ink(message)]
        pub fn transfer_through_xcm(
            &mut self,
            receiver: AccountId,
            value: Balance,
        ) -> Result<(), RuntimeError> {
            let beneficiary = AccountId32 {
                network: None,
                id: *receiver.as_ref(),
            };
            let message = self.build_xcm_message(beneficiary, value, None);

            self.env()
                .xcm_execute(&VersionedXcm::V4(message))
                .map_err(Into::into)
        }

        /// Sends funds through XCM, paying a fee for execution.
        ///
        /// Fails if:
        /// - XCM execution fails.
        /// - Insufficient funds or fees.
        /// - Unsupported environment or runtime configuration.
        #[ink(message)]
        pub fn send_funds(
            &mut self,
            value: Balance,
            fee: Balance,
        ) -> Result<XcmHash, RuntimeError> {
            let beneficiary = AccountId32 {
                network: None,
                id: *self.env().caller().as_ref(),
            };
            let destination: Location = Parent.into();
            let message = self.build_xcm_message(beneficiary, value, Some(fee));

            let hash = self
                .env()
                .xcm_send(&VersionedLocation::V4(destination), &VersionedXcm::V4(message))?;
            Ok(hash)
        }
    }

    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        use super::*;
        use ink::{
            env::{test::default_accounts, DefaultEnvironment},
            primitives::AccountId,
        };
        use ink_e2e::{
            preset::mock_network::{primitives::CENTS, MockNetworkSandbox},
            ChainBackend,
        };

        /// Initial contract balance for testing.
        pub const CONTRACT_BALANCE: u128 = 1_000_000;

        type E2EResult<T> = Result<T, Box<dyn std::error::Error>>;

        #[ink_e2e::test(backend(runtime_only(sandbox = MockNetworkSandbox)))]
        async fn transfer_through_xcm_works<Client: E2EBackend>(
            mut client: Client,
        ) -> E2EResult<()> {
            // Arrange: Instantiate the contract with a predefined balance.
            let mut constructor = ContractXcmRef::new();
            let contract = client
                .instantiate("contract_xcm", &ink_e2e::alice(), &mut constructor)
                .value(CONTRACT_BALANCE)
                .submit()
                .await
                .expect("instantiation failed");

            let receiver: AccountId = default_accounts::<DefaultEnvironment>().bob;

            // Act: Execute the transfer through XCM.
            let transfer_message = contract
                .call_builder()
                .transfer_through_xcm(receiver, 1_000 * CENTS);

            let result = client.call(&ink_e2e::alice(), &transfer_message).submit().await?;
            assert!(result.return_value().is_ok());
            Ok(())
        }
    }
}
