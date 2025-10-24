#![cfg_attr(not(feature = "std"), no_std, no_main)]

/// Defines (partial) interface of System precompile.
///
/// See <https://github.com/paritytech/polkadot-sdk/blob/d5bc25b57c300d0477ceb2d53cbbc2e6734da933/substrate/frame/revive/uapi/src/precompiles/system.rs>
#[ink::contract_ref(abi = "sol")]
pub trait System {
    /// Computes Blake2b 256-bit hash of given input.
    ///
    /// # Note
    ///
    /// This signature is the ink! equivalent of the following Solidity signature
    /// ```solidity
    /// function hashBlake256(bytes memory input) external pure returns (bytes32 digest);
    /// ```
    #[ink(message)]
    #[allow(non_snake_case)]
    fn hashBlake256(&self, data: ink::sol::DynBytes) -> ink::sol::FixedBytes<32>;
}

#[ink::contract]
mod precompile {
    use super::System;
    use ink::prelude::vec::Vec;

    #[ink(storage)]
    pub struct Precompile;

    impl Precompile {
        /// Initializes contract.
        #[ink(constructor)]
        #[allow(clippy::new_without_default)]
        pub fn new() -> Self {
            Self {}
        }

        /// Calls the `hashBlake256` function from the `System` precompile and returns the
        /// result.
        #[ink(message)]
        pub fn blake2b_256(&self, data: Vec<u8>) -> [u8; 32] {
            const SYS_ADDR: [u8; 20] =
                hex_literal::hex!("0000000000000000000000000000000000000900");
            let system_ref: super::SystemRef = ink::Address::from(SYS_ADDR).into();
            let in_bytes = ink::sol::DynBytes(data);
            let out_bytes = system_ref.hashBlake256(in_bytes);
            out_bytes.0
        }
    }

    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        use super::*;
        use ink_e2e::ContractsBackend;

        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        #[ink_e2e::test]
        async fn blake2b_256_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // Given
            let mut constructor = PrecompileRef::new();
            let contract = client
                .instantiate("precompile", &ink_e2e::bob(), &mut constructor)
                .submit()
                .await
                .expect("instantiate failed");
            let call_builder = contract.call_builder::<Precompile>();

            // Then
            let data = vec![0x1, 0x2, 0x3, 0x4];
            let expected = blake2b_256_ref(&data);
            let blake2b_256 = call_builder.blake2b_256(data);
            let res = client
                .call(&ink_e2e::bob(), &blake2b_256)
                .submit()
                .await
                .expect("blake2x256 failed");
            assert_eq!(res.return_value(), expected);

            Ok(())
        }

        /// Returns the Blake2b 256-bit hash for the given input.
        fn blake2b_256_ref(input: &[u8]) -> [u8; 32] {
            use blake2::digest::{
                Digest as _,
                consts::U32,
            };

            let mut output = [0u8; 32];
            let mut blake2 = blake2::Blake2b::<U32>::new();
            blake2.update(input);
            let result = blake2.finalize();
            output.copy_from_slice(&result);
            output
        }
    }
}
