use ink_lang as ink;

#[ink::contract]
mod erc721 {
    #[cfg(not(feature = "ink-as-dependency"))]
    use ink_storage::collections::{
        hashmap::Entry,
        HashMap as StorageHashMap,
    };
    use scale::{
        Decode,
        Encode,
    };

    /// A token ID.
    pub type TokenId = u32;

    #[ink(storage)]
    #[derive(Default)]
    pub struct Erc721 {
        /// Mapping from token to owner.
        token_owner: StorageHashMap<TokenId, AccountId>,
        /// Mapping from token to approvals users.
        token_approvals: StorageHashMap<TokenId, AccountId>,
        /// Mapping from owner to number of owned token.
        owned_tokens_count: StorageHashMap<AccountId, u32>,
        /// Mapping from owner to operator approvals.
        operator_approvals: StorageHashMap<(AccountId, AccountId), bool>,
    }

    #[derive(Encode, Decode, Debug, PartialEq, Eq, Copy, Clone)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        NotOwner,
        NotApproved,
        TokenExists,
        TokenNotFound,
        CannotInsert,
        CannotRemove,
        CannotFetchValue,
        NotAllowed,
    }

    /// Event emitted when a token transfer occurs.
    #[ink(event)]
    pub struct Transfer {
        #[ink(topic)]
        from: Option<AccountId>,
        #[ink(topic)]
        to: Option<AccountId>,
        #[ink(topic)]
        id: TokenId,
    }

    /// Event emitted when a token approve occurs.
    #[ink(event)]
    pub struct Approval {
        #[ink(topic)]
        from: AccountId,
        #[ink(topic)]
        to: AccountId,
        #[ink(topic)]
        id: TokenId,
    }

    /// Event emitted when an operator is enabled or disabled for an owner.
    /// The operator can manage all NFTs of the owner.
    #[ink(event)]
    pub struct ApprovalForAll {
        #[ink(topic)]
        owner: AccountId,
        #[ink(topic)]
        operator: AccountId,
        approved: bool,
    }

    impl Erc721 {
        /// Creates a new ERC-721 token contract.
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                token_owner: Default::default(),
                token_approvals: Default::default(),
                owned_tokens_count: Default::default(),
                operator_approvals: Default::default(),
            }
        }

        /// Returns the balance of the owner.
        ///
        /// This represents the amount of unique tokens the owner has.
        #[ink(message)]
        pub fn balance_of(&self, owner: AccountId) -> u32 {
            self.balance_of_or_zero(&owner)
        }

        /// Returns the owner of the token.
        #[ink(message)]
        pub fn owner_of(&self, id: TokenId) -> Option<AccountId> {
            self.token_owner.get(&id).cloned()
        }

        /// Returns the approved account ID for this token if any.
        #[ink(message)]
        pub fn get_approved(&self, id: TokenId) -> Option<AccountId> {
            self.token_approvals.get(&id).cloned()
        }

        /// Returns `true` if the operator is approved by the owner.
        #[ink(message)]
        pub fn is_approved_for_all(&self, owner: AccountId, operator: AccountId) -> bool {
            self.approved_for_all(owner, operator)
        }

        /// Approves or disapproves the operator for all tokens of the caller.
        #[ink(message)]
        pub fn set_approval_for_all(
            &mut self,
            to: AccountId,
            approved: bool,
        ) -> Result<(), Error> {
            self.approve_for_all(to, approved)?;
            Ok(())
        }

        /// Approves the account to transfer the specified token on behalf of the caller.
        #[ink(message)]
        pub fn approve(&mut self, to: AccountId, id: TokenId) -> Result<(), Error> {
            self.approve_for(&to, id)?;
            Ok(())
        }

        /// Transfers the token from the caller to the given destination.
        #[ink(message)]
        pub fn transfer(
            &mut self,
            destination: AccountId,
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
            from: AccountId,
            to: AccountId,
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
                from: Some(AccountId::from([0x0; 32])),
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
            let occupied = match token_owner.entry(id) {
                Entry::Vacant(_) => return Err(Error::TokenNotFound),
                Entry::Occupied(occupied) => occupied,
            };
            if occupied.get() != &caller {
                return Err(Error::NotOwner)
            };
            decrease_counter_of(owned_tokens_count, &caller)?;
            occupied.remove_entry();
            self.env().emit_event(Transfer {
                from: Some(caller),
                to: Some(AccountId::from([0x0; 32])),
                id,
            });
            Ok(())
        }

        /// Transfers token `id` `from` the sender to the `to` `AccountId`.
        fn transfer_token_from(
            &mut self,
            from: &AccountId,
            to: &AccountId,
            id: TokenId,
        ) -> Result<(), Error> {
            let caller = self.env().caller();
            if !self.exists(id) {
                return Err(Error::TokenNotFound)
            };
            if !self.approved_or_owner(Some(caller), id) {
                return Err(Error::NotApproved)
            };
            self.clear_approval(id)?;
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
            from: &AccountId,
            id: TokenId,
        ) -> Result<(), Error> {
            let Self {
                token_owner,
                owned_tokens_count,
                ..
            } = self;
            let occupied = match token_owner.entry(id) {
                Entry::Vacant(_) => return Err(Error::TokenNotFound),
                Entry::Occupied(occupied) => occupied,
            };
            decrease_counter_of(owned_tokens_count, from)?;
            occupied.remove_entry();
            Ok(())
        }

        /// Adds the token `id` to the `to` AccountID.
        fn add_token_to(&mut self, to: &AccountId, id: TokenId) -> Result<(), Error> {
            let Self {
                token_owner,
                owned_tokens_count,
                ..
            } = self;
            let vacant_token_owner = match token_owner.entry(id) {
                Entry::Vacant(vacant) => vacant,
                Entry::Occupied(_) => return Err(Error::TokenExists),
            };
            if *to == AccountId::from([0x0; 32]) {
                return Err(Error::NotAllowed)
            };
            let entry = owned_tokens_count.entry(*to);
            increase_counter_of(entry)?;
            vacant_token_owner.insert(*to);
            Ok(())
        }

        /// Approves or disapproves the operator to transfer all tokens of the caller.
        fn approve_for_all(
            &mut self,
            to: AccountId,
            approved: bool,
        ) -> Result<(), Error> {
            let caller = self.env().caller();
            if to == caller {
                return Err(Error::NotAllowed)
            }
            self.env().emit_event(ApprovalForAll {
                owner: caller,
                operator: to,
                approved,
            });
            if self.approved_for_all(caller, to) {
                let status = self
                    .operator_approvals
                    .get_mut(&(caller, to))
                    .ok_or(Error::CannotFetchValue)?;
                *status = approved;
                Ok(())
            } else {
                match self.operator_approvals.insert((caller, to), approved) {
                    Some(_) => Err(Error::CannotInsert),
                    None => Ok(()),
                }
            }
        }

        /// Approve the passed AccountId to transfer the specified token on behalf of the message's sender.
        fn approve_for(&mut self, to: &AccountId, id: TokenId) -> Result<(), Error> {
            let caller = self.env().caller();
            let owner = self.owner_of(id);
            if !(owner == Some(caller)
                || self.approved_for_all(owner.expect("Error with AccountId"), caller))
            {
                return Err(Error::NotAllowed)
            };
            if *to == AccountId::from([0x0; 32]) {
                return Err(Error::NotAllowed)
            };

            if self.token_approvals.insert(id, *to).is_some() {
                return Err(Error::CannotInsert)
            };
            self.env().emit_event(Approval {
                from: caller,
                to: *to,
                id,
            });
            Ok(())
        }

        /// Removes existing approval from token `id`.
        fn clear_approval(&mut self, id: TokenId) -> Result<(), Error> {
            if !self.token_approvals.contains_key(&id) {
                return Ok(())
            };
            match self.token_approvals.take(&id) {
                Some(_res) => Ok(()),
                None => Err(Error::CannotRemove),
            }
        }

        // Returns the total number of tokens from an account.
        fn balance_of_or_zero(&self, of: &AccountId) -> u32 {
            *self.owned_tokens_count.get(of).unwrap_or(&0)
        }

        /// Gets an operator on other Account's behalf.
        fn approved_for_all(&self, owner: AccountId, operator: AccountId) -> bool {
            *self
                .operator_approvals
                .get(&(owner, operator))
                .unwrap_or(&false)
        }

        /// Returns true if the `AccountId` `from` is the owner of token `id`
        /// or it has been approved on behalf of the token `id` owner.
        fn approved_or_owner(&self, from: Option<AccountId>, id: TokenId) -> bool {
            let owner = self.owner_of(id);
            from != Some(AccountId::from([0x0; 32]))
                && (from == owner
                    || from == self.token_approvals.get(&id).cloned()
                    || self.approved_for_all(
                        owner.expect("Error with AccountId"),
                        from.expect("Error with AccountId"),
                    ))
        }

        /// Returns true if token `id` exists or false if it does not.
        fn exists(&self, id: TokenId) -> bool {
            self.token_owner.get(&id).is_some() && self.token_owner.contains_key(&id)
        }
    }

    fn decrease_counter_of(
        hmap: &mut StorageHashMap<AccountId, u32>,
        of: &AccountId,
    ) -> Result<(), Error> {
        let count = (*hmap).get_mut(of).ok_or(Error::CannotFetchValue)?;
        *count -= 1;
        Ok(())
    }

    /// Increase token counter from the `of` AccountId.
    fn increase_counter_of(entry: Entry<AccountId, u32>) -> Result<(), Error> {
        entry.and_modify(|v| *v += 1).or_insert(1);
        Ok(())
    }

    /// Unit tests
    #[cfg(test)]
    mod tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;
        use ink_env::{
            call,
            test,
        };
        use ink_lang as ink;

        #[ink::test]
        fn mint_works() {
            let accounts =
                ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
                    .expect("Cannot get accounts");
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
            let accounts =
                ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
                    .expect("Cannot get accounts");
            // Create a new contract instance.
            let mut erc721 = Erc721::new();
            // Create token Id 1.
            assert_eq!(erc721.mint(1), Ok(()));
            // The first Transfer event takes place
            assert_eq!(1, ink_env::test::recorded_events().count());
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
            let accounts =
                ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
                    .expect("Cannot get accounts");
            // Create a new contract instance.
            let mut erc721 = Erc721::new();
            // Create token Id 1 for Alice
            assert_eq!(erc721.mint(1), Ok(()));
            // Alice owns token 1
            assert_eq!(erc721.balance_of(accounts.alice), 1);
            // Bob does not owns any token
            assert_eq!(erc721.balance_of(accounts.bob), 0);
            // The first Transfer event takes place
            assert_eq!(1, ink_env::test::recorded_events().count());
            // Alice transfers token 1 to Bob
            assert_eq!(erc721.transfer(accounts.bob, 1), Ok(()));
            // The second Transfer event takes place
            assert_eq!(2, ink_env::test::recorded_events().count());
            // Bob owns token 1
            assert_eq!(erc721.balance_of(accounts.bob), 1);
        }

        #[ink::test]
        fn invalid_transfer_should_fail() {
            let accounts =
                ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
                    .expect("Cannot get accounts");
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
            // Get contract address
            let callee = ink_env::account_id::<ink_env::DefaultEnvironment>()
                .unwrap_or([0x0; 32].into());
            // Create call
            let mut data =
                ink_env::test::CallData::new(ink_env::call::Selector::new([0x00; 4])); // balance_of
            data.push_arg(&accounts.bob);
            // Push the new execution context to set Bob as caller
            ink_env::test::push_execution_context::<ink_env::DefaultEnvironment>(
                accounts.bob,
                callee,
                1000000,
                1000000,
                data,
            );
            // Bob cannot transfer not owned tokens.
            assert_eq!(erc721.transfer(accounts.eve, 2), Err(Error::NotApproved));
        }

        #[ink::test]
        fn approved_transfer_works() {
            let accounts =
                ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
                    .expect("Cannot get accounts");
            // Create a new contract instance.
            let mut erc721 = Erc721::new();
            // Create token Id 1.
            assert_eq!(erc721.mint(1), Ok(()));
            // Token Id 1 is owned by Alice.
            assert_eq!(erc721.owner_of(1), Some(accounts.alice));
            // Approve token Id 1 transfer for Bob on behalf of Alice.
            assert_eq!(erc721.approve(accounts.bob, 1), Ok(()));
            // Get contract address.
            let callee = ink_env::account_id::<ink_env::DefaultEnvironment>()
                .unwrap_or([0x0; 32].into());
            // Create call
            let mut data =
                ink_env::test::CallData::new(ink_env::call::Selector::new([0x00; 4])); // balance_of
            data.push_arg(&accounts.bob);
            // Push the new execution context to set Bob as caller
            ink_env::test::push_execution_context::<ink_env::DefaultEnvironment>(
                accounts.bob,
                callee,
                1000000,
                1000000,
                data,
            );
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
            let accounts =
                ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
                    .expect("Cannot get accounts");
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
            assert_eq!(
                erc721.is_approved_for_all(accounts.alice, accounts.bob),
                true
            );
            // Get contract address.
            let callee = ink_env::account_id::<ink_env::DefaultEnvironment>()
                .unwrap_or([0x0; 32].into());
            // Create call
            let mut data =
                ink_env::test::CallData::new(ink_env::call::Selector::new([0x00; 4])); // balance_of
            data.push_arg(&accounts.bob);
            // Push the new execution context to set Bob as caller
            ink_env::test::push_execution_context::<ink_env::DefaultEnvironment>(
                accounts.bob,
                callee,
                1000000,
                1000000,
                data,
            );
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
            // Bob does not owns tokens.
            assert_eq!(erc721.balance_of(accounts.bob), 0);
            // Eve owns 2 tokens.
            assert_eq!(erc721.balance_of(accounts.eve), 2);
            // Get back to the parent execution context.
            ink_env::test::pop_execution_context();
            // Remove operator approval for Bob on behalf of Alice.
            assert_eq!(erc721.set_approval_for_all(accounts.bob, false), Ok(()));
            // Bob is not an approved operator for Alice.
            assert_eq!(
                erc721.is_approved_for_all(accounts.alice, accounts.bob),
                false
            );
        }

        #[ink::test]
        fn not_approved_transfer_should_fail() {
            let accounts =
                ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
                    .expect("Cannot get accounts");
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
            // Get contract address.
            let callee = ink_env::account_id::<ink_env::DefaultEnvironment>()
                .unwrap_or([0x0; 32].into());
            // Create call
            let mut data =
                ink_env::test::CallData::new(ink_env::call::Selector::new([0x00; 4])); // balance_of
            data.push_arg(&accounts.bob);
            // Push the new execution context to set Eve as caller
            ink_env::test::push_execution_context::<ink_env::DefaultEnvironment>(
                accounts.eve,
                callee,
                1000000,
                1000000,
                data,
            );
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
            let accounts =
                ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
                    .expect("Cannot get accounts");
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
            let accounts =
                ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
                    .expect("Cannot get accounts");
            // Create a new contract instance.
            let mut erc721 = Erc721::new();
            // Create token Id 1 for Alice
            assert_eq!(erc721.mint(1), Ok(()));
            // Try burning this token with a different account
            set_sender(accounts.eve);
            assert_eq!(erc721.burn(1), Err(Error::NotOwner));
        }

        fn set_sender(sender: AccountId) {
            let callee = ink_env::account_id::<ink_env::DefaultEnvironment>()
                .unwrap_or([0x0; 32].into());
            test::push_execution_context::<Environment>(
                sender,
                callee,
                1000000,
                1000000,
                test::CallData::new(call::Selector::new([0x00; 4])), // dummy
            );
        }
    }
}

fn main() {}
