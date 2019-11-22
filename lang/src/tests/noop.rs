// Copyright 2018-2019 Parity Technologies (UK) Ltd.
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

                pub type AccountId = <DefaultSrmlTypes as EnvTypes>::AccountId;
                pub type Balance = <DefaultSrmlTypes as EnvTypes>::Balance;
                pub type Hash = <DefaultSrmlTypes as EnvTypes>::Hash;
                pub type Moment = <DefaultSrmlTypes as EnvTypes>::Moment;
                pub type BlockNumber = <DefaultSrmlTypes as EnvTypes>::BlockNumber;
            }

            type Env = ink_core::env::ContractEnv<DefaultSrmlTypes>;
            use types::{
                AccountId,
                Balance,
                Hash,
                Moment,
                BlockNumber,
            };

            #[cfg(not(feature = "ink-as-dependency"))]
            mod normal {
                use super::*;

                ink_model::state! {
                    /// The contract that does nothing.
                    ///
                    /// # Note
                    ///
                    /// Can be deployed, cannot be called.
                    #[cfg_attr(
                        feature = "ink-generate-abi",
                        derive(type_metadata::Metadata, ink_abi::HasLayout,)
                    )]
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

                #[cfg(not(test))] #[no_mangle] fn deploy() -> u32 { Noop::instantiate().deploy().to_u32() }
                #[cfg(not(test))] #[no_mangle] fn call() -> u32 { Noop::instantiate().dispatch().to_u32() }
            }

            #[cfg(not(feature = "ink-as-dependency"))]
            use normal::*;

            #[cfg(not(feature = "ink-as-dependency"))]
            use ink_core::env::FromAccountId as _;

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

            #[cfg(not(feature = "ink-as-dependency"))]
            #[cfg(feature = "ink-generate-abi")]
            pub fn ink_generate_abi() -> ink_abi::InkProject {
                let contract = {
                    ink_abi::ContractSpec::new("Noop")
                        .constructors(vec![
                            ink_abi::ConstructorSpec::new("on_deploy")
                                .selector([0u8; 4])
                                .args(vec![])
                                .docs(vec![
                                    "Does nothing to initialize itself.",
                                ])
                                .done()
                        ])
                        .messages(vec![])
                        .events(vec![])
                        .docs(vec![])
                        .done()
                };
                let layout = {
                    unsafe {
                        use ink_core::storage::alloc::AllocateUsing as _;
                        use ink_abi::HasLayout as _;
                        Noop::allocate_using(
                            &mut ink_core::storage::alloc::BumpAlloc::from_raw_parts(
                                ink_core::storage::Key(
                                    [0x0; 32]
                                )
                            ))
                            .layout()
                    }
                };
                ink_abi::InkProject::new(layout, contract)
            }

            #[cfg(feature = "ink-as-dependency")]
            mod as_dependency {
                use super::*;

                /// The contract that does nothing.
                ///
                /// # Note
                ///
                /// Can be deployed, cannot be called.
                #[derive(Clone, scale::Encode, scale::Decode)]
                #[cfg_attr(feature = "ink-generate-abi", derive(type_metadata::Metadata))]
                pub struct Noop {
                    account_id: AccountId,
                }

                impl ink_core::storage::Flush for Noop {}

                /// Allows to enhance calls to `&self` contract messages.
                pub struct CallEnhancer<'a> {
                    contract: &'a Noop,
                }

                /// Allows to enhance calls to `&mut self` contract messages.
                pub struct CallEnhancerMut<'a> {
                    contract: &'a mut Noop,
                }

                impl ink_core::env::FromAccountId<Env> for Noop {
                    fn from_account_id(account_id: AccountId) -> Self {
                        Self { account_id }
                    }
                }

                impl Noop {
                    /// Does nothing to initialize itself.
                    pub fn new(code_hash: Hash,) -> ink_core::env::CreateBuilder<Env, Self> {
                        ink_core::env::CreateBuilder::<Env, Self>::new(code_hash)
                    }
                    /// Returns the internal account ID of the contract.
                    pub fn account_id(&self) -> AccountId {
                        self.account_id
                    }
                    /// Allows to enhance calls to `&self` contract messages.
                    pub fn call(&self) -> CallEnhancer {
                        CallEnhancer { contract : self }
                    }
                    /// Allows to enhance calls to `&mut self` contract messages.
                    pub fn call_mut(&mut self) -> CallEnhancerMut {
                        CallEnhancerMut { contract : self }
                    }
                }

                impl Noop { }

                impl<'a> CallEnhancer<'a> { }
                impl<'a> CallEnhancerMut<'a> { }
            }

            #[cfg(feature = "ink-as-dependency")]
            pub use as_dependency::{Noop, CallEnhancer, CallEnhancerMut,};
        },
    )
}
