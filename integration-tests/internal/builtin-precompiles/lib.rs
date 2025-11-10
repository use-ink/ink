#![cfg_attr(not(feature = "std"), no_std, no_main)]
#![allow(clippy::new_without_default)]

#[ink::contract]
mod builtin_precompiles {
    use ink::U256;

    #[ink(storage)]
    pub struct BuiltinPrecompiles {}

    impl BuiltinPrecompiles {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {}
        }

        /// Tests the `bn128_add` functionality.
        #[ink(message)]
        pub fn bn128_add(&self) -> (U256, U256) {
            self.env().bn128_add(1.into(), 2.into(), 1.into(), 2.into())
        }

        /// Tests the `bn128_mul` functionality.
        #[ink(message)]
        pub fn bn128_mul(&self) -> (U256, U256) {
            self.env().bn128_mul(1.into(), 2.into(), 3.into())
        }

        /// Tests the `bn128_pairing` functionality.
        #[ink(message)]
        pub fn bn128_pairing(&self, input: ink::prelude::vec::Vec<u8>) -> bool {
            self.env().bn128_pairing(input.as_ref())
        }
    }

    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        use super::*;
        use impl_serde::serialize as serde_hex;
        use ink_e2e::ContractsBackend;

        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        #[ink_e2e::test]
        async fn bn128_add_works<Client: E2EBackend>(
            mut client: Client,
        ) -> E2EResult<()> {
            // given
            let mut constructor = BuiltinPrecompilesRef::new();
            let contract = client
                .instantiate("builtin-precompiles", &ink_e2e::eve(), &mut constructor)
                .submit()
                .await
                .expect("instantiate failed");
            let call_builder = contract.call_builder::<BuiltinPrecompiles>();

            // when
            let (x3, y3) = client
                .call(&ink_e2e::eve(), &call_builder.bn128_add())
                .dry_run()
                .await?
                .return_value();

            // then
            // taken from https://github.com/polkadot-developers/polkavm-hardhat-examples/blob/v0.0.3/precompiles-hardhat/test/BN128Add.js
            assert_eq!(
                format!("{}", x3),
                "1368015179489954701390400359078579693043519447331113978918064868415326638035"
            );
            assert_eq!(
                format!("{}", y3),
                "9918110051302171585080402603319702774565515993150576347155970296011118125764"
            );

            Ok(())
        }

        #[ink_e2e::test]
        async fn bn128_mul_works<Client: E2EBackend>(
            mut client: Client,
        ) -> E2EResult<()> {
            // given
            let mut constructor = BuiltinPrecompilesRef::new();
            let contract = client
                .instantiate("builtin-precompiles", &ink_e2e::eve(), &mut constructor)
                .submit()
                .await
                .expect("instantiate failed");
            let call_builder = contract.call_builder::<BuiltinPrecompiles>();

            // when
            let (x2, y2) = client
                .call(&ink_e2e::eve(), &call_builder.bn128_mul())
                .dry_run()
                .await?
                .return_value();

            // then
            // taken from https://github.com/polkadot-developers/polkavm-hardhat-examples/blob/v0.0.3/precompiles-hardhat/test/BN128Mul.js
            assert_eq!(
                format!("{}", x2),
                "3353031288059533942658390886683067124040920775575537747144343083137631628272"
            );
            assert_eq!(
                format!("{}", y2),
                "19321533766552368860946552437480515441416830039777911637913418824951667761761"
            );

            Ok(())
        }

        #[ink_e2e::test]
        async fn bn128_pairing_works<Client: E2EBackend>(
            mut client: Client,
        ) -> E2EResult<()> {
            // given
            let mut constructor = BuiltinPrecompilesRef::new();
            let contract = client
                .instantiate("builtin-precompiles", &ink_e2e::eve(), &mut constructor)
                .submit()
                .await
                .expect("instantiate failed");
            let call_builder = contract.call_builder::<BuiltinPrecompiles>();

            // when
            // Taken from https://github.com/polkadot-developers/polkavm-hardhat-examples/blob/v0.0.3/precompiles-hardhat/test/BN128Pairing.js
            //
            // Using the "two_point_match_2" test vector from your data
            // This is a known valid pairing that should return true.
            let input: [u8; 384] = hex_literal::hex!(
                "00000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000002198e9393920d483a7260bfb731fb5d25f1aa493335a9e71297e485b7aef312c21800deef121f1e76426a00665e5c4479674322d4f75edadd46debd5cd992f6ed090689d0585ff075ec9e99ad690c3395bc4b313370b38ef355acdadcd122975b12c85ea5db8c6deb4aab71808dcb408fe3d1e7690c43d37b4ce6cc0166fa7daa00000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000002198e9393920d483a7260bfb731fb5d25f1aa493335a9e71297e485b7aef312c21800deef121f1e76426a00665e5c4479674322d4f75edadd46debd5cd992f6ed275dc4a288d1afb3cbb1ac09187524c7db36395df7be3b99e673b13a075a65ec1d9befcd05a5323e6da4d435f3b617cdb3af83285c2df711ef39c01571827f9d"
            );
            let res = client
                .call(&ink_e2e::eve(), &call_builder.bn128_pairing(input.to_vec()))
                .dry_run()
                .await?
                .return_value();

            // then
            assert!(res);

            Ok(())
        }

        #[ink_e2e::test]
        async fn bn128_pairing_zero_points_works<Client: E2EBackend>(
            mut client: Client,
        ) -> E2EResult<()> {
            // given
            let mut constructor = BuiltinPrecompilesRef::new();
            let contract = client
                .instantiate("builtin-precompiles", &ink_e2e::eve(), &mut constructor)
                .submit()
                .await
                .expect("instantiate failed");
            let call_builder = contract.call_builder::<BuiltinPrecompiles>();

            // when
            // Taken from https://github.com/polkadot-developers/polkavm-hardhat-examples/blob/v0.0.3/precompiles-hardhat/test/BN128Pairing.js
            //
            // Pairing with zero points:
            // The zero point in G1 is (0, 0) and in G2 is ((0, 0), (0, 0))
            // Pairing of zero points should return true
            let pairing = format!(
                "{}{}{}{}{}{}",
                // G1 zero point
                "0000000000000000000000000000000000000000000000000000000000000000", /* G1.x = 0 */
                "0000000000000000000000000000000000000000000000000000000000000000", /* G1.y = 0 */
                // G2 zero point
                "0000000000000000000000000000000000000000000000000000000000000000", /* G2.x imaginary = 0 */
                "0000000000000000000000000000000000000000000000000000000000000000", /* G2.x real = 0 */
                "0000000000000000000000000000000000000000000000000000000000000000", /* G2.y imaginary = 0 */
                "0000000000000000000000000000000000000000000000000000000000000000"
            ); // G2.y real = 0
            let input = serde_hex::from_hex(&pairing).expect("parsing hex failed");
            let res = client
                .call(&ink_e2e::eve(), &call_builder.bn128_pairing(input))
                .dry_run()
                .await?
                .return_value();

            // then
            assert!(res);

            Ok(())
        }
    }
}
