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

            /// The contract that does nothing.
            ///
            /// # Note
            ///
            /// Can be deployed, cannot be called.
            struct Noop {}

            impl Deploy for Noop {
                /// Does nothing to initialize itself.
                fn deploy(&mut self) {}
            }

            /// Provides no way to call it as extrinsic.
            impl Noop {}
        },
        quote! {
            mod types {
                use super::*;
                use ink_core::env::{ContractEnv, EnvTypes};

                pub type AccountId = <ContractEnv<DefaultSrmlTypes> as EnvTypes>::AccountId;
                pub type Balance = <ContractEnv<DefaultSrmlTypes> as EnvTypes>::Balance;
                pub type Hash = <ContractEnv<DefaultSrmlTypes> as EnvTypes>::Hash;
                pub type Moment = <ContractEnv<DefaultSrmlTypes> as EnvTypes>::Moment;
            }

            use types::{
                AccountId,
                Balance,
                Hash,
                Moment,
            };

            ink_model::state! {
                /// The contract that does nothing.
                ///
                /// # Note
                ///
                /// Can be deployed, cannot be called.
                pub struct Noop {}
            }

            mod msg {
                use super::*;
                use ink_model::messages;

                ink_model::messages! {}
            }

            impl Noop {
                /// Does nothing to initialize itself.
                pub fn deploy(&mut self, env: &mut ink_model::EnvHandler<ink_core::env::ContractEnv<DefaultSrmlTypes> >) { }
            }

            use ink_model::Contract as _;

            #[cfg(not(test))]
            impl Noop {
                pub(crate) fn instantiate() -> impl ink_model::Contract {
                    ink_model::ContractDecl::using::<Self, ink_core::env::ContractEnv<DefaultSrmlTypes>>()
                        .on_deploy(|env, ()| {
                            let (handler, state) = env.split_mut();
                            state.deploy(handler,)
                        })
                        .instantiate()
                }
            }

            #[cfg(not(test))] #[no_mangle] fn deploy() { Noop::instantiate().deploy() }
            #[cfg(not(test))] #[no_mangle] fn call() { Noop::instantiate().dispatch() }

            #[cfg(test)]
            mod test {
                use super::*;

                pub struct TestableNoop {
                    env: ink_model::ExecutionEnv<Noop, ink_core::env::ContractEnv<DefaultSrmlTypes>>,
                }

                impl Noop {
                    /// Returns a testable version of the contract.
                    pub fn deploy_mock() -> TestableNoop {
                        let mut mock = TestableNoop::allocate();
                        mock.deploy();
                        mock
                    }
                }

                impl TestableNoop {
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

                impl TestableNoop { }
            }
        },
    )
}
