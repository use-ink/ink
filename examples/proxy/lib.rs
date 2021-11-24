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
//!
//! Note though that the contract to which calls are forwarded still
//! contains it's own state.

#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
pub mod proxy {
    /// A simple proxy contract.
    #[ink(storage)]
    pub struct Proxy {
        /// The `AccountId` of a contract where any call that does not match a
        /// selector of this contract is forwarded to.
        forward_to: AccountId,
        /// The `AccountId` of a privileged account that can update the
        /// forwarding address. This address is set to the account that
        /// instantiated this contract.
        admin: AccountId,
    }

    /// Error which is returned if a caller tries to invoke
    /// [`Proxy::change_forward_address`] while not being the
    /// admin account stored in the storage of this contract.
    #[derive(scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct NotAdminErr {
        admin: AccountId,
    }

    impl core::fmt::Display for NotAdminErr {
        fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
            let msg = ink_env::format!(
                "caller does not have sufficient permissions, only {:?} does",
                self.admin
            );
            write!(f, "{}", msg)
        }
    }

    impl Proxy {
        /// Instantiate this contract with an address of the `logic` contract.
        ///
        /// Sets the privileged account to the caller. Only this account may
        /// later changed the `forward_to` address.
        #[ink(constructor)]
        pub fn new(forward_to: AccountId) -> Self {
            Self {
                admin: Self::env().caller(),
                forward_to,
            }
        }

        /// Changes the `AccountId` of the contract where any call that does
        /// not match a selector of this contract is forwarded to.
        ///
        /// Returns a [`NotAdminErr`] in case the caller is not the
        /// admin account stored in the storage of this contract.
        ///
        /// # Note:
        ///
        /// Returning an `Result::Err` here will result in the transaction
        /// being reverted, in that sense it is equal to e.g. placing an
        /// `assert!` here instead.
        #[ink(message)]
        pub fn change_forward_address(
            &mut self,
            new_address: AccountId,
        ) -> Result<(), NotAdminErr> {
            if self.env().caller() != self.admin {
                return Err(NotAdminErr { admin: self.admin })
            }
            self.forward_to = new_address;
            Ok(())
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
                .callee(self.forward_to)
                .call_flags(
                    ink_env::CallFlags::default()
                        .set_forward_input(true)
                        .set_tail_call(true),
                )
                .transferred_value(self.env().transferred_balance())
                .fire()
                .unwrap_or_else(|err| {
                    panic!(
                        "cross-contract call to {:?} failed due to {:?}",
                        self.forward_to, err
                    )
                });
            unreachable!(
                "the forwarded call will never return since `tail_call` was set"
            );
        }
    }
}
