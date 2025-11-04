use crate::sr25519::Keypair;
use sp_core::crypto::AccountId32;

/// Trait for types that can be converted into an `AccountId`.
pub trait IntoAccountId<TargetAccountId> {
    fn into_account_id(self) -> TargetAccountId;
}

impl IntoAccountId<AccountId32> for AccountId32 {
    fn into_account_id(self) -> AccountId32 {
        self
    }
}

impl IntoAccountId<AccountId32> for &AccountId32 {
    fn into_account_id(self) -> AccountId32 {
        self.clone()
    }
}

impl<AccountId> IntoAccountId<AccountId> for &ink_primitives::AccountId
where
    AccountId: From<[u8; 32]>,
{
    fn into_account_id(self) -> AccountId {
        AccountId::from(*AsRef::<[u8; 32]>::as_ref(self))
    }
}

impl<AccountId> IntoAccountId<AccountId> for &Keypair
where
    AccountId: From<[u8; 32]>,
{
    fn into_account_id(self) -> AccountId {
        AccountId::from(self.public_key().0)
    }
}
