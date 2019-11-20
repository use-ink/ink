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

            /// Tests emitting of custom defined events.
            struct CallCounter {
                /// A simple counter for the calls.
                count: storage::Value<u32>,
            }

            impl Deploy for CallCounter {
                fn deploy(&mut self) {
                }
            }

            /// Fires when the value is incremented.
            event IncCalled {
                /// The current value.
                current: u32
            }

            /// Fires when the value is decremented.
            event DecCalled {
                /// The current value.
                current: u32
            }

            impl CallCounter {
                /// Increments the internal counter.
                ///
                /// # Note
                ///
                /// Also emits an event.
                pub(external) fn inc(&mut self) {
                    self.value += 1;
                    env.emit(IncCalled { current: *self.value });
                }

                /// Decrements the internal counter.
                ///
                /// # Note
                ///
                /// Also emits an event.
                pub(external) fn dec(&mut self) {
                    self.value -= 1;
                    env.emit(DecCalled { current: *self.value });
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
                    /// Tests emitting of custom defined events.
                    #[cfg_attr(
                        feature = "ink-generate-abi",
                        derive(type_metadata::Metadata, ink_abi::HasLayout,)
                    )]
                    pub struct CallCounter {
                        /// A simple counter for the calls.
                        count: storage::Value<u32>,
                    }
                }


                mod msg {
                    use super::*;
                    use ink_model::messages;

                    ink_model::messages! {
                        /// Increments the internal counter.
                        ///
                        /// # Note
                        ///
                        /// Also emits an event.
                        [15, 89, 208, 231] => Inc();
                        /// Decrements the internal counter.
                        ///
                        /// # Note
                        ///
                        /// Also emits an event.
                        [105, 169, 85, 123] => Dec();
                    }
                }

                impl CallCounter {
                    pub fn deploy(&mut self, env: &mut ink_model::EnvHandler<ink_core::env::ContractEnv<DefaultSrmlTypes> >) {}

                    /// Increments the internal counter.
                    ///
                    /// # Note
                    ///
                    /// Also emits an event.
                    pub fn inc(&mut self, env: &mut ink_model::EnvHandler<ink_core::env::ContractEnv<DefaultSrmlTypes> >) {
                        self.value += 1;
                        env.emit(IncCalled { current: *self.value });
                    }

                    /// Decrements the internal counter.
                    ///
                    /// # Note
                    ///
                    /// Also emits an event.
                    pub fn dec(&mut self, env: &mut ink_model::EnvHandler<ink_core::env::ContractEnv<DefaultSrmlTypes> >) {
                        self.value -= 1;
                        env.emit(DecCalled { current: *self.value });
                    }
                }

                use ink_model::Contract as _;

                #[cfg(not(test))]
                impl CallCounter {
                    pub(crate) fn instantiate() -> impl ink_model::Contract {
                        ink_model::ContractDecl::using::<Self, ink_core::env::ContractEnv<DefaultSrmlTypes>>()
                            .on_deploy(|env, ()| {
                                let (handler, state) = env.split_mut();
                                state.deploy(handler,)
                            })
                            .on_msg_mut::<msg::Inc>(|env, _| {
                                let (handler, state) = env.split_mut();
                                state.inc(handler,)
                            })
                            .on_msg_mut::<msg::Dec>(|env, _| {
                                let (handler, state) = env.split_mut();
                                state.dec(handler,)
                            })
                            .instantiate()
                    }
                }

                #[cfg(not(test))] #[no_mangle] fn deploy() -> u32 { CallCounter::instantiate().deploy().to_u32() }
                #[cfg(not(test))] #[no_mangle] fn call() -> u32 { CallCounter::instantiate().dispatch().to_u32() }

                mod events {
                    use super::*;

                    mod private {
                        use super::*;

                        #[doc(hidden)]
                        #[derive(scale::Encode, scale::Decode)]
                        pub enum Event {
                            DecCalled(DecCalled),
                            IncCalled(IncCalled),
                        }

                        /// Used to seal the emit trait.
                        pub trait Sealed { }
                    }

                    #[derive(scale::Encode, scale::Decode)]
                    /// Fires when the value is decremented.
                    pub struct DecCalled {
                        /// The current value.
                        pub current: u32,
                    }

                    impl From<DecCalled> for private::Event {
                        fn from(event: DecCalled) -> Self {
                            private::Event::DecCalled(event)
                        }
                    }

                    #[derive(scale::Encode, scale::Decode)]
                    /// Fires when the value is incremented.
                    pub struct IncCalled {
                        /// The current value.
                        pub current: u32,
                    }

                    impl From<IncCalled> for private::Event {
                        fn from(event: IncCalled) -> Self {
                            private::Event::IncCalled(event)
                        }
                    }

                    pub trait EmitEventExt: private::Sealed {
                        /// Emits the given event.
                        fn emit<E>(&self, event: E)
                        where
                            E: Into<private::Event>,
                        {
                            use scale::Encode as _;
                            <ink_core::env::ContractEnv<DefaultSrmlTypes> as ink_core::env::Env>::deposit_raw_event(
                                &[], event.into().encode().as_slice()
                            )
                        }
                    }

                    impl EmitEventExt for ink_model::EnvHandler<ink_core::env::ContractEnv<DefaultSrmlTypes>> { }
                    impl private::Sealed for ink_model::EnvHandler<ink_core::env::ContractEnv<DefaultSrmlTypes>> { }
                }

                use events::{
                    EmitEventExt as _,
                    DecCalled,
                    IncCalled,
                };
            }

            #[cfg(not(feature = "ink-as-dependency"))]
            use normal::*;

            #[cfg(not(feature = "ink-as-dependency"))]
            use ink_core::env::FromAccountId as _;

            #[cfg(test)]
            mod test {
                use super::*;

                pub struct TestableCallCounter {
                    env: ink_model::ExecutionEnv<CallCounter, ink_core::env::ContractEnv<DefaultSrmlTypes>>,
                }

                impl CallCounter {
                    /// Returns a testable version of the contract.
                    pub fn deploy_mock() -> TestableCallCounter {
                        let mut mock = TestableCallCounter::allocate();
                        mock.deploy();
                        mock
                    }
                }

                impl TestableCallCounter {
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

                impl TestableCallCounter {
                    pub fn inc(&mut self) {
                        let (handler, state) = self.env.split_mut();
                        state.inc(handler,)
                    }

                    pub fn dec(&mut self) {
                        let (handler, state) = self.env.split_mut();
                        state.dec(handler,)
                    }
                }
            }

            #[cfg(not(feature = "ink-as-dependency"))]
            #[cfg(feature = "ink-generate-abi")]
            pub fn ink_generate_abi() -> ink_abi::InkProject {
                let contract = {
                    ink_abi::ContractSpec::new("CallCounter")
                        .constructors(vec![
                            ink_abi::ConstructorSpec::new("on_deploy")
                                .selector([0u8; 4])
                                .args(vec![])
                                .docs(vec![])
                                .done()
                        ])
                        .messages(vec![
                            ink_abi::MessageSpec::new("inc")
                                .selector([15, 89, 208, 231])
                                .mutates(true)
                                .args(vec![])
                                .docs(vec![
                                    "Increments the internal counter.",
                                    "",
                                    "# Note",
                                    "",
                                    "Also emits an event.",
                                ])
                                .returns(
                                    ink_abi::ReturnTypeSpec::new(None)
                                )
                                .done(),
                            ink_abi::MessageSpec::new("dec")
                                .selector([105, 169, 85, 123])
                                .mutates(true)
                                .args(vec![])
                                .docs(vec![
                                    "Decrements the internal counter.",
                                    "",
                                    "# Note",
                                    "",
                                    "Also emits an event.",
                                ])
                                .returns(ink_abi::ReturnTypeSpec::new(None))
                                .done(),
                        ])
                        .events(vec![
                            ink_abi::EventSpec::new(stringify!(DecCalled))
                                .args(vec![
                                    ink_abi::EventParamSpec::new(stringify!(current))
                                        .of_type(
                                            ink_abi::TypeSpec::with_name_segs::<u32, _>(
                                                vec!["u32"].into_iter().map(AsRef::as_ref)
                                            )
                                        )
                                        .indexed(false)
                                        .done(),
                                ])
                                .docs(vec![
                                    "Fires when the value is decremented.",
                                ])
                                .done(),
                            ink_abi::EventSpec::new(stringify!(IncCalled))
                                .args(vec![
                                    ink_abi::EventParamSpec::new(stringify!(current))
                                        .of_type(
                                            ink_abi::TypeSpec::with_name_segs::<u32, _>(
                                                vec!["u32"].into_iter().map(AsRef::as_ref)
                                            )
                                        )
                                        .indexed(false)
                                        .done(),
                                ])
                                .docs(vec![
                                    "Fires when the value is incremented.",
                                ])
                                .done(),
                        ])
                        .docs(vec![])
                        .done()
                };
                let layout = {
                    unsafe {
                        use ink_core::storage::alloc::AllocateUsing as _;
                        use ink_abi::HasLayout as _;
                        CallCounter::allocate_using(
                            &mut ink_core::storage::alloc::BumpAlloc::from_raw_parts(
                                ink_core::storage::Key([0x0; 32])))
                                .layout()
                    }
                };
                ink_abi::InkProject::new(layout, contract)
            }

            #[cfg(feature = "ink-as-dependency")]
            mod as_dependency {
                use super::*;

                /// Tests emitting of custom defined events.
                #[derive(Clone, scale::Encode, scale::Decode)]
                #[cfg_attr(feature = "ink-generate-abi", derive(type_metadata::Metadata))]
                pub struct CallCounter {
                    account_id: AccountId,
                }

                impl ink_core::storage::Flush for CallCounter {
                    fn flush(&mut self) {}
                }

                /// Allows to enhance calls to `&self` contract messages.
                pub struct CallEnhancer<'a> {
                    contract: &'a CallCounter,
                }

                /// Allows to enhance calls to `&mut self` contract messages.
                pub struct CallEnhancerMut<'a> {
                    contract: &'a mut CallCounter,
                }

                impl ink_core::env::FromAccountId<Env> for CallCounter {
                    fn from_account_id(account_id: AccountId) -> Self {
                        Self { account_id }
                    }
                }

                impl CallCounter {
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

                impl CallCounter {
                    /// Increments the internal counter.
                    ///
                    /// # Note
                    ///
                    /// Also emits an event.
                    pub fn inc(&mut self,) {
                        self
                            .call_mut()
                            .inc()
                            .fire()
                            .expect(
                                concat!(
                                    "invocation of ",
                                    stringify!(CallCounter),
                                    "::",
                                    stringify!(inc),
                                    " message was invalid"
                                )
                            )
                    }

                    /// Decrements the internal counter.
                    ///
                    /// # Note
                    ///
                    /// Also emits an event.
                    pub fn dec(&mut self,) {
                        self
                            .call_mut()
                            .dec()
                            .fire()
                            .expect(
                                concat!(
                                    "invocation of ",
                                    stringify!(CallCounter),
                                    "::",
                                    stringify!(dec),
                                    " message was invalid"
                                )
                            )
                    }
                }

                impl<'a> CallEnhancer<'a> {}

                impl<'a> CallEnhancerMut<'a> {
                    /// Increments the internal counter.
                    ///
                    /// # Note
                    ///
                    /// Also emits an event.
                    pub fn inc(self,) -> ink_core::env::CallBuilder<Env, ()> {
                        ink_core::env::CallBuilder::<Env, ()>::invoke(
                            self.contract.account_id.clone(), [15, 89, 208, 231])
                    }

                    /// Decrements the internal counter.
                    ///
                    /// # Note
                    ///
                    /// Also emits an event.
                    pub fn dec(self,) -> ink_core::env::CallBuilder<Env, ()> {
                        ink_core::env::CallBuilder::<Env, ()>::invoke(
                            self.contract.account_id.clone(), [105, 169, 85, 123])
                    }
                }
            }

            #[cfg(feature = "ink-as-dependency")]
            pub use as_dependency::{CallCounter, CallEnhancer, CallEnhancerMut,};
        },
    )
}
