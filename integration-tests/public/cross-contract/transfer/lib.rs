//! A smart contract which demonstrates behavior of the `self.env().transfer()` function.
//! It transfers some of it's balance to the caller.

#![cfg_attr(not(feature = "std"), no_std, no_main)]
#![allow(clippy::new_without_default)]

#[ink::contract]
pub mod give_me {
    use ink::primitives::U256;

    /// No storage is needed for this simple contract.
    #[ink(storage)]
    pub struct GiveMe {}

    impl GiveMe {
        /// Creates a new instance of this contract.
        ///
        /// This is a payable constructor, meaning it can receive initial funding.
        #[ink(constructor, payable)]
        pub fn new() -> Self {
            Self {}
        }

        /// Transfers `value` amount of tokens to the caller.
        ///
        /// # Errors
        ///
        /// - Panics in case the requested transfer exceeds the contract balance.
        /// - Panics in case the requested transfer would have brought this contract's
        ///   balance below the minimum balance (i.e. the chain's existential deposit).
        /// - Panics in case the transfer failed for another reason.
        #[ink(message)]
        pub fn give_me(&mut self, value: U256) {
            assert!(value <= self.env().balance(), "insufficient funds!");

            if self.env().transfer(self.env().caller(), value).is_err() {
                panic!(
                    "requested transfer failed. this can be the case if the contract does not\
                     have sufficient free funds or if the transfer would have brought the\
                     contract's balance below minimum balance."
                )
            }
        }

        /// Asserts that the token amount sent as payment with this call
        /// is exactly `10`. This method will fail otherwise, and the
        /// transaction would then be reverted.
        ///
        /// # Note
        ///
        /// The method needs to be annotated with `payable`; only then it is
        /// allowed to receive value as part of the call.
        #[ink(message, payable, selector = 0xCAFEBABE)]
        pub fn was_it_ten(&mut self) {
            /*
            ink::env::debug_println!(
                "received payment: {}",
                self.env().transferred_value()
            );
            */
            assert!(
                self.env().transferred_value() == U256::from(10),
                "payment was not ten"
            );
        }
    }
}

// Include the test file
#[cfg(test)]
mod tests;