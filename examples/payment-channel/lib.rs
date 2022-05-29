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
//! Each instantiation of this contract creates a payment channel between a `sender` and a `recipient`.
//! It uses ECDSA signatures to ensure that the `recipient` can only claim the funds if it is signed by the `sender`.
//!
//! ## Error Handling
//!
//! The only panic in the contract is when the signature is invalid. For rest, it'll return an error.
//! The possible errors are defined in the `Error` enum.
//!
//! ## Interface
//!
//! The interface is modelled after the [this blog post](https://programtheblockchain.com/posts/2018/03/02/building-long-lived-payment-channels)
//!
//! ### Deposits
//!
//! The creator of the contract, i.e the `sender`, can deposit funds to the payment channel while creating the payment channel.
//! Any subsequent deposits can be made by transferring funds to the contract's address.
//!
//! ### Withdrwals
//!
//! The `recipient` can `withdraw` from the payment channel anytime by submitting the last `signature` received from the `sender`.
//!
//! The `sender` can only `withdraw` by terminating the payment channel. He can call `start_sender_close` to set an expiration.
//! Then he can call `claim_timeout` to claim the funds. This will terminate the payment channel.

#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod payment_channel {

    #[ink(storage)]
    pub struct PaymentChannel {
        /// The `AccountId` of the sender of the payment channel.
        sender: AccountId,
        /// The `AccountId` of the recipient of the payment channel.
        recipient: AccountId,
        /// The `Timestamp` at which the contract expires.
        expiration: Timestamp,
        /// The `Amount` withdrawn by the recipient.
        withdrawn: Balance,
        /// The `Timestamp` which will be added to the current time when the sender wishes to close the channel.
        /// This will be set at the time of contract instantiation.
        close_duration: Timestamp,
    }

    /// Errors that can occur upon calling this contract.
    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(::scale_info::TypeInfo))]
    pub enum Error {
        /// Returned if caller is not the `sender` while required to.
        CallerIsNotSender,
        /// Returned if caller is not the `recipient` while required to.
        CallerIsNotRecipient,
        /// Returned if the requested withdrawal / close amount is less than the amount that is already already withdrawn.
        AmountIsLessThanWithdrawn,
        /// Returned if the requested transfer failed. this can be the case if the contract does not have sufficient free
        /// funds or if the transfer would have brought the contract's balance below minimum balance.
        TransferFailed,
        /// Returned if the contract hasn't expired yet and the `sender` wishes to close the channel.
        NotYetExpired,
    }

    /// Type alias for the contract's result type.
    pub type Result<T> = core::result::Result<T, Error>;

    /// Emitted when the sender wishes to close the channel.
    #[ink(event)]
    pub struct SenderCloseStarted {
        expiration: Timestamp,
        close_duration: Timestamp,
    }

    impl PaymentChannel {
        /// The only constructor of the contract.
        ///
        /// The recipient and the close_duration are required.
        ///
        /// `expiration` will be set to a max value so that the contract will never expire.
        /// `sender` can call `start_sender_close` to override this.
        /// `sender` will be able to claim the remaining balance by calling `claim_timeout` after `expiration` has passed.
        #[ink(constructor)]
        pub fn new(recipient: AccountId, close_duration: Timestamp) -> Self {
            Self {
                sender: Self::env().caller(),
                recipient,
                expiration: u64::pow(2, 63) - 1,
                withdrawn: 0,
                close_duration,
            }
        }

        /// `recipient` can close the payment channel anytime. The `recipient` will be sent that amount,
        /// and the remainder will go back to the `sender`.
        #[ink(message)]
        pub fn close(&mut self, amount: Balance, signature: [u8; 65]) {
            assert!(
                self.env().caller() == self.recipient,
                "{:?}: Expected caller {:?}, got {:?} instead",
                Error::CallerIsNotRecipient,
                self.recipient,
                self.env().caller()
            );

            assert!(
                amount > self.withdrawn,
                "{:?}: Expected amount ({:?}) to be greater than withdrawn amount ({:?})",
                Error::AmountIsLessThanWithdrawn,
                amount,
                self.withdrawn
            );

            // Signature validation
            self.ensure_valid_signature(amount, signature);

            if self
                .env()
                .transfer(self.recipient, amount - self.withdrawn)
                .is_err()
            {
                panic!("{:?}: This can be the case if the contract does not have sufficient free funds or if the
                transfer would have brought the contract's balance below minimum balance.", Error::TransferFailed)
            }
            self.env().terminate_contract(self.sender);
        }

        /// If the `sender` wishes to close the channel and withdraw the funds, they can do so by
        /// setting the `expiration`. If the `expiration` is reached, the sender will be able to call `claim_timeout` to claim the remaining funds
        /// and the channel will be terminated. This emits an event that the recipient can listen to in order to withdraw the funds before the `expiration`
        #[ink(message)]
        pub fn start_sender_close(&mut self) -> Result<()> {
            if self.env().caller() != self.sender {
                return Err(Error::CallerIsNotSender)
            }

            let now = self.env().block_timestamp();
            let expiration = now + self.close_duration;

            self.env().emit_event(SenderCloseStarted {
                expiration,
                close_duration: self.close_duration,
            });

            self.expiration = expiration;

            Ok(())
        }

        /// If the timeout is reached ( current_time > `expiration` ) without the recipient closing the channel, then
        /// the remaining balance is released back to the `sender`.
        #[ink(message)]
        pub fn claim_timeout(&mut self) {
            let now = self.env().block_timestamp();
            assert!(
                now > self.expiration,
                "{:?}: Expected current time ({:?}) to be greater than expiration ({:?})",
                Error::NotYetExpired,
                now,
                self.expiration
            );

            self.env().terminate_contract(self.sender);
        }

        /// `recipient` can withdraw the funds from the channel at any time.
        #[ink(message)]
        pub fn withdraw(&mut self, amount: Balance, signature: [u8; 65]) -> Result<()> {
            if self.env().caller() != self.recipient {
                return Err(Error::CallerIsNotRecipient)
            }

            // Signature validation
            self.ensure_valid_signature(amount, signature);

            // Make sure there's something to withdraw (guards against underflow)
            if amount < self.withdrawn {
                return Err(Error::AmountIsLessThanWithdrawn)
            }

            let amount_to_withdraw = amount - self.withdrawn;
            self.withdrawn += amount_to_withdraw;

            if self
                .env()
                .transfer(self.recipient, amount_to_withdraw)
                .is_err()
            {
                return Err(Error::TransferFailed)
            }

            Ok(())
        }

        /// returns the `sender` of the contract
        #[ink(message)]
        pub fn get_sender(&self) -> AccountId {
            self.sender
        }

        /// returns the `recipient` of the contract
        #[ink(message)]
        pub fn get_recipient(&self) -> AccountId {
            self.recipient
        }

        /// returns the `expiration` of the contract
        #[ink(message)]
        pub fn get_expiration(&self) -> Timestamp {
            self.expiration
        }

        /// returns the `withdrawn` amount of the contract
        #[ink(message)]
        pub fn get_withdrawn(&self) -> Balance {
            self.withdrawn
        }

        /// returns the `close_duration` of the contract
        #[ink(message)]
        pub fn get_close_duration(&self) -> Timestamp {
            self.close_duration
        }

        /// returns the `balance` of the contract
        #[ink(message)]
        pub fn get_balance(&self) -> Balance {
            self.env().balance()
        }
    }

    #[ink(impl)]
    impl PaymentChannel {
        fn ensure_valid_signature(&self, amount: Balance, signature: [u8; 65]) {
            let encodable = (self.env().account_id(), amount);
            let mut message =
                <ink_env::hash::Sha2x256 as ink_env::hash::HashOutput>::Type::default();
            ink_env::hash_encoded::<ink_env::hash::Sha2x256, _>(&encodable, &mut message);

            let mut pub_key = [0; 33];
            ink_env::ecdsa_recover(&signature, &message, &mut pub_key)
                .expect("recover failed");
            let mut signature_account_id = [0; 32];
            <ink_env::hash::Blake2x256 as ink_env::hash::CryptoHash>::hash(
                &pub_key,
                &mut signature_account_id,
            );

            assert!(
                self.recipient == signature_account_id.into(),
                "invalid signature"
            );
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        use hex_literal;
        use ink_lang as ink;
        use sp_core::{
            Encode,
            Pair,
        };

        fn default_accounts(
        ) -> ink_env::test::DefaultAccounts<ink_env::DefaultEnvironment> {
            ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
        }

        fn set_next_caller(caller: AccountId) {
            ink_env::test::set_caller::<ink_env::DefaultEnvironment>(caller);
        }

        fn set_account_balance(account: AccountId, balance: Balance) {
            ink_env::test::set_account_balance::<ink_env::DefaultEnvironment>(
                account, balance,
            );
        }

        fn get_account_balance(account: AccountId) -> Balance {
            ink_env::test::get_account_balance::<ink_env::DefaultEnvironment>(account)
                .expect("Cannot get account balance")
        }

        fn advance_block() {
            ink_env::test::advance_block::<ink_env::DefaultEnvironment>();
        }

        fn get_current_time() -> Timestamp {
            let since_the_epoch = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("Time went backwards");
            since_the_epoch.as_secs() * 1000
                + since_the_epoch.subsec_nanos() as u64 / 1_000_000
        }

        fn get_dan() -> AccountId {
            // Use Dan's seed
            // subkey inspect //Dan --scheme Ecdsa
            let seed = hex_literal::hex!(
                "c31fa562972de437802e0df146b16146349590b444db41f7e3eb9deedeee6f64"
            );
            let pair = sp_core::ecdsa::Pair::from_seed(&seed);
            let pub_key = pair.public();
            let compressed_pub_key: [u8; 33] = pub_key.encode()[..]
                .try_into()
                .expect("slice with incorrect length");
            let mut account_id = [0; 32];
            <ink_env::hash::Blake2x256 as ink_env::hash::CryptoHash>::hash(
                &compressed_pub_key,
                &mut account_id,
            );
            account_id.into()
        }

        fn contract_id() -> AccountId {
            let accounts = default_accounts();
            let contract_id = accounts.charlie;
            ink_env::test::set_callee::<ink_env::DefaultEnvironment>(contract_id);
            contract_id
        }

        fn sign(contract_id: AccountId, amount: Balance) -> [u8; 65] {
            let encodable = (contract_id, amount);
            let mut hash =
                <ink_env::hash::Sha2x256 as ink_env::hash::HashOutput>::Type::default(); // 256-bit buffer
            ink_env::hash_encoded::<ink_env::hash::Sha2x256, _>(&encodable, &mut hash);

            // Use Dan's seed
            // subkey inspect //Dan --scheme Ecdsa
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
            set_account_balance(accounts.alice, 10000);
            set_account_balance(accounts.bob, 10000);
            let mock_deposit_value = 1000;

            // when
            // Push the new execution context with Alice as the caller and
            // the mock_deposit_value as the value deposited.
            // Note: Currently there is no way to transfer funds to the contract
            set_next_caller(accounts.alice);
            let payment_channel = PaymentChannel::new(accounts.bob, 360000);
            let contract_id = contract_id();
            set_account_balance(contract_id, mock_deposit_value);

            // then
            assert_eq!(payment_channel.get_balance(), 1000);
        }

        #[ink::test]
        fn test_close() {
            // given
            let accounts = default_accounts();
            let dan = get_dan();
            set_account_balance(accounts.alice, 10000);
            set_account_balance(dan, 10000);
            let mock_deposit_value = 1000;

            // when
            set_next_caller(accounts.alice);
            let mut payment_channel = PaymentChannel::new(dan, 360000);
            let contract_id = contract_id();
            set_account_balance(contract_id, mock_deposit_value);
            set_next_caller(dan);
            let signature = sign(contract_id, 500);

            // then
            let should_close = move || payment_channel.close(500, signature);
            ink_env::test::assert_contract_termination::<ink_env::DefaultEnvironment, _>(
                should_close,
                accounts.alice,
                500,
            );
            assert_eq!(get_account_balance(dan), 10500);
        }

        #[ink::test]
        #[should_panic]
        fn close_fails_invalid_signature() {
            // given
            let accounts = default_accounts();
            let dan = get_dan();
            set_account_balance(accounts.alice, 10000);
            set_account_balance(dan, 10000);
            let mock_deposit_value = 1000;

            // when
            set_next_caller(accounts.alice);
            let mut payment_channel = PaymentChannel::new(dan, 360000);
            let contract_id = contract_id();
            set_account_balance(contract_id, mock_deposit_value);
            set_next_caller(dan);
            let signature = sign(contract_id, 400);

            // then
            let should_close = move || payment_channel.close(500, signature);
            ink_env::test::assert_contract_termination::<ink_env::DefaultEnvironment, _>(
                should_close,
                accounts.alice,
                500,
            );
            // should have panicked
        }

        #[ink::test]
        fn test_withdraw() {
            // given
            let accounts = default_accounts();
            let dan = get_dan();
            set_account_balance(accounts.alice, 10000);
            set_account_balance(dan, 10000);
            let mock_deposit_value = 1000;

            // when
            set_next_caller(accounts.alice);
            let mut payment_channel = PaymentChannel::new(dan, 360000);
            let contract_id = contract_id();
            set_account_balance(contract_id, mock_deposit_value);

            set_next_caller(dan);
            let signature = sign(contract_id, 500);
            payment_channel
                .withdraw(500, signature)
                .expect("withdraw failed");

            // then
            assert_eq!(payment_channel.get_balance(), 500);
            assert_eq!(get_account_balance(dan), 10500);
        }

        #[ink::test]
        #[should_panic(expected = "invalid signature")]
        fn withdraw_fails_invalid_signature() {
            // given
            let accounts = default_accounts();
            let dan = get_dan();
            set_account_balance(accounts.alice, 10000);
            set_account_balance(dan, 10000);
            let mock_deposit_value = 1000;

            // when
            set_next_caller(accounts.alice);
            let mut payment_channel = PaymentChannel::new(dan, 360000);
            let contract_id = contract_id();
            set_account_balance(contract_id, mock_deposit_value);

            set_next_caller(dan);
            let signature = sign(contract_id, 400);
            payment_channel
                .withdraw(500, signature)
                .expect("withdraw should't have thrown an error");

            // then should have panicked
            // assert_eq!(payment_channel.get_balance(), 600);
            // assert_eq!(get_account_balance(dan), 10400);
        }

        #[ink::test]
        fn test_start_sender_close() {
            // given
            let accounts = default_accounts();
            set_account_balance(accounts.alice, 10000);
            set_account_balance(accounts.bob, 10000);
            let mock_deposit_value = 1000;

            // when
            set_next_caller(accounts.alice);
            let mut payment_channel = PaymentChannel::new(accounts.bob, 1);
            let contract_id = contract_id();
            set_account_balance(contract_id, mock_deposit_value);

            payment_channel
                .start_sender_close()
                .expect("start_sender_close failed");
            advance_block();

            // then
            let now = get_current_time();
            assert!(now > payment_channel.get_expiration());
        }

        #[ink::test]
        fn test_claim_timeout() {
            // given
            let accounts = default_accounts();
            set_account_balance(accounts.alice, 10000);
            set_account_balance(accounts.bob, 10000);
            let mock_deposit_value = 1000;

            // when
            set_next_caller(accounts.alice);
            let contract_id = contract_id();
            let mut payment_channel = PaymentChannel::new(accounts.bob, 1);
            set_account_balance(contract_id, mock_deposit_value);

            payment_channel
                .start_sender_close()
                .expect("start_sender_close failed");
            advance_block();

            // then
            let should_close = move || payment_channel.claim_timeout();
            ink_env::test::assert_contract_termination::<ink_env::DefaultEnvironment, _>(
                should_close,
                accounts.alice,
                1000,
            );
            assert_eq!(get_account_balance(accounts.alice), 11000);
        }

        #[ink::test]
        fn test_getters() {
            // given
            let accounts = default_accounts();
            set_account_balance(accounts.alice, 10000);
            set_account_balance(accounts.bob, 10000);
            let mock_deposit_value = 1000;
            let close_duration = 360000;

            // when
            set_next_caller(accounts.alice);
            let contract_id = contract_id();
            let payment_channel = PaymentChannel::new(accounts.bob, close_duration);
            set_account_balance(contract_id, mock_deposit_value);

            // then
            assert_eq!(payment_channel.get_sender(), accounts.alice);
            assert_eq!(payment_channel.get_recipient(), accounts.bob);
            assert_eq!(payment_channel.get_balance(), mock_deposit_value);
            assert_eq!(payment_channel.get_close_duration(), close_duration);
            assert_eq!(payment_channel.get_withdrawn(), 0);
        }
    }
}
