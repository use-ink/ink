#![cfg_attr(not(feature = "std"), no_std, no_main)]

/// This trait is an implementation of the Solidity interface found at
/// <https://github.com/use-ink/ink-node/blob/main/runtime/src/IDemo.sol>.
///
/// The precompile code itself can be found at
/// <https://github.com/use-ink/ink-node/blob/main/runtime/src/demo_precompile.rs>.
///
/// Note that it's possible to just implement the Solidity interface partially
/// in this trait. This can be useful if you just want to expose part of the
/// precompile functionality.
#[ink::contract_ref(abi = "sol")]
pub trait System {
    /// Simple echo function.
    ///
    /// If `mode = 0`, the function reverts.
    /// If `mode > 0`, the input `message` is echoed back to the caller.
    ///
    /// # Note
    ///
    /// This signature is the ink! equivalent of the following Solidity signature
    ///
    /// ```solidity
    /// function echo(uint8 mode, bytes message) external view returns (bytes);
    /// ```
    #[ink(message)]
    #[allow(non_snake_case)]
    fn echo(&self, mode: u8, message: ink::sol::DynBytes) -> ink::sol::DynBytes;
}

#[ink::contract]
mod precompile_demo {
    use super::System;
    use ink::prelude::vec::Vec;

    #[ink(storage)]
    pub struct PrecompileDemo;

    impl PrecompileDemo {
        /// Initializes contract.
        #[ink(constructor)]
        #[allow(clippy::new_without_default)]
        pub fn new() -> Self {
            Self {}
        }

        /// Calls the `echo` function from `ink-node`'s `DemoPrecompile`.
        #[ink(message)]
        pub fn call_echo(&self, data: Vec<u8>) -> Vec<u8> {
            const DEMO_PRECOMPILE_ADDR: [u8; 20] =
                hex_literal::hex!("00000000000000000000000000000000000B0000");
            let system_ref: super::SystemRef =
                ink::Address::from(DEMO_PRECOMPILE_ADDR).into();
            let in_bytes = ink::sol::DynBytes(data);
            let out_bytes = system_ref.echo(1, in_bytes);
            out_bytes.0
        }
    }

    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        use super::*;
        use ink_e2e::ContractsBackend;

        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        #[ink_e2e::test]
        async fn call_echo_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // given
            let mut constructor = PrecompileDemoRef::new();
            let contract = client
                .instantiate("precompile_demo", &ink_e2e::bob(), &mut constructor)
                .submit()
                .await
                .expect("instantiate failed");
            let call_builder = contract.call_builder::<PrecompileDemo>();

            // when
            let data = vec![0x1, 0x2, 0x3, 0x4];
            let expected = data.clone();
            let call_echo = call_builder.call_echo(data);
            let res = client
                .call(&ink_e2e::bob(), &call_echo)
                .submit()
                .await
                .expect("call_echo failed");

            // then
            assert_eq!(res.return_value(), expected);

            Ok(())
        }
    }
}
