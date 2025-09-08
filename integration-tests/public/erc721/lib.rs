//! # ERC-721
//!
//! This is an ERC-721 Token implementation.
//!
//! ## Warning
//!
//! This contract is an *example*. It is neither audited nor endorsed for production use.
//! Do **not** rely on it to keep anything of value secure.
//!
//! ## Overview
//!
//! This contract demonstrates how to build non-fungible or unique tokens using ink!.
//!
//! ## Error Handling
//!
//! Any function that modifies the state returns a `Result` type and does not changes the
//! state if the `Error` occurs.
//! The errors are defined as an `enum` type. Any other error or invariant violation
//! triggers a panic and therefore rolls back the transaction.
//!
//! ## Token Management
//!
//! After creating a new token, the function caller becomes the owner.
//! A token can be created, transferred, or destroyed.
//!
//! Token owners can assign other accounts for transferring specific tokens on their
//! behalf. It is also possible to authorize an operator (higher rights) for another
//! account to handle tokens.
//!
//! ### Token Creation
//!
//! Token creation start by calling the `mint(&mut self, id: u32)` function.
//! The token owner becomes the function caller. The Token ID needs to be specified
//! as the argument on this function call.
//!
//! ### Token Transfer
//!
//! Transfers may be initiated by:
//! - The owner of a token
//! - The approved address of a token
//! - An authorized operator of the current owner of a token
//!
//! The token owner can transfer a token by calling the `transfer` or `transfer_from`
//! functions. An approved address can make a token transfer by calling the
//! `transfer_from` function. Operators can transfer tokens on another account's behalf or
//! can approve a token transfer for a different account.
//!
//! ### Token Removal
//!
//! Tokens can be destroyed by burning them. Only the token owner is allowed to burn a
//! token.

#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod erc721 {
    use ink::storage::Mapping;

    /// A token ID.
    pub type TokenId = u32;

    #[ink(storage)]
    #[derive(Default)]
    pub struct Erc721 {
        /// Mapping from token to owner.
        token_owner: Mapping<TokenId, Address>,
        /// Mapping from token to approvals users.
        token_approvals: Mapping<TokenId, Address>,
        /// Mapping from owner to number of owned token.
        owned_tokens_count: Mapping<Address, u32>,
        /// Mapping from owner to operator approvals.
        operator_approvals: Mapping<(Address, Address), ()>,
    }

    #[derive(Debug, PartialEq, Eq, Copy, Clone)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    pub enum Error {
        NotOwner,
        NotApproved,
        TokenExists,
        TokenNotFound,
        CannotInsert,
        CannotFetchValue,
        NotAllowed,
    }

    /// Event emitted when a token transfer occurs.
    #[ink(event)]
    pub struct Transfer {
        #[ink(topic)]
        from: Option<Address>,
        #[ink(topic)]
        to: Option<Address>,
        #[ink(topic)]
        id: TokenId,
    }

    /// Event emitted when a token approve occurs.
    #[ink(event)]
    pub struct Approval {
        #[ink(topic)]
        from: Address,
        #[ink(topic)]
        to: Address,
        #[ink(topic)]
        id: TokenId,
    }

    /// Event emitted when an operator is enabled or disabled for an owner.
    /// The operator can manage all NFTs of the owner.
    #[ink(event)]
    pub struct ApprovalForAll {
        #[ink(topic)]
        owner: Address,
        #[ink(topic)]
        operator: Address,
        approved: bool,
    }

    impl Erc721 {
        /// Creates a new ERC-721 token contract.
        #[ink(constructor)]
        pub fn new() -> Self {
            Default::default()
        }

        /// Returns the balance of the owner.
        ///
        /// This represents the amount of unique tokens the owner has.
        #[ink(message)]
        pub fn balance_of(&self, owner: Address) -> u32 {
            self.balance_of_or_zero(&owner)
        }

        /// Returns the owner of the token.
        #[ink(message)]
        pub fn owner_of(&self, id: TokenId) -> Option<Address> {
            self.token_owner.get(id)
        }

        /// Returns the approved account ID for this token if any.
        #[ink(message)]
        pub fn get_approved(&self, id: TokenId) -> Option<Address> {
            self.token_approvals.get(id)
        }

        /// Returns `true` if the operator is approved by the owner.
        #[ink(message)]
        pub fn is_approved_for_all(&self, owner: Address, operator: Address) -> bool {
            self.approved_for_all(owner, operator)
        }

        /// Approves or disapproves the operator for all tokens of the caller.
        #[ink(message)]
        pub fn set_approval_for_all(
            &mut self,
            to: Address,
            approved: bool,
        ) -> Result<(), Error> {
            self.approve_for_all(to, approved)?;
            Ok(())
        }

        /// Approves the account to transfer the specified token on behalf of the caller.
        #[ink(message)]
        pub fn approve(&mut self, to: Address, id: TokenId) -> Result<(), Error> {
            self.approve_for(&to, id)?;
            Ok(())
        }

        /// Transfers the token from the caller to the given destination.
        #[ink(message)]
        pub fn transfer(
            &mut self,
            destination: Address,
            id: TokenId,
        ) -> Result<(), Error> {
            let caller = self.env().caller();
            self.transfer_token_from(&caller, &destination, id)?;
            Ok(())
        }

        /// Transfer approved or owned token.
        #[ink(message)]
        pub fn transfer_from(
            &mut self,
            from: Address,
            to: Address,
            id: TokenId,
        ) -> Result<(), Error> {
            self.transfer_token_from(&from, &to, id)?;
            Ok(())
        }

        /// Creates a new token.
        #[ink(message)]
        pub fn mint(&mut self, id: TokenId) -> Result<(), Error> {
            let caller = self.env().caller();
            self.add_token_to(&caller, id)?;
            self.env().emit_event(Transfer {
                from: Some(Address::from([0x0; 20])),
                to: Some(caller),
                id,
            });
            Ok(())
        }

        /// Deletes an existing token. Only the owner can burn the token.
        #[ink(message)]
        pub fn burn(&mut self, id: TokenId) -> Result<(), Error> {
            let caller = self.env().caller();
            let Self {
                token_owner,
                owned_tokens_count,
                ..
            } = self;

            let owner = token_owner.get(id).ok_or(Error::TokenNotFound)?;
            if owner != caller {
                return Err(Error::NotOwner);
            };

            let count = owned_tokens_count
                .get(caller)
                .map(|c| c.checked_sub(1).unwrap())
                .ok_or(Error::CannotFetchValue)?;
            owned_tokens_count.insert(caller, &count);
            token_owner.remove(id);
            self.clear_approval(id);

            self.env().emit_event(Transfer {
                from: Some(caller),
                to: Some(Address::from([0x0; 20])),
                id,
            });

            Ok(())
        }

        /// Transfers token `id` `from` the sender to the `to` `Address`.
        fn transfer_token_from(
            &mut self,
            from: &Address,
            to: &Address,
            id: TokenId,
        ) -> Result<(), Error> {
            let caller = self.env().caller();
            let owner = self.owner_of(id).ok_or(Error::TokenNotFound)?;
            if !self.approved_or_owner(caller, id, owner) {
                return Err(Error::NotApproved);
            };
            if owner != *from {
                return Err(Error::NotOwner);
            };
            self.clear_approval(id);
            self.remove_token_from(from, id)?;
            self.add_token_to(to, id)?;
            self.env().emit_event(Transfer {
                from: Some(*from),
                to: Some(*to),
                id,
            });
            Ok(())
        }

        /// Removes token `id` from the owner.
        fn remove_token_from(
            &mut self,
            from: &Address,
            id: TokenId,
        ) -> Result<(), Error> {
            let Self {
                token_owner,
                owned_tokens_count,
                ..
            } = self;

            if !token_owner.contains(id) {
                return Err(Error::TokenNotFound);
            }

            let count = owned_tokens_count
                .get(from)
                .map(|c| c.checked_sub(1).unwrap())
                .ok_or(Error::CannotFetchValue)?;
            owned_tokens_count.insert(from, &count);
            token_owner.remove(id);

            Ok(())
        }

        /// Adds the token `id` to the `to` AccountID.
        fn add_token_to(&mut self, to: &Address, id: TokenId) -> Result<(), Error> {
            let Self {
                token_owner,
                owned_tokens_count,
                ..
            } = self;

            if token_owner.contains(id) {
                return Err(Error::TokenExists);
            }

            if *to == Address::from([0x0; 20]) {
                return Err(Error::NotAllowed);
            };

            let count = owned_tokens_count
                .get(to)
                .map(|c| c.checked_add(1).unwrap())
                .unwrap_or(1);

            owned_tokens_count.insert(to, &count);
            token_owner.insert(id, to);

            Ok(())
        }

        /// Approves or disapproves the operator to transfer all tokens of the caller.
        fn approve_for_all(&mut self, to: Address, approved: bool) -> Result<(), Error> {
            let caller = self.env().caller();
            if to == caller {
                return Err(Error::NotAllowed);
            }
            self.env().emit_event(ApprovalForAll {
                owner: caller,
                operator: to,
                approved,
            });

            if approved {
                self.operator_approvals.insert((&caller, &to), &());
            } else {
                self.operator_approvals.remove((&caller, &to));
            }

            Ok(())
        }

        /// Approve the passed `Address` to transfer the specified token on behalf of
        /// the message's sender.
        fn approve_for(&mut self, to: &Address, id: TokenId) -> Result<(), Error> {
            let caller = self.env().caller();
            let owner = self.owner_of(id).ok_or(Error::TokenNotFound)?;
            if !(owner == caller || self.approved_for_all(owner, caller)) {
                return Err(Error::NotAllowed);
            };

            if *to == Address::from([0x0; 20]) {
                return Err(Error::NotAllowed);
            };

            if self.token_approvals.contains(id) {
                return Err(Error::CannotInsert);
            } else {
                self.token_approvals.insert(id, to);
            }

            self.env().emit_event(Approval {
                from: caller,
                to: *to,
                id,
            });

            Ok(())
        }

        /// Removes existing approval from token `id`.
        fn clear_approval(&mut self, id: TokenId) {
            self.token_approvals.remove(id);
        }

        // Returns the total number of tokens from an account.
        fn balance_of_or_zero(&self, of: &Address) -> u32 {
            self.owned_tokens_count.get(of).unwrap_or(0)
        }

        /// Gets an operator on other Account's behalf.
        fn approved_for_all(&self, owner: Address, operator: Address) -> bool {
            self.operator_approvals.contains((&owner, &operator))
        }

        /// Returns true if the `Address` `from` is the owner of token `id`
        /// or it has been approved on behalf of the token `id` owner.
        fn approved_or_owner(&self, from: Address, id: TokenId, owner: Address) -> bool {
            from != Address::from([0x0; 20])
                && (from == owner
                    || self.token_approvals.get(id) == Some(from)
                    || self.approved_for_all(owner, from))
        }
    }

    /// Unit tests
    #[cfg(test)]
    mod tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        #[ink::test]
        fn mint_works() {
            let accounts = ink::env::test::default_accounts();
            set_caller(accounts.alice);
            // Create a new contract instance.
            let mut erc721 = Erc721::new();
            // Token 1 does not exists.
            assert_eq!(erc721.owner_of(1), None);
            // Alice does not owns tokens.
            assert_eq!(erc721.balance_of(accounts.alice), 0);
            // Create token Id 1.
            assert_eq!(erc721.mint(1), Ok(()));
            // Alice owns 1 token.
            assert_eq!(erc721.balance_of(accounts.alice), 1);
        }

        #[ink::test]
        fn mint_existing_should_fail() {
            let accounts = ink::env::test::default_accounts();
            set_caller(accounts.alice);
            // Create a new contract instance.
            let mut erc721 = Erc721::new();
            // Create token Id 1.
            assert_eq!(erc721.mint(1), Ok(()));
            // The first Transfer event takes place
            assert_eq!(1, ink::env::test::recorded_events().len());
            // Alice owns 1 token.
            assert_eq!(erc721.balance_of(accounts.alice), 1);
            // Alice owns token Id 1.
            assert_eq!(erc721.owner_of(1), Some(accounts.alice));
            // Cannot create  token Id if it exists.
            // Bob cannot own token Id 1.
            assert_eq!(erc721.mint(1), Err(Error::TokenExists));
        }

        #[ink::test]
        fn transfer_works() {
            let accounts = ink::env::test::default_accounts();
            set_caller(accounts.alice);
            // Create a new contract instance.
            let mut erc721 = Erc721::new();
            // Create token Id 1 for Alice
            assert_eq!(erc721.mint(1), Ok(()));
            // Alice owns token 1
            assert_eq!(erc721.balance_of(accounts.alice), 1);
            // Bob does not owns any token
            assert_eq!(erc721.balance_of(accounts.bob), 0);
            // The first Transfer event takes place
            assert_eq!(1, ink::env::test::recorded_events().len());
            // Alice transfers token 1 to Bob
            assert_eq!(erc721.transfer(accounts.bob, 1), Ok(()));
            // The second Transfer event takes place
            assert_eq!(2, ink::env::test::recorded_events().len());
            // Bob owns token 1
            assert_eq!(erc721.balance_of(accounts.bob), 1);
        }

        #[ink::test]
        fn invalid_transfer_should_fail() {
            let accounts = ink::env::test::default_accounts();
            set_caller(accounts.alice);
            // Create a new contract instance.
            let mut erc721 = Erc721::new();
            // Transfer token fails if it does not exists.
            assert_eq!(erc721.transfer(accounts.bob, 2), Err(Error::TokenNotFound));
            // Token Id 2 does not exists.
            assert_eq!(erc721.owner_of(2), None);
            // Create token Id 2.
            assert_eq!(erc721.mint(2), Ok(()));
            // Alice owns 1 token.
            assert_eq!(erc721.balance_of(accounts.alice), 1);
            // Token Id 2 is owned by Alice.
            assert_eq!(erc721.owner_of(2), Some(accounts.alice));
            // Set Bob as caller
            set_caller(accounts.bob);
            // Bob cannot transfer not owned tokens.
            assert_eq!(erc721.transfer(accounts.eve, 2), Err(Error::NotApproved));
        }

        #[ink::test]
        fn approved_transfer_works() {
            let accounts = ink::env::test::default_accounts();
            set_caller(accounts.alice);
            // Create a new contract instance.
            let mut erc721 = Erc721::new();
            // Create token Id 1.
            assert_eq!(erc721.mint(1), Ok(()));
            // Token Id 1 is owned by Alice.
            assert_eq!(erc721.owner_of(1), Some(accounts.alice));
            // Approve token Id 1 transfer for Bob on behalf of Alice.
            assert_eq!(erc721.approve(accounts.bob, 1), Ok(()));
            // Set Bob as caller
            set_caller(accounts.bob);
            // Bob transfers token Id 1 from Alice to Eve.
            assert_eq!(
                erc721.transfer_from(accounts.alice, accounts.eve, 1),
                Ok(())
            );
            // TokenId 3 is owned by Eve.
            assert_eq!(erc721.owner_of(1), Some(accounts.eve));
            // Alice does not owns tokens.
            assert_eq!(erc721.balance_of(accounts.alice), 0);
            // Bob does not owns tokens.
            assert_eq!(erc721.balance_of(accounts.bob), 0);
            // Eve owns 1 token.
            assert_eq!(erc721.balance_of(accounts.eve), 1);
        }

        #[ink::test]
        fn approved_for_all_works() {
            let accounts = ink::env::test::default_accounts();
            set_caller(accounts.alice);
            // Create a new contract instance.
            let mut erc721 = Erc721::new();
            // Create token Id 1.
            assert_eq!(erc721.mint(1), Ok(()));
            // Create token Id 2.
            assert_eq!(erc721.mint(2), Ok(()));
            // Alice owns 2 tokens.
            assert_eq!(erc721.balance_of(accounts.alice), 2);
            // Approve token Id 1 transfer for Bob on behalf of Alice.
            assert_eq!(erc721.set_approval_for_all(accounts.bob, true), Ok(()));
            // Bob is an approved operator for Alice
            assert!(erc721.is_approved_for_all(accounts.alice, accounts.bob));
            // Set Bob as caller
            set_caller(accounts.bob);
            // Bob transfers token Id 1 from Alice to Eve.
            assert_eq!(
                erc721.transfer_from(accounts.alice, accounts.eve, 1),
                Ok(())
            );
            // TokenId 1 is owned by Eve.
            assert_eq!(erc721.owner_of(1), Some(accounts.eve));
            // Alice owns 1 token.
            assert_eq!(erc721.balance_of(accounts.alice), 1);
            // Bob transfers token Id 2 from Alice to Eve.
            assert_eq!(
                erc721.transfer_from(accounts.alice, accounts.eve, 2),
                Ok(())
            );
            // Bob does not own tokens.
            assert_eq!(erc721.balance_of(accounts.bob), 0);
            // Eve owns 2 tokens.
            assert_eq!(erc721.balance_of(accounts.eve), 2);
            // Remove operator approval for Bob on behalf of Alice.
            set_caller(accounts.alice);
            assert_eq!(erc721.set_approval_for_all(accounts.bob, false), Ok(()));
            // Bob is not an approved operator for Alice.
            assert!(!erc721.is_approved_for_all(accounts.alice, accounts.bob));
        }

        #[ink::test]
        fn approve_nonexistent_token_should_fail() {
            let accounts = ink::env::test::default_accounts();
            set_caller(accounts.alice);
            // Create a new contract instance.
            let mut erc721 = Erc721::new();
            // Approve transfer of nonexistent token id 1
            assert_eq!(erc721.approve(accounts.bob, 1), Err(Error::TokenNotFound));
        }

        #[ink::test]
        fn not_approved_transfer_should_fail() {
            let accounts = ink::env::test::default_accounts();
            set_caller(accounts.alice);
            // Create a new contract instance.
            let mut erc721 = Erc721::new();
            // Create token Id 1.
            assert_eq!(erc721.mint(1), Ok(()));
            // Alice owns 1 token.
            assert_eq!(erc721.balance_of(accounts.alice), 1);
            // Bob does not owns tokens.
            assert_eq!(erc721.balance_of(accounts.bob), 0);
            // Eve does not owns tokens.
            assert_eq!(erc721.balance_of(accounts.eve), 0);
            // Set Eve as caller
            set_caller(accounts.eve);
            // Eve is not an approved operator by Alice.
            assert_eq!(
                erc721.transfer_from(accounts.alice, accounts.frank, 1),
                Err(Error::NotApproved)
            );
            // Alice owns 1 token.
            assert_eq!(erc721.balance_of(accounts.alice), 1);
            // Bob does not owns tokens.
            assert_eq!(erc721.balance_of(accounts.bob), 0);
            // Eve does not owns tokens.
            assert_eq!(erc721.balance_of(accounts.eve), 0);
        }

        #[ink::test]
        fn burn_works() {
            let accounts = ink::env::test::default_accounts();
            set_caller(accounts.alice);
            // Create a new contract instance.
            let mut erc721 = Erc721::new();
            // Create token Id 1 for Alice
            assert_eq!(erc721.mint(1), Ok(()));
            // Alice owns 1 token.
            assert_eq!(erc721.balance_of(accounts.alice), 1);
            // Alice owns token Id 1.
            assert_eq!(erc721.owner_of(1), Some(accounts.alice));
            // Destroy token Id 1.
            assert_eq!(erc721.burn(1), Ok(()));
            // Alice does not owns tokens.
            assert_eq!(erc721.balance_of(accounts.alice), 0);
            // Token Id 1 does not exists
            assert_eq!(erc721.owner_of(1), None);
        }

        #[ink::test]
        fn burn_fails_token_not_found() {
            // Create a new contract instance.
            let mut erc721 = Erc721::new();
            // Try burning a non existent token
            assert_eq!(erc721.burn(1), Err(Error::TokenNotFound));
        }

        #[ink::test]
        fn burn_fails_not_owner() {
            let accounts = ink::env::test::default_accounts();
            set_caller(accounts.alice);
            // Create a new contract instance.
            let mut erc721 = Erc721::new();
            // Create token Id 1 for Alice
            assert_eq!(erc721.mint(1), Ok(()));
            // Try burning this token with a different account
            set_caller(accounts.eve);
            assert_eq!(erc721.burn(1), Err(Error::NotOwner));
        }

        #[ink::test]
        fn burn_clears_approval() {
            let accounts = ink::env::test::default_accounts();
            set_caller(accounts.alice);
            // Create a new contract instance.
            let mut erc721 = Erc721::new();
            // Create token Id 1 for Alice
            assert_eq!(erc721.mint(1), Ok(()));
            // Alice gives approval to Bob to transfer token Id 1
            assert_eq!(erc721.approve(accounts.bob, 1), Ok(()));
            // Alice burns token
            assert_eq!(erc721.burn(1), Ok(()));
            // Set caller to Frank
            set_caller(accounts.frank);
            // Frank mints token Id 1
            assert_eq!(erc721.mint(1), Ok(()));
            // Set caller to Bob
            set_caller(accounts.bob);
            // Bob tries to transfer token Id 1 from Frank to himself
            assert_eq!(
                erc721.transfer_from(accounts.frank, accounts.bob, 1),
                Err(Error::NotApproved)
            );
        }

        #[ink::test]
        fn transfer_from_fails_not_owner() {
            let accounts = ink::env::test::default_accounts();
            set_caller(accounts.alice);
            // Create a new contract instance.
            let mut erc721 = Erc721::new();
            // Create token Id 1 for Alice
            assert_eq!(erc721.mint(1), Ok(()));
            // Bob can transfer alice's tokens
            assert_eq!(erc721.set_approval_for_all(accounts.bob, true), Ok(()));
            // Set caller to Frank
            set_caller(accounts.frank);
            // Create token Id 2 for Frank
            assert_eq!(erc721.mint(2), Ok(()));
            // Set caller to Bob
            set_caller(accounts.bob);
            // Bob makes invalid call to transfer_from (Alice is token owner, not Frank)
            assert_eq!(
                erc721.transfer_from(accounts.frank, accounts.bob, 1),
                Err(Error::NotOwner)
            );
        }

        #[ink::test]
        fn transfer_fails_not_owner() {
            let accounts = ink::env::test::default_accounts();
            set_caller(accounts.alice);
            // Create a new contract instance.
            let mut erc721 = Erc721::new();
            // Create token Id 1 for Alice
            assert_eq!(erc721.mint(1), Ok(()));
            // Bob can transfer alice's tokens
            assert_eq!(erc721.set_approval_for_all(accounts.bob, true), Ok(()));
            // Set caller to bob
            set_caller(accounts.bob);
            // Bob makes invalid call to transfer (he is not token owner, Alice is)
            assert_eq!(erc721.transfer(accounts.bob, 1), Err(Error::NotOwner));
        }

        fn set_caller(sender: Address) {
            ink::env::test::set_caller(sender);
        }
    }
}
