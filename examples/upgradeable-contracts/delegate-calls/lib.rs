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
        StorageKey,
        StorageKeyComposer,
    };
    use ink_storage::traits::{
        ManualKey,
        StorageKeyHolder,
    };

    const PROXY_STORAGE_KEY: StorageKey = StorageKeyComposer::from_str("ProxyFields");

    /// A simple proxy contract.
    ///
    /// The proxy contracts is stored in own storage cell under the `PROXY_STORAGE_KEY`
    /// instead of the default contract storage key = `0`.
    ///
    /// This allows us to store the proxy contract's storage in such a way that it will not
    /// conflict with the the default storage layout of the contract we're proxying calls to.
    #[ink(storage)]
    pub struct Proxy<KEY: StorageKeyHolder = ManualKey<PROXY_STORAGE_KEY>> {
        /// The `Hash` of a contract code where any call that does not match a
        /// selector of this contract is forward to.
        forward_to: Hash,
        /// The `AccountId` of a privileged account that can update the
        /// forwarding address. This address is set to the account that
        /// instantiated this contract.
        admin: AccountId,
    }

    impl Proxy {
        /// Instantiate this contract with an address of the `logic` contract.
        ///
        /// Sets the privileged account to the caller. Only this account may
        /// later changed the `forward_to` address.
        #[ink(constructor)]
        pub fn new(forward_to: Hash) -> Self {
            Self {
                forward_to,
                admin: Self::env().caller(),
            }
        }

        /// Changes the `Hash` of the contract where any call that does
        /// not match a selector of this contract is delegated to.
        #[ink(message)]
        pub fn change_delegate_code(&mut self, new_code_hash: Hash) {
            assert_eq!(
                self.env().caller(),
                self.admin,
                "caller {:?} does not have sufficient permissions, only {:?} does",
                self.env().caller(),
                self.admin,
            );
            self.forward_to = new_code_hash;
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
                .call_type(DelegateCall::new().code_hash(self.forward_to))
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
                        self.forward_to, err
                    )
                });
            unreachable!(
                "the forwarded call will never return since `tail_call` was set"
            );
        }
    }
}
