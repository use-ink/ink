// Copyright 2018-2019 Parity Technologies (UK) Ltd.
// This file is part of pDSL.
//
// pDSL is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// pDSL is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with pDSL.  If not, see <http://www.gnu.org/licenses/>.

use super::*;

#[test]
fn incrementer_contract() {
    assert_eq_tokenstreams(
        quote! {
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
            pdsl_model::state! {
                /// A simple contract that has a value that can be
                /// incremented, returned and compared.
                pub struct Incrementer {
                    /// The internal value.
                    value: storage::Value<u32>,
                }
            }

            use pdsl_model::messages;

            pdsl_model::messages! {
                /// Increments the internal counter.
                257544423 => Inc(by: u32);
                /// Returns the internal counter.
                4266279973 => Get() -> u32;
                /// Returns `true` if `x` is greater than the internal value.
                363906316 => Compare(x: u32) -> bool;
            }

            impl Incrementer {
                /// Automatically called when the contract is deployed.
                pub fn deploy(&mut self, env: &mut pdsl_model::EnvHandler, init_value: u32) {
                    self.value.set(init_value)
                }

                /// Increments the internal counter.
                pub fn inc(&mut self, env: &mut pdsl_model::EnvHandler, by: u32) {
                    self.value += by
                }

                /// Returns the internal counter.
                pub fn get(&self, env: &pdsl_model::EnvHandler) -> u32 {
                    *self.value
                }

                /// Returns `true` if `x` is greater than the internal value.
                pub fn compare(&self, env: &pdsl_model::EnvHandler, x: u32) -> bool {
                    x > *self.value
                }
            }

            use pdsl_model::Contract as _;

            #[cfg(not(test))]
            impl Incrementer {
                pub(crate) fn instantiate() -> impl pdsl_model::Contract {
                    pdsl_model::ContractDecl::using::<Self>()
                        .on_deploy(|env, init_value: u32| {
                            let (handler, state) = env.split_mut();
                            state.deploy(handler, init_value)
                        })
                        .on_msg_mut::<Inc>(|env, by: u32| {
                            let (handler, state) = env.split_mut();
                            state.inc(handler, by)
                        })
                        .on_msg::<Get>(|env, _| {
                            let (handler, state) = env.split();
                            state.get(handler,)
                        })
                        .on_msg::<Compare>(|env, x: u32| {
                            let (handler, state) = env.split();
                            state.compare(handler, x)
                        })
                        .instantiate()
                }
            }

            #[cfg(not(test))] #[no_mangle] fn deploy() { Incrementer::instantiate().deploy() }
            #[cfg(not(test))] #[no_mangle] fn call() { Incrementer::instantiate().dispatch() }

            #[cfg(test)]
            mod test {
                use super::*;

                pub struct TestableIncrementer {
                    env: pdsl_model::ExecutionEnv<Incrementer>,
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
                        use pdsl_core::storage::{
                            Key,
                            alloc::{
                                AllocateUsing,
                                BumpAlloc,
                            },
                        };
                        Self {
                            env: unsafe {
                                let mut alloc = BumpAlloc::from_raw_parts(Key([0x0; 32]));
                                AllocateUsing::allocate_using(&mut alloc)
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
        },
    )
}
