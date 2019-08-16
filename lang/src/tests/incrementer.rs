// Copyright 2018-2019 Parity Technologies (UK) Ltd.
// This file is part of ink!.
//
// ink! is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// ink! is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with ink!.  If not, see <http://www.gnu.org/licenses/>.

use super::*;

#[test]
fn contract_compiles() {
    assert_eq_tokenstreams(
        quote! {
            #![env = DefaultSrmlTypes]

            /// A simple contract that has a value that can be
            /// incremented, returned and compared.
            struct Incrementer {
                /// The internal value.
                value: storage::Value<u32>,
            }

            impl Deploy for Incrementer {
                /// Automatically called when the contract is deployed.
                fn deploy(&mut self, init_value: u32) {
                    self.value.set(init_value)
                }
            }

            impl Incrementer {
                /// Increments the internal counter.
                pub(external) fn inc(&mut self, by: u32) {
                    self.value += by
                }

                /// Returns the internal counter.
                pub(external) fn get(&self) -> u32 {
                    *self.value
                }

                /// Returns `true` if `x` is greater than the internal value.
                pub(external) fn compare(&self, x: u32) -> bool {
                    x > *self.value
                }
            }
        },
        quote! {
            mod types {
                use super::*;
                use ink_core::env::{ContractEnv, EnvTypes};

                pub type AccountId = <ContractEnv<DefaultSrmlTypes> as EnvTypes>::AccountId;
                pub type Balance = <ContractEnv<DefaultSrmlTypes> as EnvTypes>::Balance;
                pub type Hash = <ContractEnv<DefaultSrmlTypes> as EnvTypes>::Hash;
                pub type Moment = <ContractEnv<DefaultSrmlTypes> as EnvTypes>::Moment;
                pub type BlockNumber = <ContractEnv<DefaultSrmlTypes> as EnvTypes>::BlockNumber;
            }

            use types::{
                AccountId,
                Balance,
                Hash,
                Moment,
                BlockNumber,
            };

            ink_model::state! {
                /// A simple contract that has a value that can be
                /// incremented, returned and compared.
                #[cfg_attr(
                    feature = "ink-generate-abi",
                    derive(type_metadata::Metadata, ink_abi::HasLayout,)
                )]
                pub struct Incrementer {
                    /// The internal value.
                    value: storage::Value<u32>,
                }
            }

            mod msg {
                use super::*;
                use ink_model::messages;

                ink_model::messages! {
                    /// Increments the internal counter.
                    257544423 => Inc(by: u32);
                    /// Returns the internal counter.
                    4266279973 => Get() -> u32;
                    /// Returns `true` if `x` is greater than the internal value.
                    363906316 => Compare(x: u32) -> bool;
                }
            }

            impl Incrementer {
                /// Automatically called when the contract is deployed.
                pub fn deploy(&mut self, env: &mut ink_model::EnvHandler<ink_core::env::ContractEnv<DefaultSrmlTypes> >, init_value: u32) {
                    self.value.set(init_value)
                }

                /// Increments the internal counter.
                pub fn inc(&mut self, env: &mut ink_model::EnvHandler<ink_core::env::ContractEnv<DefaultSrmlTypes> >, by: u32) {
                    self.value += by
                }

                /// Returns the internal counter.
                pub fn get(&self, env: &ink_model::EnvHandler<ink_core::env::ContractEnv<DefaultSrmlTypes> >) -> u32 {
                    *self.value
                }

                /// Returns `true` if `x` is greater than the internal value.
                pub fn compare(&self, env: &ink_model::EnvHandler<ink_core::env::ContractEnv<DefaultSrmlTypes> >, x: u32) -> bool {
                    x > *self.value
                }
            }

            use ink_model::Contract as _;

            #[cfg(not(test))]
            impl Incrementer {
                pub(crate) fn instantiate() -> impl ink_model::Contract {
                    ink_model::ContractDecl::using::<Self, ink_core::env::ContractEnv<DefaultSrmlTypes>>()
                        .on_deploy(|env, init_value: u32| {
                            let (handler, state) = env.split_mut();
                            state.deploy(handler, init_value)
                        })
                        .on_msg_mut::<msg::Inc>(|env, by: u32| {
                            let (handler, state) = env.split_mut();
                            state.inc(handler, by)
                        })
                        .on_msg::<msg::Get>(|env, _| {
                            let (handler, state) = env.split();
                            state.get(handler,)
                        })
                        .on_msg::<msg::Compare>(|env, x: u32| {
                            let (handler, state) = env.split();
                            state.compare(handler, x)
                        })
                        .instantiate()
                }
            }

            #[cfg(not(test))] #[no_mangle] fn deploy() -> u32 { Incrementer::instantiate().deploy().to_u32() }
            #[cfg(not(test))] #[no_mangle] fn call() -> u32 { Incrementer::instantiate().dispatch().to_u32() }

            #[cfg(test)]
            mod test {
                use super::*;

                pub struct TestableIncrementer {
                    env: ink_model::ExecutionEnv<Incrementer, ink_core::env::ContractEnv<DefaultSrmlTypes>>,
                }

                impl Incrementer {
                    /// Returns a testable version of the contract.
                    pub fn deploy_mock(init_value: u32) -> TestableIncrementer {
                        let mut mock = TestableIncrementer::allocate();
                        mock.deploy(init_value);
                        mock
                    }
                }

                impl TestableIncrementer {
                    /// Allocates the testable contract storage.
                    fn allocate() -> Self {
                        use ink_core::storage::{
                            Key,
                            alloc::{
                                AllocateUsing as _,
                                Initialize as _,
                                BumpAlloc,
                            },
                        };
                        Self {
                            env: unsafe {
                                let mut alloc = BumpAlloc::from_raw_parts(Key([0x0; 32]));
                                ink_model::ExecutionEnv::allocate_using(&mut alloc).initialize_into(())
                            }
                        }
                    }

                    /// Deploys the testable contract by initializing it with the given values.
                    fn deploy(&mut self , init_value: u32) {
                        let (handler, state) = self.env.split_mut();
                        state.deploy(handler, init_value)
                    }
                }

                impl TestableIncrementer {
                    pub fn inc(& mut self, by: u32) {
                        let (handler, state) = self.env.split_mut();
                        state.inc(handler, by)
                    }

                    pub fn get(&self) -> u32 {
                        let (handler, state) = self.env.split();
                        state.get(handler,)
                    }

                    pub fn compare(&self, x: u32) -> bool {
                        let (handler, state) = self.env.split();
                        state.compare(handler, x)
                    }
                }
            }

            #[cfg(feature = "ink-generate-abi")]
            pub fn ink_generate_abi() -> ink_abi::InkProject {
                let contract = {
                    ink_abi::ContractSpec::new("Incrementer")
                        .on_deploy(ink_abi::DeploySpec::new()
                            .args(vec![
                                ink_abi::MessageParamSpec::new::<u32>("init_value")
                            ])
                            .docs(vec![
                                "Automatically called when the contract is deployed."
                            ])
                            .done()
                        )
                        .messages(vec![
                            ink_abi::MessageSpec::new("inc")
                                .selector(257544423u32)
                                .mutates(true)
                                .args(vec![
                                    ink_abi::MessageParamSpec::new::<u32>("by"),
                                ])
                                .docs(vec![
                                    "Increments the internal counter.",
                                ])
                                .returns(
                                    ink_abi::ReturnTypeSpec::none()
                                )
                                .done(),
                            ink_abi::MessageSpec::new("get")
                                .selector(4266279973u32)
                                .mutates(false)
                                .args(vec![])
                                .docs(vec![
                                    "Returns the internal counter.",
                                ])
                                .returns(
                                    ink_abi::ReturnTypeSpec::new::<u32>()
                                )
                                .done(),
                            ink_abi::MessageSpec::new("compare")
                                .selector(363906316u32)
                                .mutates(false)
                                .args(vec![ink_abi::MessageParamSpec::new::<u32>("x"),
                            ])
                            .docs(vec![
                                "Returns `true` if `x` is greater than the internal value.",
                            ])
                            .returns(
                                ink_abi::ReturnTypeSpec::new::<bool>()
                            )
                            .done(),
                        ])
                        .events(vec![])
                        .docs(vec![])
                        .done()
                };
                let layout = {
                    unsafe {
                        use ink_core::storage::alloc::AllocateUsing as _;
                        use ink_abi::HasLayout as _;
                        Incrementer::allocate_using(
                            &mut ink_core::storage::alloc::BumpAlloc::from_raw_parts(
                                ink_core::storage::Key([0x0; 32])
                            )
                        ).layout()
                    }
                };
                ink_abi::InkProject::new(layout, contract)
            }
        },
    )
}
