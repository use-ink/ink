//! This example demonstrates how the Proxy pattern can be
//! implemented in ink! to have upgradeable functionality.
//!
//! What the contract does is:
//!
//!   * Any call to this contract that does not match a selector
//!     of it is delegates to a specified address.
//!   * The instantiator of the contract can modify this specified
//!     `forward_to` address at any point.
//!
//!   User ---- tx ---> Proxy ---------> Implementation_v0
//!                      | ------------> Implementation_v1
//!                      | ------------> Implementation_v2

#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
pub mod upgradeable_contract {
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
    /// of [`SpreadLayout`](ink_storage::traits::SpreadLayout)), so that it does not get
    /// overwritten by any contract upgrade, which might introduce storage changes.
    #[derive(Debug)]
    #[cfg_attr(feature = "std", derive(ink_storage::traits::StorageLayout))]
    struct ProxyFields {
        /// The `Hash` of a contract code where any call that does not match a
        /// selector of this contract is forward to.
        forward_to: Hash,
        /// The `AccountId` of a privileged account that can update the
        /// forwarding address. This address is set to the account that
        /// instantiated this contract.
        admin: AccountId,
    }

    const PROXY_FIELDS_STORAGE_KEY: [u8; 32] = ink_lang::blake2x256!("ProxyFields");

    /// `SpreadLayout` is implemented manually to use its own `PROXY_FIELDS_STORAGE_KEY`
    /// storage key instead of the default contract storage `ContractRootKey::ROOT_KEY`.
    ///
    /// This allows us to store the proxy contract's storage in such a way that it will not
    /// conflict with the the default storage layout of the contract we're proxying calls to.
    impl SpreadLayout for ProxyFields {
        const FOOTPRINT: u64 =
            <AccountId as SpreadLayout>::FOOTPRINT + <Hash as SpreadLayout>::FOOTPRINT;

        fn pull_spread(_: &mut KeyPtr) -> Self {
            let mut ptr = KeyPtr::from(Key::from(PROXY_FIELDS_STORAGE_KEY));
            Self {
                forward_to: SpreadLayout::pull_spread(&mut ptr),
                admin: SpreadLayout::pull_spread(&mut ptr),
            }
        }

        fn push_spread(&self, _: &mut KeyPtr) {
            let mut ptr = KeyPtr::from(Key::from(PROXY_FIELDS_STORAGE_KEY));
            SpreadLayout::push_spread(&self.forward_to, &mut ptr);
            SpreadLayout::push_spread(&self.admin, &mut ptr);
        }

        fn clear_spread(&self, _: &mut KeyPtr) {
            let mut ptr = KeyPtr::from(Key::from(PROXY_FIELDS_STORAGE_KEY));
            SpreadLayout::clear_spread(&self.forward_to, &mut ptr);
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
        pub fn new(forward_to: Hash) -> Self {
            Self {
                proxy: ProxyFields {
                    forward_to,
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
            self.proxy.forward_to = new_code_hash;
        }

        /// Fallback message for a contract call that doesn't match any
        /// of the other message selectors. Proxy contract delegates the execution
        /// of that message to the `forward_to` contract with all input data.
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
                .call_type(DelegateCall::new().code_hash(self.proxy.forward_to))
                .call_flags(
                    ink_env::CallFlags::default()
                        // We don't plan to use the input data after the delegated call, so the 
                        // input data can be forwarded to delegated contract to reduce the gas usage.
                        .set_forward_input(true)
                        // We don't plan to return back to that contract after execution, so we 
                        // marked delegated call as "tail", to end the execution of the contract.
                        .set_tail_call(true),
                )
                .fire()
                .unwrap_or_else(|err| {
                    panic!(
                        "delegate call to {:?} failed due to {:?}",
                        self.proxy.forward_to, err
                    )
                });
            unreachable!(
                "the forwarded call will never return since `tail_call` was set"
            );
        }
    }
}
