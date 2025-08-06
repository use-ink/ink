#![allow(unexpected_cfgs)]

#[ink::contract]
mod erc721 {
    use ink_storage::Mapping;

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

    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
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
            Self::default()
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
            self.token_owner.get(&id)
        }

        /// Returns the approved account ID for this token if any.
        #[ink(message)]
        pub fn get_approved(&self, id: TokenId) -> Option<Address> {
            self.token_approvals.get(&id)
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
        pub fn transfer(&mut self, destination: Address, id: TokenId) -> Result<(), Error> {
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

            let owner = token_owner.get(&id).ok_or(Error::TokenNotFound)?;
            if owner != caller {
                return Err(Error::NotOwner)
            };

            let count = owned_tokens_count
                .get(&caller)
                .map(|c| c - 1)
                .ok_or(Error::CannotFetchValue)?;
            owned_tokens_count.insert(&caller, &count);
            token_owner.remove(&id);

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
            if !self.exists(id) {
                return Err(Error::TokenNotFound)
            };
            if !self.approved_or_owner(Some(caller), id) {
                return Err(Error::NotApproved)
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
        fn remove_token_from(&mut self, from: &Address, id: TokenId) -> Result<(), Error> {
            let Self {
                token_owner,
                owned_tokens_count,
                ..
            } = self;

            if token_owner.get(&id).is_none() {
                return Err(Error::TokenNotFound)
            }

            let count = owned_tokens_count
                .get(&from)
                .map(|c| c - 1)
                .ok_or(Error::CannotFetchValue)?;
            owned_tokens_count.insert(&from, &count);
            token_owner.remove(&id);

            Ok(())
        }

        /// Adds the token `id` to the `to` AccountID.
        fn add_token_to(&mut self, to: &Address, id: TokenId) -> Result<(), Error> {
            let Self {
                token_owner,
                owned_tokens_count,
                ..
            } = self;

            if token_owner.get(&id).is_some() {
                return Err(Error::TokenExists)
            }

            if *to == Address::from([0x0; 20]) {
                return Err(Error::NotAllowed)
            };

            let count = owned_tokens_count.get(to).map(|c| c + 1).unwrap_or(1);

            owned_tokens_count.insert(to, &count);
            token_owner.insert(&id, to);

            Ok(())
        }

        /// Approves or disapproves the operator to transfer all tokens of the caller.
        fn approve_for_all(&mut self, to: Address, approved: bool) -> Result<(), Error> {
            let caller = self.env().caller();
            if to == caller {
                return Err(Error::NotAllowed)
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
            let owner = self.owner_of(id);
            if !(owner == Some(caller)
                || self.approved_for_all(owner.expect("Error with Address"), caller))
            {
                return Err(Error::NotAllowed)
            };

            if *to == Address::from([0x0; 20]) {
                return Err(Error::NotAllowed)
            };

            if self.token_approvals.get(&id).is_some() {
                return Err(Error::CannotInsert)
            } else {
                self.token_approvals.insert(&id, to);
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
            self.token_approvals.remove(&id);
        }

        // Returns the total number of tokens from an account.
        fn balance_of_or_zero(&self, of: &Address) -> u32 {
            self.owned_tokens_count.get(of).unwrap_or(0)
        }

        /// Gets an operator on other Account's behalf.
        fn approved_for_all(&self, owner: Address, operator: Address) -> bool {
            self.operator_approvals.get((&owner, &operator)).is_some()
        }

        /// Returns true if the `Address` `from` is the owner of token `id`
        /// or it has been approved on behalf of the token `id` owner.
        fn approved_or_owner(&self, from: Option<Address>, id: TokenId) -> bool {
            let owner = self.owner_of(id);
            from != Some(Address::from([0x0; 20]))
                && (from == owner
                    || from == self.token_approvals.get(&id)
                    || self.approved_for_all(
                        owner.expect("Error with Address"),
                        from.expect("Error with Address"),
                    ))
        }

        /// Returns true if token `id` exists or false if it does not.
        fn exists(&self, id: TokenId) -> bool {
            self.token_owner.get(&id).is_some()
        }
    }
}

fn main() {}
