//! This example demonstrates how the Proxy/Forward pattern can be
//! implemented in ink!.
//!
//! What the contract does is:
//!
//!   * Any call to this contract that does not match a selector
//!     of it is forwarded to a specified address.
//!   * The instantiator of the contract can modify this specified
//!     `forward_to` address at any point.
//!
//! Using this pattern it is possible to implement upgradable contracts.

#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
pub mod proxy {
    use ink_env::call::DelegateCall;
    use ink_primitives::{
        Key,
        KeyPtr,
    };
    use ink_storage::traits::SpreadLayout;

    /// This struct contains the data related to the Proxy storage.
    ///
    /// The reason this is a separate structure is that we want to keep
    /// the data for this contract in a separate place (as in the implementation
    /// of [`SpreadLayout`]), so that it does not get overwritten by any contract
    /// upgrade, which might introduce storage changes.
    #[derive(Debug)]
    #[cfg_attr(feature = "std", derive(ink_storage::traits::StorageLayout))]
    struct ProxyFields {
        /// The `Hash` of a contract code where any call that does not match a
        /// selector of this contract is forwarded to.
        delegate_to: Hash,
        /// The `AccountId` of a privileged account that can update the
        /// forwarding address. This address is set to the account that
        /// instantiated this contract.
        admin: AccountId,
    }

    const PROXY_FIELDS_STORAGE_KEY: [u8; 32] = ink_lang::blake2x256!("ProxyFields");

    impl SpreadLayout for ProxyFields {
        const FOOTPRINT: u64 =
            <AccountId as SpreadLayout>::FOOTPRINT + <Hash as SpreadLayout>::FOOTPRINT;
        const REQUIRES_DEEP_CLEAN_UP: bool = false;

        fn pull_spread(_: &mut KeyPtr) -> Self {
            let mut ptr = KeyPtr::from(Key::from(PROXY_FIELDS_STORAGE_KEY));
            Self {
                delegate_to: SpreadLayout::pull_spread(&mut ptr),
                admin: SpreadLayout::pull_spread(&mut ptr),
            }
        }

        fn push_spread(&self, _: &mut KeyPtr) {
            let mut ptr = KeyPtr::from(Key::from(PROXY_FIELDS_STORAGE_KEY));
            SpreadLayout::push_spread(&self.delegate_to, &mut ptr);
            SpreadLayout::push_spread(&self.admin, &mut ptr);
        }

        fn clear_spread(&self, _: &mut KeyPtr) {
            let mut ptr = KeyPtr::from(Key::from(PROXY_FIELDS_STORAGE_KEY));
            SpreadLayout::clear_spread(&self.delegate_to, &mut ptr);
            SpreadLayout::clear_spread(&self.admin, &mut ptr);
        }
    }

    /// A simple proxy contract.
    #[ink(storage)]
    pub struct Proxy {
        proxy: ProxyFields,
    }

    impl Proxy {
        /// Instantiate this contract with an address of the `logic` contract.
        ///
        /// Sets the privileged account to the caller. Only this account may
        /// later changed the `forward_to` address.
        #[ink(constructor)]
        pub fn new(delegate_to: Hash) -> Self {
            Self {
                proxy: ProxyFields {
                    delegate_to,
                    admin: Self::env().caller(),
                },
            }
        }

        /// Changes the `Hash` of the contract where any call that does
        /// not match a selector of this contract is delegated to.
        #[ink(message)]
        pub fn change_delegate_code(&mut self, new_code_hash: Hash) {
            assert_eq!(
                self.env().caller(),
                self.proxy.admin,
                "caller {:?} does not have sufficient permissions, only {:?} does",
                self.env().caller(),
                self.proxy.admin,
            );
            self.proxy.delegate_to = new_code_hash;
        }

        /// Fallback message for a contract call that doesn't match any
        /// of the other message selectors.
        ///
        /// # Note:
        ///
        /// - We allow payable messages here and would forward any optionally supplied
        ///   value as well.
        /// - If the self receiver were `forward(&mut self)` here, this would not
        ///   have any effect whatsoever on the contract we forward to.
        #[ink(message, payable, selector = _)]
        pub fn forward(&self) -> u32 {
            ink_env::call::build_call::<ink_env::DefaultEnvironment>()
                .set_call_type(DelegateCall::new().set_code_hash(self.proxy.delegate_to))
                .call_flags(
                    ink_env::CallFlags::default()
                        .set_forward_input(true)
                        .set_tail_call(true),
                )
                .fire()
                .unwrap_or_else(|err| {
                    panic!(
                        "delegate call to {:?} failed due to {:?}",
                        self.proxy.delegate_to, err
                    )
                });
            unreachable!(
                "the forwarded call will never return since `tail_call` was set"
            );
        }
    }
}
