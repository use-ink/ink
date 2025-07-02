//! A smart contract that uses Solidity ABI encoding to call another ink!
//! contracts that uses Solidity encoding.

#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod sol_cross_contract {
    use crate::keccak_selector;
    use ink::{
        env::{
            call::{
                build_call_solidity,
                ExecutionInput,
            },
            CallFlags,
        },
        U256,
    };

    #[ink(storage)]
    #[derive(Default)]
    pub struct SolCrossContract {}

    impl SolCrossContract {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {}
        }

        #[ink(message)]
        pub fn call_contract_sol_encoding(&mut self, callee: Address) {
            let selector = keccak_selector(b"set_value(bool)");
            let result = build_call_solidity::<<Self as ::ink::env::ContractEnv>::Env>()
                .call(callee)
                .ref_time_limit(1000000000)
                .transferred_value(U256::zero())
                .call_flags(CallFlags::empty())
                .exec_input(ExecutionInput::new(selector.into()).push_arg(true))
                .returns::<()>()
                .try_invoke();

            assert!(result.is_ok(), "call failed");
            assert_eq!(
                result.unwrap(),
                Ok([
                    136, 15, 25, 218, 88, 54, 49, 152, 115, 168, 147, 189, 207, 171, 243,
                    129, 161, 76, 15, 141, 197, 106, 111, 213, 19, 197, 133, 219, 181, 233,
                    195, 120
                ])
            );
        }
    }
}

fn keccak_selector(input: &[u8]) -> [u8; 4] {
    let mut output = [0; 32];
    use sha3::{
        digest::generic_array::GenericArray,
        Digest as _,
    };
    let mut hasher = sha3::Keccak256::new();
    hasher.update(input);
    hasher.finalize_into(<&mut GenericArray<u8, _>>::from(&mut output[..]));
    [output[0], output[1], output[2], output[3]]
}

#[cfg(all(test, feature = "e2e-tests"))]
mod e2e_tests;
