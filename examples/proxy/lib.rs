#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
pub mod proxy {
    /// A simple proxy contract, enabling upgradable smart contracts.
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

        /// Forward everything to the next contract
        #[ink(message)]
        pub fn upgrade(&mut self, new_forward_addr: AccountId) {
            assert_eq!(self.env().caller(), self.admin);
            self.forward_to = new_forward_addr;
        }

        /// Fallback message for a contract call that doesn't match any
        /// of the other message selectors.
        ///
        /// *Note:*
        /// - We allow payable messages here and would forward any optionally supplied
        ///   value as well.
        /// - If the self receiver would be `forward(&mut self)` here, this would not
        ///   imply that the contract to which we forward to does not mutate it's own
        ///   state.
        #[ink(message, payable, selector = "_")]
        pub fn forward(&self) -> u32 {
            ink_env::call::build_call::<ink_env::DefaultEnvironment>()
                .callee(self.forward_to)
                .call_flags(
                    ink_env::CallFlags::default()
                        .set_clone_input(true)
                        .set_tail_call(true),
                )
                .gas_limit(0)
                .transferred_value(self.env().transferred_value())
                .fire()
                .expect("cross-contract call failed");
            unreachable!(
                "the forwarded call will never return since `tail_call` was set"
            );
        }
    }
}
