//! A smart contract to test using RLP encoding for EVM compatibility.

#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract(abi_encoding = "rlp")]
mod rlp_cross_contract {
    use ink::env::call::{build_call, ExecutionInput};
    use ink::env::{debug_println, CallFlags};
    use crate::keccak_selector;

    #[ink(storage)]
    #[derive(Default)]
    pub struct RlpCrossContract {}

    impl RlpCrossContract {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self { }
        }

        #[ink(message)]
        pub fn call_contract_rlp(&mut self, callee: [u8;32]) {
            let selector = keccak_selector(b"flip");
            let callee: AccountId = callee.into();

            // TODO: fails due to call builder encoding with scale
            let result = build_call::<<Self as ::ink::env::ContractEnv>::Env>()
                .call(callee)
                .ref_time_limit(1000000000)
                .transferred_value(0)
                .call_flags(CallFlags::empty())
                .exec_input(
                    ExecutionInput::new(selector.into())
                )
                .returns::<()>()
                .try_invoke();

            debug_println!("call_contract_rlp: {:?}", result);

            assert!(result.is_ok(), "call failed");
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
