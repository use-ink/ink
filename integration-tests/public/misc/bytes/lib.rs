#![cfg_attr(not(feature = "std"), no_std, no_main)]

/// Example for using bytes wrapper types (i.e. `ink::sol::FixedBytes<N>` and
/// `ink::sol::DynBytes`) as message and event arguments.
///
/// # Note
///
/// In Solidity ABI encoding, `uint8[]` and `uint8[N]` are encoded differently from
/// `bytes` and `bytesN`. In Rust/ink!, `Vec<u8>` and `[u8; N]` are mapped to Solidity's
/// `uint8[]` and `uint8[N]` representations, so there's a need for dedicated Rust/ink!
/// types (i.e. `ink::sol::DynBytes` and `ink::sol::FixedBytes<N>`) that map to Solidity's
/// `bytes` and `bytesN` representations.
///
/// # References
///
/// - <https://use-ink.github.io/ink/ink_primitives/sol/struct.FixedBytes.html>
/// - <https://use-ink.github.io/ink/ink_primitives/sol/struct.DynBytes.html>
/// - <https://use.ink/docs/v6/background/solidity-metamask-compatibility#rustink-to-solidity-abi-type-mapping>

#[ink::event]
pub struct FixedBytesPayload {
    pub data: ink::sol::FixedBytes<8>,
}

#[ink::event]
pub struct DynBytesPayload {
    pub data: ink::sol::DynBytes,
}

#[ink::contract]
pub mod bytes {
    use super::{
        DynBytesPayload,
        FixedBytesPayload,
    };

    #[ink(storage)]
    pub struct Bytes;

    impl Bytes {
        /// Creates a new smart contract.
        #[ink(constructor)]
        #[allow(clippy::new_without_default)]
        pub fn new() -> Self {
            Self {}
        }

        /// Handles fixed-size byte arrays.
        #[ink(message)]
        pub fn handle_fixed_bytes(&mut self, data: ink::sol::FixedBytes<8>) {
            self.env().emit_event(FixedBytesPayload { data })
        }

        /// Handles dynamic size byte arrays.
        #[ink(message)]
        pub fn handle_dyn_bytes(&mut self, data: ink::sol::DynBytes) {
            self.env().emit_event(DynBytesPayload { data })
        }
    }

    #[cfg(test)]
    mod tests;
}