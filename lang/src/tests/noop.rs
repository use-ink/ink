use super::*;

#[test]
fn noop_contract() {
    assert_eq_tokenstreams(
        quote!{
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
        quote!{
            pdsl_model::state! {
                /// The contract that does nothing.
                ///
                /// # Note
                ///
                /// Can be deployed, cannot be called.
                struct Noop {}
            }

            use pdsl_model::messages;
            pdsl_model::messages! {}

            impl Noop {
                /// Does nothing to initialize itself.
                pub fn deploy(&mut self, env: &mut pdsl_model::EnvHandler) { }
            }

            impl Noop {}

            use pdsl_model::Contract;
            fn instantiate() -> impl pdsl_model::Contract {
                pdsl_model::ContractDecl::using::<Noop>()
                    .on_deploy(|env, ()| {
                        let (handler, state) = env.split_mut();
                        state.deploy(handler,)
                    })
                    .instantiate()
            }

            #[no_mangle] fn deploy() { instantiate().deploy() }
            #[no_mangle] fn call() { instantiate().dispatch() }
        }
    )
}
