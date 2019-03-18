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
                struct Incrementer {
                    /// The internal value.
                    value: storage::Value<u32>,
                }
            }

            use pdsl_model::messages;

            pdsl_model::messages! {
                /// Increments the internal counter.
                0 => Inc(by: u32);
                /// Returns the internal counter.
                1 => Get() -> u32;
                /// Returns `true` if `x` is greater than the internal value.
                2 => Compare(x: u32) -> bool;
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

            impl Incrementer {}
            use pdsl_model::Contract;

            fn instantiate() -> impl pdsl_model::Contract {
                pdsl_model::ContractDecl::using::<Incrementer>()
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

            #[no_mangle] fn deploy() { instantiate().deploy() }
            #[no_mangle] fn call() { instantiate().dispatch() }
        },
    )
}
