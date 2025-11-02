#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract_ref(abi = "sol")]
pub trait Precompile {
    #[ink(message)]
    fn fortytwo(&self) -> u64;
}

#[ink::contract]
mod precompile_demo {
    use super::Precompile;

    #[ink(storage)]
    pub struct PrecompileDemo;

    impl PrecompileDemo {
        #[ink(constructor)]
        #[allow(clippy::new_without_default)]
        pub fn new() -> Self {
            Self {}
        }

        #[ink(message)]
        pub fn demo(&self) -> u64 {
            const PRECOMPILE_ADDR: [u8; 20] =
                hex_literal::hex!("00000000000000000000000000000000000C0000");
            let precompile_ref: super::PrecompileRef = ink::Address::from(PRECOMPILE_ADDR).into();
            precompile_ref.fortytwo()
        }
    }

    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        use super::*;
        use ink_e2e::ContractsBackend;

        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        #[ink_e2e::test]
        async fn works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // Given
            let mut constructor = PrecompileDemoRef::new();
            let contract = client
                .instantiate("precompile_demo", &ink_e2e::bob(), &mut constructor)
                .submit()
                .await
                .expect("instantiate failed");
            let call_builder = contract.call_builder::<PrecompileDemo>();

            // Then
            let call = call_builder.demo();
            let res = client
                .call(&ink_e2e::bob(), &call)
                .submit()
                .await
                .expect("call failed");
            assert_eq!(res.return_value(), 42);

            Ok(())
        }
    }
}
