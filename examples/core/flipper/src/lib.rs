#![feature(proc_macro_hygiene)]
#![cfg_attr(not(feature = "std"), no_std)]

use core::mem::ManuallyDrop;
use ink_core::{
    env2::{
        call::CallData,
        DefaultSrmlTypes,
        DynEnv,
        EnvAccessMut,
        // EnvTypes,
    },
    storage::{
        self,
        alloc::{
            Allocate,
            AllocateUsing,
            BumpAlloc,
            Initialize,
        },
        Flush,
        Key,
    },
};

type Env = ink_core::env2::EnvImpl<DefaultSrmlTypes>;
// type AccountId = <DefaultSrmlTypes as EnvTypes>::AccountId;
// type Balance = <DefaultSrmlTypes as EnvTypes>::Balance;
// type Hash = <DefaultSrmlTypes as EnvTypes>::Hash;
// type Moment = <DefaultSrmlTypes as EnvTypes>::Moment;
// type BlockNumber = <DefaultSrmlTypes as EnvTypes>::BlockNumber;

pub struct Flipper {
    value: storage::Value<bool>,
    env: DynEnv<EnvAccessMut<Env>>,
}

impl Flipper {
    pub fn new(&mut self, init_value: bool) {
        self.value.set(init_value);
    }

    pub fn default(&mut self) {
        self.new(false)
    }

    pub fn flip(&mut self) {
        *self.value = !self.get();
    }

    pub fn get(&self) -> bool {
        *self.value
    }
}

impl AllocateUsing for Flipper {
    unsafe fn allocate_using<A>(alloc: &mut A) -> Self
    where
        A: Allocate,
    {
        Self {
            value: AllocateUsing::allocate_using(alloc),
            env: AllocateUsing::allocate_using(alloc),
        }
    }
}

impl Flush for Flipper {
    fn flush(&mut self) {
        self.value.flush();
        self.env.flush();
    }
}

impl Initialize for Flipper {
    type Args = ();

    fn default_value() -> Option<Self::Args> {
        Some(())
    }

    fn initialize(&mut self, _args: Self::Args) {
        self.value.try_default_initialize();
        self.env.try_default_initialize();
    }
}

const _: () = {
    impl ink_lang2::Dispatch for Flipper {
        fn dispatch(mode: ink_lang2::DispatchMode) -> ink_lang2::DispatchRetCode {
            impl ink_lang2::DerefEnv<Env> for Flipper {
                type Target = DynEnv<EnvAccessMut<Env>>;

                #[inline]
                fn env(&self) -> &Self::Target {
                    &self.env
                }

                #[inline]
                fn env_mut(&mut self) -> &mut Self::Target {
                    &mut self.env
                }
            }

            /// A concrete instance of a dispatchable message.
            pub struct Msg<S> {
                /// We need to wrap inner because of Rust's orphan rules.
                inner: ink_lang2::Msg<S>,
            }

            pub struct Constr<S> {
                /// We need to wrap inner because of Rust's orphan rules.
                inner: ink_lang2::Constr<S>,
            }

            const NEW_ID: usize = 0;

            const FLIP_ID: usize = {
                (0u32
                    + ((57u8 as u32) << 0)
                    + ((219u8 as u32) << 2)
                    + ((151u8 as u32) << 4)
                    + ((140u8 as u32) << 6)) as usize
            };

            const GET_ID: usize = {
                (0u32
                    + ((254u8 as u32) << 0)
                    + ((74u8 as u32) << 2)
                    + ((68u8 as u32) << 4)
                    + ((37u8 as u32) << 6)) as usize
            };

            impl ink_lang2::FnInput for Constr<[(); 0]> {
                type Input = bool;
            }

            impl ink_lang2::FnSelector for Constr<[(); 0]> {
                const SELECTOR: ink_core::env2::call::Selector =
                    ink_core::env2::call::Selector::from_bytes([0x00; 4]);
            }

            impl ink_lang2::Constructor for Constr<[(); 0]> {}

            impl ink_lang2::FnInput for Msg<[(); FLIP_ID]> {
                type Input = ();
            }

            impl ink_lang2::FnOutput for Msg<[(); FLIP_ID]> {
                type Output = ();
            }

            impl ink_lang2::FnSelector for Msg<[(); FLIP_ID]> {
                const SELECTOR: ink_core::env2::call::Selector =
                    ink_core::env2::call::Selector::from_bytes([57, 219, 151, 140]);
            }

            impl ink_lang2::Message for Msg<[(); FLIP_ID]> {
                const IS_MUT: bool = true;
            }

            impl ink_lang2::FnInput for Msg<[(); GET_ID]> {
                type Input = ();
            }

            impl ink_lang2::FnOutput for Msg<[(); GET_ID]> {
                type Output = bool;
            }

            impl ink_lang2::FnSelector for Msg<[(); GET_ID]> {
                const SELECTOR: ink_core::env2::call::Selector =
                    ink_core::env2::call::Selector::from_bytes([254, 74, 68, 37]);
            }

            impl ink_lang2::Message for Msg<[(); GET_ID]> {
                const IS_MUT: bool = false;
            }

            const BUMP_ALLOC_ORIGIN: [u8; 32] = [0x00; 32];
            const SUCCESS: u32 = 0;

            /// Dispatches for instantiations of the contract.
            fn dispatch_instantiate(
                flipper: &mut Flipper,
                call_data: &CallData,
            ) -> ink_lang2::DispatchResult {
                let selector = call_data.selector();
                match selector {
                    s if s
                        == <Constr<[(); NEW_ID]> as ink_lang2::FnSelector>::SELECTOR =>
                    {
                        ink_lang2::dispatch_constr::<Env, _, Constr<[(); NEW_ID]>>(
                            flipper,
                            call_data,
                            |flipper, arg| flipper.new(arg),
                        )
                    }
                    _ => Err(ink_lang2::DispatchError::UnknownInstantiateSelector),
                }
            }

            /// Dispatches for calls on the contract.
            fn dispatch_call(
                flipper: &mut Flipper,
                call_data: &CallData,
            ) -> ink_lang2::DispatchResult {
                match call_data.selector() {
                    s if s == <Msg<[(); FLIP_ID]> as ink_lang2::FnSelector>::SELECTOR => {
                        ink_lang2::dispatch_msg_mut::<Env, _, Msg<[(); FLIP_ID]>>(
                            flipper,
                            call_data,
                            |flipper, _| flipper.flip(),
                        )
                    }
                    s if s == <Msg<[(); GET_ID]> as ink_lang2::FnSelector>::SELECTOR => {
                        ink_lang2::dispatch_msg::<Env, _, Msg<[(); GET_ID]>>(
                            flipper,
                            call_data,
                            |flipper, _| flipper.get(),
                        )
                    }
                    _ => Err(ink_lang2::DispatchError::UnknownCallSelector),
                }
            }

            /// Allocates the contract on the storage.
            ///
            /// # Note
            ///
            /// Returns a `ManuallyDrop` instance because the static storage should
            /// never be dropped upon finishing a contract execution.
            fn allocate() -> ManuallyDrop<Flipper> {
                let flipper = unsafe {
                    let mut alloc = BumpAlloc::from_raw_parts(Key(BUMP_ALLOC_ORIGIN));
                    Flipper::allocate_using(&mut alloc)
                };
                ManuallyDrop::new(flipper)
            }

            let mut flipper = allocate();
            // Initialize only if we instantiate a new contract.
            if mode == ink_lang2::DispatchMode::Instantiate {
                flipper.try_default_initialize();
            }
            // Dispatch using the contract execution input.
            let call_data = flipper.env.input();
            let ret = match mode {
                ink_lang2::DispatchMode::Instantiate => {
                    dispatch_instantiate(&mut flipper, &call_data)
                }
                ink_lang2::DispatchMode::Call => dispatch_call(&mut flipper, &call_data),
            };
            ret.into()
        }
    }
};

#[cfg(not(test))]
#[no_mangle]
fn deploy() -> u32 {
    <Flipper as ink_lang2::Dispatch>::dispatch(ink_lang2::DispatchMode::Instantiate).to_u32()
}

#[cfg(not(test))]
#[no_mangle]
fn call() -> u32 {
    <Flipper as ink_lang2::Dispatch>::dispatch(ink_lang2::DispatchMode::Call).to_u32()
}
