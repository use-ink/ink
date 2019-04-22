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
fn flipper_contract() {
    assert_eq_tokenstreams(
        quote! {
            /// A simple contract that has a boolean value that can be flipped and be retured.
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
            pdsl_model::state! {
                /// A simple contract that has a boolean value that can be flipped and be retured.
                pub struct Flipper {
                    /// The internal value.
                    value: storage::Value<bool>,
                }
            }

            use pdsl_model::messages;

            pdsl_model::messages! {
                /// Flips the internal boolean.
                970692492 => Flip();
                /// Returns the internal boolean.
                4266279973 => Get() -> bool;
            }

            impl Flipper {
                /// The internal boolean is initialized with `true`.
                pub fn deploy(&mut self, env: &mut pdsl_model::EnvHandler) {
                    self.value.set(true)
                }

                /// Flips the internal boolean.
                pub fn flip(&mut self, env: &mut pdsl_model::EnvHandler) {
                    self.value = !(*self.value)
                }

                /// Returns the internal boolean.
                pub fn get(&self, env: &pdsl_model::EnvHandler) -> bool {
                    *self.value
                }
            }

            use pdsl_model::Contract as _;

            #[cfg(not(test))]
            impl Flipper {
                pub(crate) fn instantiate() -> impl pdsl_model::Contract {
                    pdsl_model::ContractDecl::using::<Self>()
                        .on_deploy(|env, ()| {
                            let (handler, state) = env.split_mut();
                            state.deploy(handler,)
                        })
                        .on_msg_mut::<Flip>(|env, _| {
                            let (handler, state) = env.split_mut();
                            state.flip(handler,)
                        })
                        .on_msg::<Get>(|env, _| {
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
                    env: pdsl_model::ExecutionEnv<Flipper>,
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
                        use pdsl_core::storage::{
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
                                pdsl_model::ExecutionEnv::allocate_using(&mut alloc).initialize_into(())
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
