#![cfg_attr(not(feature = "std"), no_std, no_main)]

use ink::{
    Address,
    U256,
    prelude::vec::Vec,
};

// This is the return value that we expect if a smart contract supports receiving ERC-1155
// tokens.
//
// It is calculated with
// `bytes4(keccak256("onERC1155Received(address,address,uint256,uint256,bytes)"))`, and
// corresponds to 0xf23a6e61.
#[cfg_attr(test, allow(dead_code))]
const ON_ERC_1155_RECEIVED_SELECTOR: [u8; 4] = [0xF2, 0x3A, 0x6E, 0x61];

// This is the return value that we expect if a smart contract supports batch receiving
// ERC-1155 tokens.
//
// It is calculated with
// `bytes4(keccak256("onERC1155BatchReceived(address,address,uint256[],uint256[],bytes)"
// ))`, and corresponds to 0xbc197c81.
const _ON_ERC_1155_BATCH_RECEIVED_SELECTOR: [u8; 4] = [0xBC, 0x19, 0x7C, 0x81];

/// A type representing the unique IDs of tokens managed by this contract.
pub type TokenId = u128;

// The ERC-1155 error types.
#[derive(Debug, PartialEq, Eq)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub enum Error {
    /// This token ID has not yet been created by the contract.
    UnexistentToken,
    /// The caller tried to sending tokens to the zero-address (`0x00`).
    ZeroAddressTransfer,
    /// The caller is not approved to transfer tokens on behalf of the account.
    NotApproved,
    /// The account does not have enough funds to complete the transfer.
    InsufficientU256,
    /// An account does not need to approve themselves to transfer tokens.
    SelfApproval,
    /// The number of tokens being transferred does not match the specified number of
    /// transfers.
    BatchTransferMismatch,
}

// The ERC-1155 result types.
pub type Result<T> = core::result::Result<T, Error>;

/// Evaluate `$x:expr` and if not true return `Err($y:expr)`.
///
/// Used as `ensure!(expression_to_ensure, expression_to_return_on_false)`.
macro_rules! ensure {
    ( $condition:expr, $error:expr $(,)? ) => {{
        if !$condition {
            return ::core::result::Result::Err(::core::convert::Into::into($error))
        }
    }};
}

/// The interface for an ERC-1155 compliant contract.
///
/// The interface is defined here: <https://eips.ethereum.org/EIPS/eip-1155>.
///
/// The goal of ERC-1155 is to allow a single contract to manage a variety of assets.
/// These assets can be fungible, non-fungible, or a combination.
///
/// By tracking multiple assets the ERC-1155 standard is able to support batch transfers,
/// which make it easy to transfer a mix of multiple tokens at once.
#[ink::trait_definition]
pub trait Erc1155 {
    /// Transfer a `value` amount of `token_id` tokens to the `to` account from the `from`
    /// account.
    ///
    /// Note that the call does not have to originate from the `from` account, and may
    /// originate from any account which is approved to transfer `from`'s tokens.
    #[ink(message)]
    fn safe_transfer_from(
        &mut self,
        from: Address,
        to: Address,
        token_id: TokenId,
        value: U256,
        data: Vec<u8>,
    ) -> Result<()>;

    /// Perform a batch transfer of `token_ids` to the `to` account from the `from`
    /// account.
    ///
    /// The number of `values` specified to be transferred must match the number of
    /// `token_ids`, otherwise this call will revert.
    ///
    /// Note that the call does not have to originate from the `from` account, and may
    /// originate from any account which is approved to transfer `from`'s tokens.
    #[ink(message)]
    fn safe_batch_transfer_from(
        &mut self,
        from: Address,
        to: Address,
        token_ids: Vec<TokenId>,
        values: Vec<U256>,
        data: Vec<u8>,
    ) -> Result<()>;

    /// Query the balance of a specific token for the provided account.
    #[ink(message)]
    fn balance_of(&self, owner: Address, token_id: TokenId) -> U256;

    /// Query the balances for a set of tokens for a set of accounts.
    ///
    /// E.g use this call if you want to query what Alice and Bob's balances are for
    /// Tokens ID 1 and ID 2.
    ///
    /// This will return all the balances for a given owner before moving on to the next
    /// owner. In the example above this means that the return value should look like:
    ///
    /// [Alice U256 of Token ID 1, Alice U256 of Token ID 2, Bob U256 of Token ID
    /// 1, Bob U256 of Token ID 2]
    #[ink(message)]
    fn balance_of_batch(
        &self,
        owners: Vec<Address>,
        token_ids: Vec<TokenId>,
    ) -> Vec<U256>;

    /// Enable or disable a third party, known as an `operator`, to control all tokens on
    /// behalf of the caller.
    #[ink(message)]
    fn set_approval_for_all(&mut self, operator: Address, approved: bool) -> Result<()>;

    /// Query if the given `operator` is allowed to control all of `owner`'s tokens.
    #[ink(message)]
    fn is_approved_for_all(&self, owner: Address, operator: Address) -> bool;
}

/// The interface for an ERC-1155 Token Receiver contract.
///
/// The interface is defined here: <https://eips.ethereum.org/EIPS/eip-1155>.
///
/// Smart contracts which want to accept token transfers must implement this interface. By
/// default if a contract does not support this interface any transactions originating
/// from an ERC-1155 compliant contract which attempt to transfer tokens directly to the
/// contract's address must be reverted.
#[ink::trait_definition]
pub trait Erc1155TokenReceiver {
    /// Handle the receipt of a single ERC-1155 token.
    ///
    /// This should be called by a compliant ERC-1155 contract if the intended recipient
    /// is a smart contract.
    ///
    /// If the smart contract implementing this interface accepts token transfers then it
    /// must return `ON_ERC_1155_RECEIVED_SELECTOR` from this function. To reject a
    /// transfer it must revert.
    ///
    /// Any callers must revert if they receive anything other than
    /// `ON_ERC_1155_RECEIVED_SELECTOR` as a return value.
    #[ink(message, selector = 0xF23A6E61)]
    fn on_received(
        &mut self,
        operator: Address,
        from: Address,
        token_id: TokenId,
        value: U256,
        data: Vec<u8>,
    ) -> Vec<u8>;

    /// Handle the receipt of multiple ERC-1155 tokens.
    ///
    /// This should be called by a compliant ERC-1155 contract if the intended recipient
    /// is a smart contract.
    ///
    /// If the smart contract implementing this interface accepts token transfers then it
    /// must return `BATCH_ON_ERC_1155_RECEIVED_SELECTOR` from this function. To
    /// reject a transfer it must revert.
    ///
    /// Any callers must revert if they receive anything other than
    /// `BATCH_ON_ERC_1155_RECEIVED_SELECTOR` as a return value.
    #[ink(message, selector = 0xBC197C81)]
    fn on_batch_received(
        &mut self,
        operator: Address,
        from: Address,
        token_ids: Vec<TokenId>,
        values: Vec<U256>,
        data: Vec<u8>,
    ) -> Vec<u8>;
}

#[ink::contract]
mod erc1155 {
    use super::*;

    use ink::{
        U256,
        storage::Mapping,
    };

    type Owner = Address;
    type Operator = Address;

    /// Indicate that a token transfer has occurred.
    ///
    /// This must be emitted even if a zero value transfer occurs.
    #[ink(event)]
    pub struct TransferSingle {
        #[ink(topic)]
        operator: Option<Address>,
        #[ink(topic)]
        from: Option<Address>,
        #[ink(topic)]
        to: Option<Address>,
        token_id: TokenId,
        value: U256,
    }

    /// Indicate that an approval event has happened.
    #[ink(event)]
    pub struct ApprovalForAll {
        #[ink(topic)]
        owner: Address,
        #[ink(topic)]
        operator: Address,
        approved: bool,
    }

    /// Indicate that a token's URI has been updated.
    #[ink(event)]
    pub struct Uri {
        value: ink::prelude::string::String,
        #[ink(topic)]
        token_id: TokenId,
    }

    /// An ERC-1155 contract.
    #[ink(storage)]
    #[derive(Default)]
    pub struct Contract {
        /// Tracks the balances of accounts across the different tokens that they might
        /// be holding.
        balances: Mapping<(Address, TokenId), U256>,
        /// Which accounts (called operators) have been approved to spend funds on behalf
        /// of an owner.
        approvals: Mapping<(Owner, Operator), ()>,
        /// A unique identifier for the tokens which have been minted (and are therefore
        /// supported) by this contract.
        token_id_nonce: TokenId,
    }

    impl Contract {
        /// Initialize a default instance of this ERC-1155 implementation.
        #[ink(constructor)]
        pub fn new() -> Self {
            Default::default()
        }

        /// Create the initial supply for a token.
        ///
        /// The initial supply will be provided to the caller (a.k.a the minter), and the
        /// `token_id` will be assigned by the smart contract.
        ///
        /// Note that as implemented anyone can create tokens. If you were to instantiate
        /// this contract in a production environment you'd probably want to lock down
        /// the addresses that are allowed to create tokens.
        #[ink(message)]
        pub fn create(&mut self, value: U256) -> TokenId {
            let caller = self.env().caller();

            // Given that TokenId is a `u128` the likelihood of this overflowing is pretty
            // slim.
            #[allow(clippy::arithmetic_side_effects)]
            {
                self.token_id_nonce += 1;
            }
            self.balances.insert((caller, self.token_id_nonce), &value);

            // Emit transfer event but with mint semantics
            self.env().emit_event(TransferSingle {
                operator: Some(caller),
                from: None,
                to: if value == U256::zero() {
                    None
                } else {
                    Some(caller)
                },
                token_id: self.token_id_nonce,
                value,
            });

            self.token_id_nonce
        }

        /// Mint a `value` amount of `token_id` tokens.
        ///
        /// It is assumed that the token has already been `create`-ed. The newly minted
        /// supply will be assigned to the caller (a.k.a the minter).
        ///
        /// Note that as implemented anyone can mint tokens. If you were to instantiate
        /// this contract in a production environment you'd probably want to lock down
        /// the addresses that are allowed to mint tokens.
        #[ink(message)]
        pub fn mint(&mut self, token_id: TokenId, value: U256) -> Result<()> {
            ensure!(token_id <= self.token_id_nonce, Error::UnexistentToken);

            let caller = self.env().caller();
            self.balances.insert((caller, token_id), &value);

            // Emit transfer event but with mint semantics
            self.env().emit_event(TransferSingle {
                operator: Some(caller),
                from: None,
                to: Some(caller),
                token_id,
                value,
            });

            Ok(())
        }

        // Helper function for performing single token transfers.
        //
        // Should not be used directly since it's missing certain checks which are
        // important to the ERC-1155 standard (it is expected that the caller has
        // already performed these).
        //
        // # Panics
        //
        // If `from` does not hold any `token_id` tokens.
        fn perform_transfer(
            &mut self,
            from: Address,
            to: Address,
            token_id: TokenId,
            value: U256,
        ) {
            let mut sender_balance = self
                .balances
                .get((from, token_id))
                .expect("Caller should have ensured that `from` holds `token_id`.");
            // checks that sender_balance >= value were performed by caller
            #[allow(clippy::arithmetic_side_effects)]
            {
                sender_balance -= value;
            }
            self.balances.insert((from, token_id), &sender_balance);

            let mut recipient_balance =
                self.balances.get((to, token_id)).unwrap_or(U256::zero());
            recipient_balance = recipient_balance.checked_add(value).unwrap();
            self.balances.insert((to, token_id), &recipient_balance);

            let caller = self.env().caller();
            self.env().emit_event(TransferSingle {
                operator: Some(caller),
                from: Some(from),
                to: Some(to),
                token_id,
                value,
            });
        }

        // Check if the address at `to` is a smart contract which accepts ERC-1155 token
        // transfers.
        //
        // If they're a smart contract which **doesn't** accept tokens transfers this call
        // will revert. Otherwise we risk locking user funds at in that contract
        // with no chance of recovery.
        #[cfg_attr(test, allow(unused_variables))]
        fn transfer_acceptance_check(
            &mut self,
            caller: Address,
            from: Address,
            to: Address,
            token_id: TokenId,
            value: U256,
            data: Vec<u8>,
        ) {
            // This is disabled during tests due to the use of `invoke_contract()` not
            // being supported (tests end up panicking).
            #[cfg(not(test))]
            {
                use ink::env::call::{
                    ExecutionInput,
                    Selector,
                    build_call,
                };

                // If our recipient is a smart contract we need to see if they accept or
                // reject this transfer. If they reject it we need to revert the call.
                let result = build_call::<Environment>()
                    .call(to)
                    .ref_time_limit(5000)
                    .exec_input(
                        ExecutionInput::new(Selector::new(ON_ERC_1155_RECEIVED_SELECTOR))
                            .push_arg(caller)
                            .push_arg(from)
                            .push_arg(token_id)
                            .push_arg(value)
                            .push_arg(data),
                    )
                    .returns::<Vec<u8>>()
                    .params()
                    .try_invoke();

                match result {
                    Ok(v) => {
                        /*
                        // todo
                        ink::env::debug_println!(
                            "Received return value \"{:?}\" from contract {:?}",
                            v.clone().expect(
                                "Call should be valid, don't expect a `LangError`."
                            ),
                            from
                        );
                        */
                        assert_eq!(
                            v.clone().expect("Call should be valid, don't expect a `LangError`."),
                            &ON_ERC_1155_RECEIVED_SELECTOR[..],
                            "The recipient contract at {to:?} does not accept token transfers.\n
                            Expected: {ON_ERC_1155_RECEIVED_SELECTOR:?}, Got {v:?}"
                        )
                    }
                    Err(e) => {
                        use ink::env::ReturnErrorCode;

                        match e {
                            ink::env::Error::ReturnError(
                                ReturnErrorCode::Unknown, /* todo: these error codes
                                                           * don't exist in uapi yet,
                                                           * fallback
                                                           * is `Unknown`
                                                           * ReturnErrorCode::CodeNotFound | ReturnErrorCode::NotCallable, */
                            ) => {
                                // Our recipient wasn't a smart contract, so there's
                                // nothing more for
                                // us to do
                                // todo
                                // ink::env::debug_println!("Recipient at {:?} from is not
                                // a smart contract ({:?})", from, e);
                            }
                            _ => {
                                // We got some sort of error from the call to our
                                // recipient smart
                                // contract, and as such we must revert this call
                                panic!(
                                    "Got error \"{e:?}\" while trying to call {from:?}"
                                )
                            }
                        }
                    }
                }
            }
        }
    }

    impl super::Erc1155 for Contract {
        #[ink(message)]
        fn safe_transfer_from(
            &mut self,
            from: Address,
            to: Address,
            token_id: TokenId,
            value: U256,
            data: Vec<u8>,
        ) -> Result<()> {
            let caller = self.env().caller();
            if caller != from {
                ensure!(self.is_approved_for_all(from, caller), Error::NotApproved);
            }

            ensure!(to != zero_address(), Error::ZeroAddressTransfer);

            let balance = self.balance_of(from, token_id);
            ensure!(balance >= value, Error::InsufficientU256);

            self.perform_transfer(from, to, token_id, value);
            self.transfer_acceptance_check(caller, from, to, token_id, value, data);

            Ok(())
        }

        #[ink(message)]
        fn safe_batch_transfer_from(
            &mut self,
            from: Address,
            to: Address,
            token_ids: Vec<TokenId>,
            values: Vec<U256>,
            data: Vec<u8>,
        ) -> Result<()> {
            let caller = self.env().caller();
            if caller != from {
                ensure!(self.is_approved_for_all(from, caller), Error::NotApproved);
            }

            ensure!(to != zero_address(), Error::ZeroAddressTransfer);
            ensure!(!token_ids.is_empty(), Error::BatchTransferMismatch);
            ensure!(
                token_ids.len() == values.len(),
                Error::BatchTransferMismatch,
            );

            let transfers = token_ids.iter().zip(values.iter());
            for (&id, &v) in transfers.clone() {
                let balance = self.balance_of(from, id);
                ensure!(balance >= v, Error::InsufficientU256);
            }

            for (&id, &v) in transfers {
                self.perform_transfer(from, to, id, v);
            }

            // Can use any token ID/value here, we really just care about knowing if
            // `to` is a smart contract which accepts transfers
            self.transfer_acceptance_check(
                caller,
                from,
                to,
                token_ids[0],
                values[0],
                data,
            );

            Ok(())
        }

        #[ink(message)]
        fn balance_of(&self, owner: Address, token_id: TokenId) -> U256 {
            self.balances.get((owner, token_id)).unwrap_or(0.into())
        }

        #[ink(message)]
        fn balance_of_batch(
            &self,
            owners: Vec<Address>,
            token_ids: Vec<TokenId>,
        ) -> Vec<U256> {
            let mut output = Vec::new();
            for o in &owners {
                for t in &token_ids {
                    let amount = self.balance_of(*o, *t);
                    output.push(amount);
                }
            }
            output
        }

        #[ink(message)]
        fn set_approval_for_all(
            &mut self,
            operator: Address,
            approved: bool,
        ) -> Result<()> {
            let caller = self.env().caller();
            ensure!(operator != caller, Error::SelfApproval);

            if approved {
                self.approvals.insert((&caller, &operator), &());
            } else {
                self.approvals.remove((&caller, &operator));
            }

            self.env().emit_event(ApprovalForAll {
                owner: caller,
                operator,
                approved,
            });

            Ok(())
        }

        #[ink(message)]
        fn is_approved_for_all(&self, owner: Address, operator: Address) -> bool {
            self.approvals.contains((&owner, &operator))
        }
    }

    impl super::Erc1155TokenReceiver for Contract {
        #[ink(message, selector = 0xF23A6E61)]
        fn on_received(
            &mut self,
            _operator: Address,
            _from: Address,
            _token_id: TokenId,
            _value: U256,
            _data: Vec<u8>,
        ) -> Vec<u8> {
            // The ERC-1155 standard dictates that if a contract does not accept token
            // transfers directly to the contract, then the contract must
            // revert.
            //
            // This prevents a user from unintentionally transferring tokens to a smart
            // contract and getting their funds stuck without any sort of
            // recovery mechanism.
            //
            // Note that the choice of whether or not to accept tokens is implementation
            // specific, and we've decided to not accept them in this
            // implementation.
            unimplemented!("This smart contract does not accept token transfer.")
        }

        #[ink(message, selector = 0xBC197C81)]
        fn on_batch_received(
            &mut self,
            _operator: Address,
            _from: Address,
            _token_ids: Vec<TokenId>,
            _values: Vec<U256>,
            _data: Vec<u8>,
        ) -> Vec<u8> {
            // The ERC-1155 standard dictates that if a contract does not accept token
            // transfers directly to the contract, then the contract must
            // revert.
            //
            // This prevents a user from unintentionally transferring tokens to a smart
            // contract and getting their funds stuck without any sort of
            // recovery mechanism.
            //
            // Note that the choice of whether or not to accept tokens is implementation
            // specific, and we've decided to not accept them in this
            // implementation.
            unimplemented!("This smart contract does not accept batch token transfers.")
        }
    }

    /// Helper for referencing the zero address (`0x00`). Note that in practice this
    /// address should not be treated in any special way (such as a default
    /// placeholder) since it has a known private key.
    fn zero_address() -> Address {
        [0u8; 20].into()
    }

    #[cfg(test)]
    mod tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;
        use crate::Erc1155;

        fn set_sender(sender: Address) {
            ink::env::test::set_caller(sender);
        }

        fn default_accounts() -> ink::env::test::DefaultAccounts {
            ink::env::test::default_accounts()
        }

        fn alice() -> Address {
            default_accounts().alice
        }

        fn bob() -> Address {
            default_accounts().bob
        }

        fn charlie() -> Address {
            default_accounts().charlie
        }

        fn init_contract() -> Contract {
            set_sender(alice());
            let mut erc = Contract::new();
            erc.balances.insert((alice(), 1), &U256::from(10));
            erc.balances.insert((alice(), 2), &U256::from(20));
            erc.balances.insert((bob(), 1), &U256::from(10));

            erc
        }

        #[ink::test]
        fn can_get_correct_balance_of() {
            let erc = init_contract();

            assert_eq!(erc.balance_of(alice(), 1), U256::from(10));
            assert_eq!(erc.balance_of(alice(), 2), U256::from(20));
            assert_eq!(erc.balance_of(alice(), 3), U256::zero());
            assert_eq!(erc.balance_of(bob(), 2), U256::zero());
        }

        #[ink::test]
        fn can_get_correct_batch_balance_of() {
            let erc = init_contract();

            assert_eq!(
                erc.balance_of_batch(vec![alice()], vec![1, 2, 3]),
                vec![U256::from(10), 20.into(), 0.into()]
            );
            assert_eq!(
                erc.balance_of_batch(vec![alice(), bob()], vec![1]),
                vec![U256::from(10), 10.into()]
            );

            assert_eq!(
                erc.balance_of_batch(vec![alice(), bob(), charlie()], vec![1, 2]),
                vec![
                    U256::from(10),
                    20.into(),
                    10.into(),
                    0.into(),
                    0.into(),
                    0.into()
                ]
            );
        }

        #[ink::test]
        fn can_send_tokens_between_accounts() {
            let mut erc = init_contract();

            assert!(
                erc.safe_transfer_from(alice(), bob(), 1, 5.into(), vec![])
                    .is_ok()
            );
            assert_eq!(erc.balance_of(alice(), 1), U256::from(5));
            assert_eq!(erc.balance_of(bob(), 1), U256::from(15));

            assert!(
                erc.safe_transfer_from(alice(), bob(), 2, 5.into(), vec![])
                    .is_ok()
            );
            assert_eq!(erc.balance_of(alice(), 2), U256::from(15));
            assert_eq!(erc.balance_of(bob(), 2), U256::from(5));
        }

        #[ink::test]
        fn sending_too_many_tokens_fails() {
            let mut erc = init_contract();
            let res = erc.safe_transfer_from(alice(), bob(), 1, 99.into(), vec![]);
            assert_eq!(res.unwrap_err(), Error::InsufficientU256);
        }

        #[ink::test]
        fn sending_tokens_to_zero_address_fails() {
            let burn: Address = [0; 20].into();

            let mut erc = init_contract();
            let res = erc.safe_transfer_from(alice(), burn, 1, 10.into(), vec![]);
            assert_eq!(res.unwrap_err(), Error::ZeroAddressTransfer);
        }

        #[ink::test]
        fn can_send_batch_tokens() {
            let mut erc = init_contract();
            assert!(
                erc.safe_batch_transfer_from(
                    alice(),
                    bob(),
                    vec![1, 2],
                    vec![U256::from(5), U256::from(10)],
                    vec![]
                )
                .is_ok()
            );

            let balances = erc.balance_of_batch(vec![alice(), bob()], vec![1, 2]);
            assert_eq!(
                balances,
                vec![U256::from(5), 10.into(), 15.into(), 10.into()]
            );
        }

        #[ink::test]
        fn rejects_batch_if_lengths_dont_match() {
            let mut erc = init_contract();
            let res = erc.safe_batch_transfer_from(
                alice(),
                bob(),
                vec![1, 2, 3],
                vec![U256::from(5)],
                vec![],
            );
            assert_eq!(res.unwrap_err(), Error::BatchTransferMismatch);
        }

        #[ink::test]
        fn batch_transfers_fail_if_len_is_zero() {
            let mut erc = init_contract();
            let res =
                erc.safe_batch_transfer_from(alice(), bob(), vec![], vec![], vec![]);
            assert_eq!(res.unwrap_err(), Error::BatchTransferMismatch);
        }

        #[ink::test]
        fn operator_can_send_tokens() {
            let mut erc = init_contract();

            let owner = alice();
            let operator = bob();

            set_sender(owner);
            assert!(erc.set_approval_for_all(operator, true).is_ok());

            set_sender(operator);
            assert!(
                erc.safe_transfer_from(owner, charlie(), 1, 5.into(), vec![])
                    .is_ok()
            );
            assert_eq!(erc.balance_of(alice(), 1), U256::from(5));
            assert_eq!(erc.balance_of(charlie(), 1), U256::from(5));
        }

        #[ink::test]
        fn approvals_work() {
            let mut erc = init_contract();
            let owner = alice();
            let operator = bob();
            let another_operator = charlie();

            // Note: All of these tests are from the context of the owner who is either
            // allowing or disallowing an operator to control their funds.
            set_sender(owner);
            assert!(!erc.is_approved_for_all(owner, operator));

            assert!(erc.set_approval_for_all(operator, true).is_ok());
            assert!(erc.is_approved_for_all(owner, operator));

            assert!(erc.set_approval_for_all(another_operator, true).is_ok());
            assert!(erc.is_approved_for_all(owner, another_operator));

            assert!(erc.set_approval_for_all(operator, false).is_ok());
            assert!(!erc.is_approved_for_all(owner, operator));
        }

        #[ink::test]
        fn minting_tokens_works() {
            let mut erc = Contract::new();

            set_sender(alice());
            assert_eq!(erc.create(0.into()), 1);
            assert_eq!(erc.balance_of(alice(), 1), U256::zero());

            assert!(erc.mint(1, 123.into()).is_ok());
            assert_eq!(erc.balance_of(alice(), 1), U256::from(123));
        }

        #[ink::test]
        fn minting_not_allowed_for_nonexistent_tokens() {
            let mut erc = Contract::new();

            let res = erc.mint(1, 123.into());
            assert_eq!(res.unwrap_err(), Error::UnexistentToken);
        }
    }
}
