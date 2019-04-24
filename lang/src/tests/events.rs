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
fn incrementer_contract() {
    assert_eq_tokenstreams(
        quote! {
            /// Tests emitting of custom defined events.
            struct CallCounter {
                /// A simple counter for the calls.
                count: storage::Value<u32>,
            }

            impl Deploy for CallCounter {
                fn deploy(&mut self) {
                }
            }

            event IncCalled { current: u32 }
            event DecCalled { current: u32 }

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
            ink_model::state! {
                /// Tests emitting of custom defined events.
                pub struct CallCounter {
                    /// A simple counter for the calls.
                    count: storage::Value<u32>,
                }
            }

            use ink_model::messages;

            ink_model::messages! {
                /// Increments the internal counter.
                ///
                /// # Note
                ///
                /// Also emits an event.
                257544423 => Inc();
                /// Decrements the internal counter.
                ///
                /// # Note
                ///
                /// Also emits an event.
                1772705147 => Dec();
            }

            impl CallCounter {
                pub fn deploy(&mut self, env: &mut ink_model::EnvHandler) {}

                /// Increments the internal counter.
                ///
                /// # Note
                ///
                /// Also emits an event.
                pub fn inc(&mut self, env: &mut ink_model::EnvHandler) {
                    self.value += 1;
                    env.emit(IncCalled { current: *self.value });
                }

                /// Decrements the internal counter.
                ///
                /// # Note
                ///
                /// Also emits an event.
                pub fn dec(&mut self, env: &mut ink_model::EnvHandler) {
                    self.value -= 1;
                    env.emit(DecCalled { current: *self.value });
                }
            }

            use ink_model::Contract as _;

            #[cfg(not(test))]
            impl CallCounter {
                pub(crate) fn instantiate() -> impl ink_model::Contract {
                    ink_model::ContractDecl::using::<Self>()
                        .on_deploy(|env, ()| {
                            let (handler, state) = env.split_mut();
                            state.deploy(handler,)
                        })
                        .on_msg_mut::<Inc>(|env, _| {
                            let (handler, state) = env.split_mut();
                            state.inc(handler,)
                        })
                        .on_msg_mut::<Dec>(|env, _| {
                            let (handler, state) = env.split_mut();
                            state.dec(handler,)
                        })
                        .instantiate()
                }
            }

            #[cfg(not(test))] #[no_mangle] fn deploy() { CallCounter::instantiate().deploy() }
            #[cfg(not(test))] #[no_mangle] fn call() { CallCounter::instantiate().dispatch() }

            mod events {
                use super::*;

                mod private {
                    use super::*;

                    #[doc(hidden)]
                    #[derive(parity_codec::Encode, parity_codec::Decode)]
                    pub enum Event {
                        DecCalled(DecCalled),
                        IncCalled(IncCalled),
                    }

                    /// Used to seal the emit trait.
                    pub trait Sealed { }
                }

                /// The documentation for `BalanceChanged`.
                #[derive(parity_codec::Encode, parity_codec::Decode)]
                pub struct DecCalled {
                    pub current: u32,
                }

                impl From<DecCalled> for private::Event {
                    fn from(event: DecCalled) -> Self {
                        private::Event::DecCalled(event)
                    }
                }

                /// The documentation for `BalanceChanged`.
                #[derive(parity_codec::Encode, parity_codec::Decode)]
                pub struct IncCalled {
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
                        use parity_codec::Encode as _;
                        ink_core::env::deposit_raw_event(
                            event.into().encode().as_slice()
                        )
                    }
                }

                impl EmitEventExt for ink_model::EnvHandler { }
                impl private::Sealed for ink_model::EnvHandler { }
            }

            use events::{
                EmitEventExt as _,
                DecCalled,
                IncCalled,
            };

            #[cfg(test)]
            mod test {
                use super::*;

                pub struct TestableCallCounter {
                    env: ink_model::ExecutionEnv<CallCounter>,
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
        },
    )
}
