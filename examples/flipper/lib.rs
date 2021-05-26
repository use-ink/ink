// Copyright 2018-2021 Parity Technologies (UK) Ltd.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
pub mod flipper {
    use ink_env::hash::Sha2x256;
    use ink_env::DefaultEnvironment;
    use ink_env::hash::HashOutput;
    use ink_env::call::{
        utils::ReturnType,
        Selector,
        CreateParams,
        ExecutionInput,
    };
    use ink_prelude;
    use hex_literal::hex;

    #[ink(storage)]
    pub struct Flipper {
        value: bool,
    }

    impl Flipper {
        /// Creates a new flipper smart contract initialized with the given value.
        #[ink(constructor)]
        pub fn new(init_value: bool) -> Self {
            Self { value: init_value }
        }

        /// Creates a new flipper smart contract initialized to `false`.
        #[ink(constructor)]
        pub fn default() -> Self {
            Self::new(Default::default())
        }

        /// Flips the current value of the Flipper's bool.
        #[ink(message)]
        pub fn flip(&mut self) {
            let additional_randomness = &[];
            let (hash, _block_number) = self.env().random(additional_randomness);
            let _random_bit = hash.as_ref()[0] != 0;


            let value: Balance = 10;
            self.env().transfer(self.env().caller(), value).expect("transfer failed");

            self.value = !self.value;
        }

        /// Returns the current value of the Flipper's bool.
        #[ink(message)]
        pub fn get(&self) -> bool {
            self.value
        }

        #[ink(message)]
        pub fn hash_it(&self, ) -> bool {
                let account_id = self.env().account_id();
                let message = ink_prelude::format!("contract's account id is {:?}", account_id);
                ink_env::debug_println(&message);


            let mut output = <Sha2x256 as HashOutput>::Type::default(); // 256-bit buffer
            let input: &[u8] = &[13, 14, 15];
            let _hash = ink_env::hash_bytes::<Sha2x256>(input, &mut output);
            true
            //let additional_randomness = &[];
            //let (hash, _block_number) = self.env().random(additional_randomness);
            //hash.as_ref()[0] != 0
        }

        /// Returns the accumulator's value.
        #[ink(message)]
        pub fn abc(&self) {
            //let return_type: ReturnType<()> = ReturnType<()>(());
            /*
            //pub struct ReturnType<T>(PhantomData<fn() -> T>);
            //let return_type: ReturnType<()> = ReturnType( || { () } );
            let return_type: ReturnType<()> = ReturnType::default();
            let params = CreateParams {
                /// The code hash of the created contract.
                //code_hash: Hash::from("0xf7cadb78509c34c58d04e07954374c838d9be3569a5e9b50cd6560301effa67a"),
                code_hash: Hash::from(hex!("f7cadb78509c34c58d04e07954374c838d9be3569a5e9b50cd6560301effa67a")),
                /// The maximum gas costs allowed for the instantiation.
                gas_limit: 2500,
                /// The endowment for the instantiated contract.
                endowment: 1000,
                /// The input data for the instantiation.
                //exec_input: ExecutionInput<Args>,
                exec_input:
                        ExecutionInput::new(Selector::new([0xDE, 0xAD, 0xBE, 0xEF]))
                            .push_arg(42)
                            .push_arg(true)
                            .push_arg(&[0x10; 32]),
            /// The salt for determining the hash for the contract account ID.
                salt_bytes: &[0xDE, 0xAD, 0xBE, 0xEF],
                                /// The type of the instantiated contract.
                return_type,
            };
             */

            struct MyContract;
            impl ink_env::call::FromAccountId<DefaultEnvironment> for MyContract {
                fn from_account_id(account_id: AccountId) -> Self { Self }
            }
            let create_params = ink_env::call::build_create::<DefaultEnvironment, MyContract>()
                .code_hash(Hash::from([0x42; 32]))
                .gas_limit(4000)
                .endowment(25)
                .exec_input(
                    ExecutionInput::new(Selector::new([0xDE, 0xAD, 0xBE, 0xEF]))
                        .push_arg(42)
                        .push_arg(true)
                        .push_arg(&[0x10u8; 32])
                )
                .salt_bytes(&[0xDE, 0xAD, 0xBE, 0xEF])
                .params();
            self.env().instantiate_contract(&create_params).expect("instantiation must succeed");

            /*
            //self.accumulator.get()
            let params = CallParams {
                /// The account ID of the to-be-called smart contract.
                callee: self.accumulator,
                /// The maximum gas costs allowed for the call.
                gas_limit: u64,
                /// The transferred value for the call.
                transferred_value: E::Balance,
                /// The expected return type.
                return_type: ReturnType<R>,
                /// The inputs to the execution which is a selector and encoded arguments.
                exec_input: ExecutionInput<Args>,
            };
            self.env().invoke_contract(params).expect("invocation must succeed");
             */
        }

        #[ink(message)]
        pub fn resurrect(&self, contract: AccountId) {
            self.env().restore_contract(contract,
                Hash::from([0x42; 32]),
                1000,
                &[]
            )
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use ink_lang as ink;

        #[ink::test]
        fn default_works() {
            let flipper = Flipper::default();
            assert_eq!(flipper.get(), false);
        }

        #[ink::test]
        fn it_works() {
            let mut flipper = Flipper::new(false);
            assert_eq!(flipper.get(), false);
            flipper.flip();
            assert_eq!(flipper.get(), true);
        }
    }
}
