#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod contract_xcm {
    use ink::xcm::prelude::*;

    /// A contract demonstrating usage of the XCM API.
    #[ink(storage)]
    #[derive(Default)]
    pub struct ContractXcm;

    #[derive(Debug, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    pub enum RuntimeError {
        XcmExecuteFailed,
        XcmSendFailed,
    }

    impl ContractXcm {
        /// The constructor is `payable`, so that during instantiation it can be given
        /// some tokens that will be further transferred when transferring funds through
        /// XCM.
        #[ink(constructor, payable)]
        pub fn new() -> Self {
            Default::default()
        }

        /// Tries to transfer `value` from the contract's balance to `receiver`
        /// on the SAME chain using XCM execution.
        ///
        /// This demonstrates `xcm_execute`, which executes an XCM message locally.
        #[ink(message)]
        pub fn transfer_through_xcm(
            &mut self,
            receiver: AccountId,
            value: Balance,
        ) -> Result<(), RuntimeError> {
            // Define the asset as the native currency (Parent) with amount `value`.
            let asset: Asset = (Parent, value).into();
            
            // Define beneficiary on the current network.
            let beneficiary = AccountId32 {
                network: None,
                id: *receiver.as_ref(),
            };

            // Build the XCM message:
            // 1. Withdraw asset from contract's account.
            // 2. Buy execution time (gas) for the XCM VM.
            // 3. Deposit the asset into the beneficiary's account.
            let message: ink::xcm::v5::Xcm<()> = Xcm::builder()
                .withdraw_asset(asset.clone())
                .buy_execution(asset.clone(), Unlimited)
                .deposit_asset(asset, beneficiary)
                .build();
            let msg = VersionedXcm::V5(message);

            // Calculate the weight (gas) required for this XCM message.
            let weight = self.env().xcm_weigh(&msg).expect("weight should work");

            // Execute the XCM message locally.
            self.env()
                .xcm_execute(&msg, weight)
                .map_err(|_| RuntimeError::XcmExecuteFailed)
        }

        /// Transfer some funds to the relay chain via XCM using `xcm_send`.
        ///
        /// This sends an XCM message to another chain (the Parent/Relay Chain).
        #[ink(message)]
        pub fn send_funds(
            &mut self,
            value: Balance,
            fee: Balance,
        ) -> Result<(), RuntimeError> {
            // Target destination: The Parent chain (Relay Chain).
            let destination: ink::xcm::v5::Location = ink::xcm::v5::Parent.into();

            // Asset: Native token of the relay chain (represented as Here relative to Parent).
            let asset: Asset = (Here, value).into();

            // Beneficiary: The caller's account on the Relay Chain.
            let caller_account_id = self.env().to_account_id(self.env().caller());
            let beneficiary = AccountId32 {
                network: None,
                id: caller_account_id.0,
            };

            // Build XCM:
            // 1. Withdraw asset from this chain's sovereign account on the Relay Chain.
            // 2. Buy execution on the Relay Chain using the withdrawn asset.
            // 3. Deposit asset to the caller's account on the Relay Chain.
            let message: Xcm<()> = Xcm::builder()
                .withdraw_asset(asset.clone())
                .buy_execution((Here, fee), WeightLimit::Unlimited)
                .deposit_asset(asset, beneficiary)
                .build();

            // Send the message to the Relay Chain.
            self.env()
                .xcm_send(
                    &VersionedLocation::V5(destination),
                    &VersionedXcm::V5(message),
                )
                .map_err(|_| RuntimeError::XcmSendFailed)
        }

        /// Initiates a reserve transfer, burning tokens here and releasing them on the Parent chain.
        #[ink(message)]
        pub fn reserve_transfer(
            &mut self,
            amount: Balance,
            fee: Balance,
        ) -> Result<(), RuntimeError> {
            let caller_account_id = self.env().to_account_id(self.env().caller());
            let beneficiary: Location = AccountId32 {
                network: None,
                id: caller_account_id.0,
            }
            .into();

            // Build XCM using `builder_unsafe` for advanced operations like reserve transfers.
            let message: Xcm<()> = Xcm::builder_unsafe()
                // Withdraw the derivative token (Parent) from contract's local account.
                .withdraw_asset((Parent, amount))

                // Burn the local derivative and send an instruction to the Reserve (Parent)
                // to release the real asset to the beneficiary.
                .initiate_reserve_withdraw(
                    All,
                    Parent,
                    Xcm::builder_unsafe()
                        .buy_execution((Here, fee), Unlimited)
                        .deposit_asset(All, beneficiary)
                        .build(),
                )
                .build();

            let msg = VersionedXcm::V5(message);
            let weight = self.env().xcm_weigh(&msg).expect("`xcm_weigh` failed");
            
            self.env()
                .xcm_execute(&msg, weight)
                .map_err(|_| RuntimeError::XcmExecuteFailed)
        }
    }
}

#[cfg(test)]
mod tests;