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
                        [15, 89, 208, 231] => Inc(by: u32);
                        /// Returns the internal counter.
                        [254, 74, 68, 37] => Get() -> u32;
                        /// Returns `true` if `x` is greater than the internal value.
                        [21, 176, 197, 12] => Compare(x: u32) -> bool;
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
            }

            #[cfg(not(feature = "ink-as-dependency"))]
            use normal::*;

            #[cfg(not(feature = "ink-as-dependency"))]
            use ink_core::env::FromAccountId as _;

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

            #[cfg(not(feature = "ink-as-dependency"))]
            #[cfg(feature = "ink-generate-abi")]
            pub fn ink_generate_abi() -> ink_abi::InkProject {
                let contract = {
                    ink_abi::ContractSpec::new("Incrementer")
                        .constructors(vec![
                            ink_abi::ConstructorSpec::new("on_deploy")
                                .selector([0u8; 4])
                                .args(vec![
                                    ink_abi::MessageParamSpec::new("init_value")
                                        .of_type(
                                            ink_abi::TypeSpec::with_name_segs::<u32, _>(
                                                vec!["u32"].into_iter().map(AsRef::as_ref)
                                            )
                                        )
                                        .done(),
                                ])
                                .docs(vec![
                                    "Automatically called when the contract is deployed.",
                                ])
                                .done()
                        ])
                        .messages(vec![
                            ink_abi::MessageSpec::new("inc")
                                .selector([15, 89, 208, 231])
                                .mutates(true)
                                .args(vec![
                                    ink_abi::MessageParamSpec::new("by")
                                        .of_type(
                                            ink_abi::TypeSpec::with_name_segs::<u32, _>(
                                                vec!["u32"].into_iter().map(AsRef::as_ref)
                                            )
                                        )
                                        .done(),
                                ])
                                .docs(vec![
                                    "Increments the internal counter.",
                                ])
                                .returns(
                                    ink_abi::ReturnTypeSpec::new(None)
                                )
                                .done(),
                            ink_abi::MessageSpec::new("get")
                                .selector([254, 74, 68, 37])
                                .mutates(false)
                                .args(vec![])
                                .docs(vec![
                                    "Returns the internal counter.",
                                ])
                                .returns(
                                    ink_abi::ReturnTypeSpec::new(
                                        ink_abi::TypeSpec::with_name_segs::<u32, _>(
                                            vec!["u32"].into_iter().map(AsRef::as_ref)
                                        )
                                    )
                                )
                                .done(),
                            ink_abi::MessageSpec::new("compare")
                                .selector([21, 176, 197, 12])
                                .mutates(false)
                                .args(vec![
                                    ink_abi::MessageParamSpec::new("x")
                                        .of_type(
                                            ink_abi::TypeSpec::with_name_segs::<u32, _>(
                                                vec!["u32"].into_iter().map(AsRef::as_ref)
                                            )
                                        )
                                        .done(),
                                ])
                                .docs(vec![
                                    "Returns `true` if `x` is greater than the internal value.",
                                ])
                                .returns(
                                    ink_abi::ReturnTypeSpec::new(
                                        ink_abi::TypeSpec::with_name_segs::<bool, _>(
                                            vec!["bool"].into_iter().map(AsRef::as_ref)
                                        )
                                    )
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

            #[cfg(feature = "ink-as-dependency")]
            mod as_dependency {
                use super::*;

                /// A simple contract that has a value that can be
                /// incremented, returned and compared.
                #[derive(Clone, scale::Encode, scale::Decode)]
                #[cfg_attr(feature = "ink-generate-abi", derive(type_metadata::Metadata))]
                pub struct Incrementer {
                    account_id: AccountId,
                }

                impl ink_core::storage::Flush for Incrementer {}

                /// Allows to enhance calls to `&self` contract messages.
                pub struct CallEnhancer<'a> {
                    contract: &'a Incrementer,
                }

                /// Allows to enhance calls to `&mut self` contract messages.
                pub struct CallEnhancerMut<'a> {
                    contract: &'a mut Incrementer,
                }

                impl ink_core::env::FromAccountId<Env> for Incrementer {
                    fn from_account_id(account_id: AccountId) -> Self {
                        Self { account_id }
                    }
                }

                impl Incrementer {
                    /// Automatically called when the contract is deployed.
                    pub fn new(code_hash: Hash, init_value: u32,) -> ink_core::env::CreateBuilder<Env, Self> {
                        ink_core::env::CreateBuilder::<Env, Self>::new(code_hash)
                            .push_arg(&init_value)
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

                impl Incrementer {
                    /// Increments the internal counter.
                    pub fn inc(&mut self, by: u32,) {
                        self
                            .call_mut()
                            .inc(by,)
                            .fire()
                            .expect(
                                concat!(
                                    "invocation of ",
                                    stringify!(Incrementer),
                                    "::",
                                    stringify!(inc),
                                    " message was invalid"
                                )
                            )
                    }

                    /// Returns the internal counter.
                    pub fn get(&self,) -> u32 {
                        self
                            .call()
                            .get()
                            .fire()
                            .expect(
                                concat!(
                                    "evaluation of ",
                                    stringify!(Incrementer),
                                    "::",
                                    stringify!(get),
                                    " message was invalid"
                                )
                            )
                    }

                    /// Returns `true` if `x` is greater than the internal value.
                    pub fn compare(&self, x: u32,) -> bool {
                        self
                            .call()
                            .compare(x,)
                            .fire()
                            .expect(
                                concat!(
                                    "evaluation of ",
                                    stringify!(Incrementer),
                                    "::",
                                    stringify!(compare),
                                    " message was invalid"
                                )
                            )
                    }
                }

                impl<'a> CallEnhancer<'a> {
                    /// Returns the internal counter.
                    pub fn get(self,) -> ink_core::env::CallBuilder<Env, ink_core::env::ReturnType<u32>> {
                        ink_core::env::CallBuilder::eval(
                            self.contract.account_id.clone(), [254, 74, 68, 37]
                        )
                    }

                    /// Returns `true` if `x` is greater than the internal value.
                    pub fn compare(self, x: u32,) -> ink_core::env::CallBuilder<Env, ink_core::env::ReturnType<bool>> {
                        ink_core::env::CallBuilder::eval(
                            self
                                .contract
                                .account_id
                                .clone(),
                            [21, 176, 197, 12]
                        ).push_arg(&x)
                    }
                }

                impl<'a> CallEnhancerMut<'a> {
                    /// Increments the internal counter.
                    pub fn inc(self, by: u32,) -> ink_core::env::CallBuilder<Env, ()> {
                        ink_core::env::CallBuilder::<Env, ()>::invoke(
                            self.contract.account_id.clone(), [15, 89, 208, 231]
                        ).push_arg(&by)
                    }
                }
            }

            #[cfg(feature = "ink-as-dependency")]
            pub use as_dependency::{Incrementer, CallEnhancer, CallEnhancerMut,};
        },
    )
}
