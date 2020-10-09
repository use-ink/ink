use ink_lang as ink;

#[ink::contract]
mod erc721 {
    use ink_storage::collections::HashMap as StorageHashMap;

    /// A token ID.
    pub type TokenId = u32;

    #[derive(scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        SpecifiedTokenHasNoOwner,
        ApprovalToCurrentOwner,
        ApproveCallerNotLegitimate,
        ApprovedQueryForNonexistentToken,
        ApproveToCaller,
        TransferCallerIsNotOwnerOrApproved,
        OperatorQueryForNonexistentToken,
        TransferOfTokenThatIsNotOwned,
        TokenAlreadyMinted,
        CannotBurnNonexistentToken,
    }

    pub type Result<T> = core::result::Result<T, Error>;

    /// The storage items for a typical ERC721 token implementation.
    #[ink(storage)]
    #[derive(Default)]
    pub struct Erc721 {
        /// Stores one owner for every token.
        token_owner: StorageHashMap<TokenId, AccountId>,
        /// Mapping from token ID to approved address.
        token_approvals: StorageHashMap<TokenId, AccountId>,
        /// Mapping from owner to number of owned tokens.
        owned_tokens_count: StorageHashMap<AccountId, u32>,
        /// Mapping from owner to operator approval.
        operator_approvals: StorageHashMap<(AccountId, AccountId), bool>,
    }

    /// Notifies about token approvals.
    #[ink(event)]
    pub struct Approval {
        /// The owner of the token.
        owner: AccountId,
        /// The approved account.
        to: AccountId,
        /// The approved token.
        token: TokenId,
    }

    /// Notifies about approval for all tokens.
    #[ink(event)]
    pub struct ApprovalForAll {
        /// The source.
        from: AccountId,
        /// The destination.
        to: AccountId,
        /// If it was approved.
        approved: bool,
    }

    /// Notifies about token transfers.
    #[ink(event)]
    pub struct Transfer {
        /// The source of the transfered token.
        from: Option<AccountId>,
        /// The destination of the transfered token.
        to: Option<AccountId>,
        /// The transfered token.
        token: TokenId,
    }

    impl Erc721 {
        /// Nothing to do for initialization.
        #[ink(constructor)]
        pub fn new() -> Self {
            Default::default()
        }

        /// Returns the balance of the specified address.
        ///
        /// # Note
        ///
        /// The returned amount represents the number of owned tokens by the address.
        #[ink(message)]
        pub fn balance_of(&self, owner: AccountId) -> u32 {
            *self.owned_tokens_count.get(&owner).unwrap_or(&0)
        }

        /// Returns the owner of the specified token ID if any.
        #[ink(message)]
        pub fn owner_of(&self, token: TokenId) -> Option<AccountId> {
            self.token_owner.get(&token).cloned()
        }

        /// Approves another address to transfer the given token ID.
        ///
        /// There can only be one approved address per token at a given time.
        /// Can only be called by the token owner or an approved operator.
        #[ink(message)]
        pub fn approve(&mut self, to: AccountId, token: TokenId) -> Result<()> {
            let owner = self
                .owner_of(token)
                .ok_or(Error::SpecifiedTokenHasNoOwner)?;
            if to == owner {
                return Err(Error::ApprovalToCurrentOwner)
            }
            let caller = self.env().caller();
            if caller == owner || self.is_approved_for_all(owner, caller) {
                return Err(Error::ApproveCallerNotLegitimate)
            }
            self.token_approvals.insert(token, to);
            self.env().emit_event(Approval { owner, to, token });
            Ok(())
        }

        /// Returns the approved address for the token ID if any.
        ///
        /// Reverts if the token ID does not exist.
        #[ink(message)]
        pub fn get_approved(&self, token: TokenId) -> Result<AccountId> {
            self.token_owner
                .get(&token)
                .ok_or(Error::ApprovedQueryForNonexistentToken)
                .map(Clone::clone)
        }

        /// Sets of unsets the approval of a given operator.
        ///
        /// An operator is allowed to transfer all tokens of the sender on their behalf.
        #[ink(message)]
        pub fn set_approval_for_all(
            &mut self,
            to: AccountId,
            approved: bool,
        ) -> Result<()> {
            let caller = self.env().caller();
            if to == caller {
                return Err(Error::ApproveToCaller)
            }
            self.operator_approvals.insert((caller, to), approved);
            self.env().emit_event(ApprovalForAll {
                from: caller,
                to,
                approved,
            });
            Ok(())
        }

        /// Returns `true` if an operator is approved by a given owner.
        #[ink(message)]
        pub fn is_approved_for_all(&self, owner: AccountId, operator: AccountId) -> bool {
            *self
                .operator_approvals
                .get(&(owner, operator))
                .unwrap_or(&false)
        }

        /// Transfers the ownership of a given token ID to another address.
        ///
        /// # Note
        ///
        /// Usage of this method is discouraged, use `safe_transfer_from` whenever possible.
        ///
        /// # Errors
        ///
        /// If the caller is not the owner, approved or operator.
        #[ink(message)]
        pub fn transfer_from(
            &mut self,
            from: AccountId,
            to: AccountId,
            token: TokenId,
        ) -> Result<()> {
            let caller = self.env().caller();
            if !self.is_approved_or_owner(&caller, token) {
                return Err(Error::TransferCallerIsNotOwnerOrApproved)
            }
            self.transfer_from_impl(from, to, token)?;
            Ok(())
        }

        /// Returns `true` if the given spender can transfer the given token.
        fn is_approved_or_owner(&self, spender: &AccountId, token: TokenId) -> bool {
            self.token_owner
                .get(&token)
                .ok_or(Error::OperatorQueryForNonexistentToken)
                .map(|&owner| {
                    let approved = self.get_approved(token).unwrap_or(owner);
                    *spender == owner
                        || approved == *spender
                        || self.is_approved_for_all(owner, *spender)
                })
                .unwrap_or(false)
        }

        /// Transfers ownership of the token to another address.
        ///
        /// # Safety
        ///
        /// As opposed to `transfer_from` this imposes no restructions on the `caller`.
        fn transfer_from_impl(
            &mut self,
            from: AccountId,
            to: AccountId,
            token: TokenId,
        ) -> Result<()> {
            if self.owner_of(token).unwrap_or(from) != from {
                return Err(Error::TransferOfTokenThatIsNotOwned)
            }
            self.clear_approval(token);
            self.owned_tokens_count[&from] -= 1; // TODO: are these calls safe here?
            self.owned_tokens_count[&to] += 1;
            self.token_owner[&token] = to;
            self.env().emit_event(Transfer {
                from: Some(from),
                to: Some(to),
                token,
            });
            Ok(())
        }

        /// Clears the current approval of a given token.
        fn clear_approval(&mut self, token: TokenId) {
            self.token_approvals.take(&token);
        }

        /// Mints a new token.
        fn mint(&mut self, to: AccountId, token: TokenId) -> Result<()> {
            let _ = self
                .token_owner
                .get(&token)
                .ok_or(Error::TokenAlreadyMinted)?;
            self.token_owner[&token] = to;
            self.owned_tokens_count[&to] += 1;
            self.env().emit_event(Transfer {
                from: None,
                to: Some(to),
                token,
            });
            Ok(())
        }

        // Burns the token.
        fn burn(&mut self, token: TokenId) -> Result<()> {
            let owner = *self
                .token_owner
                .get(&token)
                .ok_or(Error::CannotBurnNonexistentToken)?;
            self.clear_approval(token);
            self.owned_tokens_count[&owner] -= 1;
            self.token_owner.take(&token);
            self.env().emit_event(Transfer {
                from: Some(owner),
                to: None,
                token,
            });
            Ok(())
        }
    }
}

fn main() {}
