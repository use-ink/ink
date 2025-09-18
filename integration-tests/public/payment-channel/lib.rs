//! # Payment Channel
//!
//! This implements a payment channel between two parties.
//!
//! ## Warning
//!
//! This contract is an *example*. It is neither audited nor endorsed for production use.
//! Do **not** rely on it to keep anything of value secure.
//!
//! ## Overview
//!
//! Each instantiation of this contract creates a payment channel between a `sender` and a
//! `recipient`. It uses ECDSA signatures to ensure that the `recipient` can only claim
//! the funds if it is signed by the `sender`.
//!
//! ## Error Handling
//!
//! The only panic in the contract is when the signature is invalid. For all other
//! error cases an error is returned. Possible errors are defined in the `Error` enum.
//!
//! ## Interface
//!
//! The interface is modelled after [this blog post](https://programtheblockchain.com/posts/2018/03/02/building-long-lived-payment-channels)
//!
//! ### Deposits
//!
//! The creator of the contract, i.e. the `sender`, can deposit funds to the payment
//! channel while creating the payment channel. Any subsequent deposits can be made by
//! transferring funds to the contract's address.
//!
//! ### Withdrawals
//!
//! The `recipient` can `withdraw` from the payment channel anytime by submitting the last
//! `signature` received from the `sender`.
//!
//! The `sender` can only `withdraw` by terminating the payment channel. This is
//! done by calling `start_sender_close` to set an expiration with a subsequent call
//! of `claim_timeout` to claim the funds. This will terminate the payment channel.

#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod payment_channel {
    use ink::U256;

    /// Struct for storing the payment channel details.
    /// The creator of the contract, i.e. the `sender`, can deposit funds to the payment
    /// channel while deploying the contract.
    #[ink(storage)]
    pub struct PaymentChannel {
        /// The `Address` of the sender of the payment channel.
        sender: Address,
        /// The `Address` of the recipient of the payment channel.
        recipient: Address,
        /// The `Timestamp` at which the contract expires. The field is optional.
        /// The contract never expires if set to `None`.
        expiration: Option<Timestamp>,
        /// The `Amount` withdrawn by the recipient.
        withdrawn: U256,
        /// The `Timestamp` which will be added to the current time when the sender
        /// wishes to close the channel. This will be set at the time of contract
        /// instantiation.
        close_duration: Timestamp,
    }

    /// Errors that can occur upon calling this contract.
    #[derive(Debug, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    pub enum Error {
        /// Returned if caller is not the `sender` while required to.
        CallerIsNotSender,
        /// Returned if caller is not the `recipient` while required to.
        CallerIsNotRecipient,
        /// Returned if the requested withdrawal amount is less than the amount
        /// that is already withdrawn.
        AmountIsLessThanWithdrawn,
        /// Returned if the requested transfer failed. This can be the case if the
        /// contract does not have sufficient free funds or if the transfer would
        /// have brought the contract's balance below minimum balance.
        TransferFailed,
        /// Returned if the contract hasn't expired yet and the `sender` wishes to
        /// close the channel.
        NotYetExpired,
        /// Returned if the signature is invalid.
        InvalidSignature,
    }

    /// Type alias for the contract's `Result` type.
    pub type Result<T> = core::result::Result<T, Error>;

    /// Emitted when the sender starts closing the channel.
    #[ink(event)]
    pub struct SenderCloseStarted {
        expiration: Timestamp,
        close_duration: Timestamp,
    }

    impl PaymentChannel {
        /// The only constructor of the contract.
        ///
        /// The arguments `recipient` and `close_duration` are required.
        ///
        /// `expiration` will be set to `None`, so that the contract will
        /// never expire. `sender` can call `start_sender_close` to override
        /// this. `sender` will be able to claim the remaining balance by calling
        /// `claim_timeout` after `expiration` has passed.
        #[ink(constructor)]
        pub fn new(recipient: Address, close_duration: Timestamp) -> Self {
            Self {
                sender: Self::env().caller(),
                recipient,
                expiration: None,
                withdrawn: 0.into(),
                close_duration,
            }
        }

        /// The `recipient` can close the payment channel anytime. The specified
        /// `amount` will be sent to the `recipient` and the remainder will go
        /// back to the `sender`.
        #[ink(message)]
        pub fn close(&mut self, amount: U256, signature: [u8; 65]) -> Result<()> {
            self.close_inner(amount, signature)?;
            self.env().terminate_contract(self.sender);
        }

        /// We split this out in order to make testing `close` simpler.
        fn close_inner(&mut self, amount: U256, signature: [u8; 65]) -> Result<()> {
            if self.env().caller() != self.recipient {
                return Err(Error::CallerIsNotRecipient)
            }

            if amount < self.withdrawn {
                return Err(Error::AmountIsLessThanWithdrawn)
            }

            // Signature validation
            if !self.is_signature_valid(amount, signature) {
                return Err(Error::InvalidSignature)
            }

            // We checked that amount >= self.withdrawn
            #[allow(clippy::arithmetic_side_effects)]
            self.env()
                .transfer(self.recipient, amount - self.withdrawn)
                .map_err(|_| Error::TransferFailed)?;

            Ok(())
        }

        /// If the `sender` wishes to close the channel and withdraw the funds they can
        /// do so by setting the `expiration`. If the `expiration` is reached, the
        /// sender will be able to call `claim_timeout` to claim the remaining funds
        /// and the channel will be terminated. This emits an event that the recipient can
        /// listen to in order to withdraw the funds before the `expiration`.
        #[ink(message)]
        pub fn start_sender_close(&mut self) -> Result<()> {
            if self.env().caller() != self.sender {
                return Err(Error::CallerIsNotSender)
            }

            let now = self.env().block_timestamp();
            let expiration = now.checked_add(self.close_duration).unwrap();

            self.env().emit_event(SenderCloseStarted {
                expiration,
                close_duration: self.close_duration,
            });

            self.expiration = Some(expiration);

            Ok(())
        }

        /// If the timeout is reached (`current_time >= expiration`) without the
        /// recipient closing the channel, then the remaining balance is released
        /// back to the `sender`.
        #[ink(message)]
        pub fn claim_timeout(&mut self) -> Result<()> {
            match self.expiration {
                Some(expiration) => {
                    // expiration is set. Check if it's reached and if so, release the
                    // funds and terminate the contract.
                    let now = self.env().block_timestamp();
                    if now < expiration {
                        return Err(Error::NotYetExpired)
                    }

                    self.env().terminate_contract(self.sender);
                }

                None => Err(Error::NotYetExpired),
            }
        }

        /// The `recipient` can withdraw the funds from the channel at any time.
        #[ink(message)]
        pub fn withdraw(&mut self, amount: U256, signature: [u8; 65]) -> Result<()> {
            if self.env().caller() != self.recipient {
                return Err(Error::CallerIsNotRecipient)
            }

            // Signature validation
            if !self.is_signature_valid(amount, signature) {
                return Err(Error::InvalidSignature)
            }

            // Make sure there's something to withdraw (guards against underflow)
            if amount < self.withdrawn {
                return Err(Error::AmountIsLessThanWithdrawn)
            }

            // We checked that amount >= self.withdrawn
            #[allow(clippy::arithmetic_side_effects)]
            let amount_to_withdraw = amount - self.withdrawn;
            self.withdrawn.checked_add(amount_to_withdraw).unwrap();

            self.env()
                .transfer(self.recipient, amount_to_withdraw)
                .map_err(|_| Error::TransferFailed)?;

            Ok(())
        }

        /// Returns the `sender` of the contract.
        #[ink(message)]
        pub fn get_sender(&self) -> Address {
            self.sender
        }

        /// Returns the `recipient` of the contract.
        #[ink(message)]
        pub fn get_recipient(&self) -> Address {
            self.recipient
        }

        /// Returns the `expiration` of the contract.
        #[ink(message)]
        pub fn get_expiration(&self) -> Option<Timestamp> {
            self.expiration
        }

        /// Returns the `withdrawn` amount of the contract.
        #[ink(message)]
        pub fn get_withdrawn(&self) -> U256 {
            self.withdrawn
        }

        /// Returns the `close_duration` of the contract.
        #[ink(message)]
        pub fn get_close_duration(&self) -> Timestamp {
            self.close_duration
        }

        /// Returns the `balance` of the contract.
        #[ink(message)]
        pub fn get_balance(&self) -> U256 {
            self.env().balance()
        }
    }

    #[ink(impl)]
    impl PaymentChannel {
        fn is_signature_valid(&self, amount: U256, signature: [u8; 65]) -> bool {
            let encodable = (self.env().address(), amount);
            let mut message =
                <ink::env::hash::Sha2x256 as ink::env::hash::HashOutput>::Type::default();
            ink::env::hash_encoded::<ink::env::hash::Sha2x256, _>(
                &encodable,
                &mut message,
            );

            let mut pub_key = [0; 33];
            ink::env::ecdsa_recover(&signature, &message, &mut pub_key)
                .unwrap_or_else(|err| panic!("recover failed: {err:?}"));
            let mut signature_account_id = [0u8; 32];
            <ink::env::hash::Blake2x256 as ink::env::hash::CryptoHash>::hash(
                &pub_key,
                &mut signature_account_id,
            );

            self.recipient
                == ink::primitives::AccountIdMapper::to_address(&signature_account_id)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        use hex_literal;
        use sp_core::{
            Encode,
            Pair,
        };

        fn default_accounts() -> ink::env::test::DefaultAccounts {
            ink::env::test::default_accounts()
        }

        fn set_next_caller(caller: Address) {
            ink::env::test::set_caller(caller);
        }

        fn set_contract_balance(addr: Address, balance: U256) {
            ink::env::test::set_contract_balance(addr, balance);
        }

        fn get_contract_balance(addr: Address) -> U256 {
            ink::env::test::get_contract_balance::<ink::env::DefaultEnvironment>(addr)
                .expect("Cannot get contract balance")
        }

        fn advance_block() {
            ink::env::test::advance_block::<ink::env::DefaultEnvironment>();
        }

        fn get_current_time() -> Timestamp {
            let since_the_epoch = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("Time went backwards");
            since_the_epoch.as_secs()
                + since_the_epoch.subsec_nanos() as u64 / 1_000_000_000
        }

        fn get_dan() -> Address {
            // Use Dan's seed
            // `subkey inspect //Dan --scheme Ecdsa --output-type json | jq .secretSeed`
            let seed = hex_literal::hex!(
                "c31fa562972de437802e0df146b16146349590b444db41f7e3eb9deedeee6f64"
            );
            let pair = sp_core::ecdsa::Pair::from_seed(&seed);
            let pub_key = pair.public();
            let compressed_pub_key: [u8; 33] = pub_key.encode()[..]
                .try_into()
                .expect("slice with incorrect length");
            let mut account_id = [0; 32];
            <ink::env::hash::Blake2x256 as ink::env::hash::CryptoHash>::hash(
                &compressed_pub_key,
                &mut account_id,
            );
            ink::primitives::AccountIdMapper::to_address(&account_id)
        }

        fn contract_id() -> Address {
            let accounts = default_accounts();
            let contract_id = accounts.charlie;
            ink::env::test::set_callee(contract_id);
            contract_id
        }

        fn sign(contract_id: Address, amount: U256) -> [u8; 65] {
            let encodable = (contract_id, amount);
            let mut hash =
                <ink::env::hash::Sha2x256 as ink::env::hash::HashOutput>::Type::default(); // 256-bit buffer
            ink::env::hash_encoded::<ink::env::hash::Sha2x256, _>(&encodable, &mut hash);

            // Use Dan's seed
            // `subkey inspect //Dan --scheme Ecdsa --output-type json | jq .secretSeed`
            let seed = hex_literal::hex!(
                "c31fa562972de437802e0df146b16146349590b444db41f7e3eb9deedeee6f64"
            );
            let pair = sp_core::ecdsa::Pair::from_seed(&seed);

            let signature = pair.sign_prehashed(&hash);
            signature.0
        }

        #[ink::test]
        fn test_deposit() {
            // given
            let accounts = default_accounts();
            let initial_balance = 10_000.into();
            let close_duration = 360_000;
            let mock_deposit_value = 1_000.into();
            set_contract_balance(accounts.alice, initial_balance);
            set_contract_balance(accounts.bob, initial_balance);

            // when
            // Push the new execution context with Alice as the caller and
            // the `mock_deposit_value` as the value deposited.
            // Note: Currently there is no way to transfer funds to the contract.
            set_next_caller(accounts.alice);
            let payment_channel = PaymentChannel::new(accounts.bob, close_duration);
            let contract_id = contract_id();
            set_contract_balance(contract_id, mock_deposit_value);

            // then
            assert_eq!(payment_channel.get_balance(), mock_deposit_value);
        }

        #[ink::test]
        fn test_close() {
            // given
            let accounts = default_accounts();
            let dan = get_dan();
            let close_duration = 360_000;
            let mock_deposit_value = 1_000.into();
            let amount = 500.into();
            let initial_balance = 10_000.into();
            set_contract_balance(accounts.alice, initial_balance);
            set_contract_balance(dan, initial_balance);

            // when
            set_next_caller(accounts.alice);
            let mut payment_channel = PaymentChannel::new(dan, close_duration);
            let contract_id = contract_id();
            set_contract_balance(contract_id, mock_deposit_value);
            set_next_caller(dan);
            let signature = sign(contract_id, amount);

            // then
            let should_close = move || payment_channel.close(amount, signature).unwrap();
            ink::env::test::assert_contract_termination::<ink::env::DefaultEnvironment, _>(
                should_close,
                accounts.alice,
                amount,
            );
            assert_eq!(get_contract_balance(dan), initial_balance + amount);
        }

        #[ink::test]
        fn close_fails_invalid_signature() {
            // given
            let accounts = default_accounts();
            let dan = get_dan();
            let mock_deposit_value = 1_000.into();
            let close_duration = 360_000;
            let amount = 400.into();
            let unexpected_amount = amount + U256::from(1);
            let initial_balance = 10_000.into();
            set_contract_balance(accounts.alice, initial_balance);
            set_contract_balance(dan, initial_balance);

            // when
            set_next_caller(accounts.alice);
            let mut payment_channel = PaymentChannel::new(dan, close_duration);
            let contract_id = contract_id();
            set_contract_balance(contract_id, mock_deposit_value);
            set_next_caller(dan);
            let signature = sign(contract_id, amount);

            // then
            let res = payment_channel.close_inner(unexpected_amount, signature);
            assert!(res.is_err(), "Expected an error, got {res:?} instead.");
            assert_eq!(res.unwrap_err(), Error::InvalidSignature,);
        }

        #[ink::test]
        fn test_withdraw() {
            // given
            let accounts = default_accounts();
            let dan = get_dan();
            let initial_balance = 10_000.into();
            let mock_deposit_value = 1_000.into();
            let close_duration = 360_000;
            let amount = 500.into();
            set_contract_balance(accounts.alice, initial_balance);
            set_contract_balance(dan, initial_balance);

            // when
            set_next_caller(accounts.alice);
            let mut payment_channel = PaymentChannel::new(dan, close_duration);
            let contract_id = contract_id();
            set_contract_balance(contract_id, mock_deposit_value);

            set_next_caller(dan);
            let signature = sign(contract_id, amount);
            payment_channel
                .withdraw(amount, signature)
                .expect("withdraw failed");

            // then
            assert_eq!(payment_channel.get_balance(), amount);
            assert_eq!(get_contract_balance(dan), initial_balance + amount);
        }

        #[ink::test]
        fn withdraw_fails_invalid_signature() {
            // given
            let accounts = default_accounts();
            let dan = get_dan();
            let initial_balance = 10_000.into();
            let close_duration = 360_000;
            let amount = 400.into();
            let unexpected_amount = amount + U256::from(1);
            let mock_deposit_value = 1_000.into();
            set_contract_balance(accounts.alice, initial_balance);
            set_contract_balance(dan, initial_balance);

            // when
            set_next_caller(accounts.alice);
            let mut payment_channel = PaymentChannel::new(dan, close_duration);
            let contract_id = contract_id();
            set_contract_balance(contract_id, mock_deposit_value);
            set_next_caller(dan);
            let signature = sign(contract_id, amount);

            // then
            let res = payment_channel.withdraw(unexpected_amount, signature);
            assert!(res.is_err(), "Expected an error, got {res:?} instead.");
            assert_eq!(res.unwrap_err(), Error::InvalidSignature,);
        }

        #[ink::test]
        fn test_start_sender_close() {
            // given
            let accounts = default_accounts();
            let initial_balance = 10_000.into();
            let mock_deposit_value = 1_000.into();
            let close_duration = 1;
            set_contract_balance(accounts.alice, initial_balance);
            set_contract_balance(accounts.bob, initial_balance);

            // when
            set_next_caller(accounts.alice);
            let mut payment_channel = PaymentChannel::new(accounts.bob, close_duration);
            let contract_id = contract_id();
            set_contract_balance(contract_id, mock_deposit_value);

            payment_channel
                .start_sender_close()
                .expect("start_sender_close failed");
            advance_block();

            // then
            let now = get_current_time();
            assert!(now > payment_channel.get_expiration().unwrap());
        }

        #[ink::test]
        fn test_claim_timeout() {
            // given
            let accounts = default_accounts();
            let initial_balance = 10_000.into();
            let close_duration = 1;
            let mock_deposit_value = 1_000.into();
            set_contract_balance(accounts.alice, initial_balance);
            set_contract_balance(accounts.bob, initial_balance);

            // when
            set_next_caller(accounts.alice);
            let contract_id = contract_id();
            let mut payment_channel = PaymentChannel::new(accounts.bob, close_duration);
            set_contract_balance(contract_id, mock_deposit_value);

            payment_channel
                .start_sender_close()
                .expect("start_sender_close failed");
            advance_block();

            // then
            let should_close = move || payment_channel.claim_timeout().unwrap();
            ink::env::test::assert_contract_termination::<ink::env::DefaultEnvironment, _>(
                should_close,
                accounts.alice,
                mock_deposit_value,
            );
            assert_eq!(
                get_contract_balance(accounts.alice),
                initial_balance + mock_deposit_value
            );
        }

        #[ink::test]
        fn test_getters() {
            // given
            let accounts = default_accounts();
            let initial_balance = 10_000.into();
            let mock_deposit_value = 1_000.into();
            let close_duration = 360_000;
            set_contract_balance(accounts.alice, initial_balance);
            set_contract_balance(accounts.bob, initial_balance);

            // when
            set_next_caller(accounts.alice);
            let contract_id = contract_id();
            let payment_channel = PaymentChannel::new(accounts.bob, close_duration);
            set_contract_balance(contract_id, mock_deposit_value);

            // then
            assert_eq!(payment_channel.get_sender(), accounts.alice);
            assert_eq!(payment_channel.get_recipient(), accounts.bob);
            assert_eq!(payment_channel.get_balance(), mock_deposit_value);
            assert_eq!(payment_channel.get_close_duration(), close_duration);
            assert_eq!(payment_channel.get_withdrawn(), U256::zero());
        }
    }
}
