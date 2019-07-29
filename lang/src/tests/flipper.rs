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

            /// A simple contract that has a boolean value that can be flipped and be returned.
            struct Flipper {
                /// The internal value.
                value: storage::Value<bool>,
            }

            impl Deploy for Flipper {
                /// The internal boolean is initialized with `true`.
                fn deploy(&mut self) {
                    self.value.set(true)
                }
            }

            impl Flipper {
                /// Flips the internal boolean.
                pub(external) fn flip(&mut self) {
                    self.value = !(*self.value)
                }

                /// Returns the internal boolean.
                pub(external) fn get(&self) -> bool {
                    *self.value
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
                /// A simple contract that has a boolean value that can be flipped and be returned.
                pub struct Flipper {
                    /// The internal value.
                    value: storage::Value<bool>,
                }
            }

            mod msg {
                use super::*;
                use ink_model::messages;

                ink_model::messages! {
                    /// Flips the internal boolean.
                    970692492 => Flip();
                    /// Returns the internal boolean.
                    4266279973 => Get() -> bool;
                }
            }

            impl Flipper {
                /// The internal boolean is initialized with `true`.
                pub fn deploy(&mut self, env: &mut ink_model::EnvHandler<ink_core::env::ContractEnv<DefaultSrmlTypes> >) {
                    self.value.set(true)
                }

                /// Flips the internal boolean.
                pub fn flip(&mut self, env: &mut ink_model::EnvHandler<ink_core::env::ContractEnv<DefaultSrmlTypes> >) {
                    self.value = !(*self.value)
                }

                /// Returns the internal boolean.
                pub fn get(&self, env: &ink_model::EnvHandler<ink_core::env::ContractEnv<DefaultSrmlTypes> >) -> bool {
                    *self.value
                }
            }

            use ink_model::Contract as _;

            #[cfg(not(test))]
            impl Flipper {
                pub(crate) fn instantiate() -> impl ink_model::Contract {
                    ink_model::ContractDecl::using::<Self, ink_core::env::ContractEnv<DefaultSrmlTypes>>()
                        .on_deploy(|env, ()| {
                            let (handler, state) = env.split_mut();
                            state.deploy(handler,)
                        })
                        .on_msg_mut::<msg::Flip>(|env, _| {
                            let (handler, state) = env.split_mut();
                            state.flip(handler,)
                        })
                        .on_msg::<msg::Get>(|env, _| {
                            let (handler, state) = env.split();
                            state.get(handler,)
                        })
                        .instantiate()
                }
            }

            #[cfg(not(test))] #[no_mangle] fn deploy() { Flipper::instantiate().deploy() }
            #[cfg(not(test))] #[no_mangle] fn call() { Flipper::instantiate().dispatch() }

            #[cfg(test)]
            mod test {
                use super::*;

                pub struct TestableFlipper {
                    env: ink_model::ExecutionEnv<Flipper, ink_core::env::ContractEnv<DefaultSrmlTypes>>,
                }

                impl Flipper {
                    /// Returns a testable version of the contract.
                    pub fn deploy_mock() -> TestableFlipper {
                        let mut mock = TestableFlipper::allocate();
                        mock.deploy();
                        mock
                    }
                }

                impl TestableFlipper {
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
                    fn deploy(&mut self,) {
                        let (handler, state) = self.env.split_mut();
                        state.deploy(handler,)
                    }
                }

                impl TestableFlipper {
                    pub fn flip(&mut self) {
                        let (handler, state) = self.env.split_mut();
                        state.flip(handler,)
                    }

                    pub fn get(&self) -> bool {
                        let (handler, state) = self.env.split();
                        state.get(handler,)
                    }
                }
            }
        },
    )
}
