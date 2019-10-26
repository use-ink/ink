#![feature(proc_macro_hygiene)]
#![cfg_attr(not(feature = "std"), no_std)]

use ink_core::storage;

type Env = ink_core::env2::EnvImpl<ink_core::env2::DefaultSrmlTypes>;
type AccountId =
    <ink_core::env2::DefaultSrmlTypes as ink_core::env2::EnvTypes>::AccountId;
type _Balance = <ink_core::env2::DefaultSrmlTypes as ink_core::env2::EnvTypes>::Balance;
type Hash = <ink_core::env2::DefaultSrmlTypes as ink_core::env2::EnvTypes>::Hash;
type _Moment = <ink_core::env2::DefaultSrmlTypes as ink_core::env2::EnvTypes>::Moment;
type _BlockNumber =
    <ink_core::env2::DefaultSrmlTypes as ink_core::env2::EnvTypes>::BlockNumber;

#[doc(hidden)]
mod __ink_private {
    use super::*;

    #[cfg(feature = "ink-dyn-alloc")]
    pub type UsedEnv = ink_core::env2::DynEnv<ink_core::env2::EnvAccess<Env>>;
    #[cfg(not(feature = "ink-dyn-alloc"))]
    pub type UsedEnv = ink_core::env2::EnvAccess<Env>;

    #[cfg_attr(
        feature = "ink-generate-abi",
        derive(type_metadata::Metadata, ink_abi::HasLayout)
    )]
    pub struct Storage {
        pub value: storage::Value<bool>,
    }

    #[cfg_attr(feature = "ink-generate-abi", derive(type_metadata::Metadata))]
    pub struct StorageAndEnv {
        storage: Storage,
        env: UsedEnv,
    }

    impl core::ops::Deref for StorageAndEnv {
        type Target = Storage;

        fn deref(&self) -> &Self::Target {
            &self.storage
        }
    }

    impl core::ops::DerefMut for StorageAndEnv {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.storage
        }
    }

    #[cfg(feature = "ink-dyn-alloc")]
    impl ink_lang2::AccessEnv<Env> for StorageAndEnv {
        fn access_env(&mut self) -> &mut ink_core::env2::EnvAccess<Env> {
            self.env.env_mut()
        }
    }

    #[cfg(not(feature = "ink-dyn-alloc"))]
    impl ink_lang2::AccessEnv<Env> for StorageAndEnv {
        fn access_env(&mut self) -> &mut ink_core::env2::EnvAccess<Env> {
            &mut self.env
        }
    }

    impl<'a> ink_core::env2::AccessEnv for &'a StorageAndEnv {
        type Target = <&'a UsedEnv as ink_core::env2::AccessEnv>::Target;

        fn env(self) -> Self::Target {
            ink_core::env2::AccessEnv::env(&self.env)
        }
    }

    impl<'a> ink_core::env2::AccessEnv for &'a mut StorageAndEnv {
        type Target = <&'a mut UsedEnv as ink_core::env2::AccessEnv>::Target;

        fn env(self) -> Self::Target {
            ink_core::env2::AccessEnv::env(&mut self.env)
        }
    }

    impl ink_core::storage::alloc::AllocateUsing for Storage {
        unsafe fn allocate_using<A>(alloc: &mut A) -> Self
        where
            A: ink_core::storage::alloc::Allocate,
        {
            Self {
                value: ink_core::storage::alloc::AllocateUsing::allocate_using(alloc),
            }
        }
    }

    impl ink_core::storage::Flush for Storage {
        fn flush(&mut self) {
            self.value.flush();
        }
    }

    impl ink_core::storage::alloc::Initialize for Storage {
        type Args = ();

        fn default_value() -> Option<Self::Args> {
            Some(())
        }

        fn initialize(&mut self, _args: Self::Args) {
            self.value.try_default_initialize();
        }
    }

    impl ink_core::storage::alloc::AllocateUsing for StorageAndEnv {
        unsafe fn allocate_using<A>(alloc: &mut A) -> Self
        where
            A: ink_core::storage::alloc::Allocate,
        {
            Self {
                storage: ink_core::storage::alloc::AllocateUsing::allocate_using(alloc),
                env: ink_core::storage::alloc::AllocateUsing::allocate_using(alloc),
            }
        }
    }

    impl ink_core::storage::Flush for StorageAndEnv {
        fn flush(&mut self) {
            self.storage.flush();
            self.env.flush();
        }
    }

    impl ink_core::storage::alloc::Initialize for StorageAndEnv {
        type Args = ();

        fn default_value() -> Option<Self::Args> {
            Some(())
        }

        fn initialize(&mut self, _args: Self::Args) {
            self.storage.try_default_initialize();
            self.env.try_default_initialize();
        }
    }

    impl ink_lang2::Storage for StorageAndEnv {}

    pub use __ink_events::{EmitEvent, Flipped, OwnedFlipped};

    mod __ink_events {
        use super::*;

        #[derive(scale::Encode)]
        pub struct Flipped<'a> {
            pub caller: &'a AccountId,
            pub result: bool,
        }

        impl ink_core::env2::Topics<Env> for Flipped<'_> {
            fn topics(&self) -> &'static [Hash] {
                &[
                    // ink_utils::keccak(&caller),
                    // ink_utils::keccak(&result),
                ]
            }
        }

        #[derive(scale::Encode)]
        pub struct OwnedFlipped {
            pub caller: AccountId,
            pub result: bool,
        }

        impl ink_core::env2::Topics<Env> for OwnedFlipped {
            fn topics(&self) -> &'static [Hash] {
                &[
                    // ink_utils::keccak(&caller),
                    // ink_utils::keccak(&result),
                ]
            }
        }

        #[derive(scale::Encode)]
        pub enum Event<'a> {
            Flipped(Flipped<'a>),
            OwnedFlipped(OwnedFlipped),
        }

        impl<'a> From<Flipped<'a>> for Event<'a> {
            fn from(kind: Flipped<'a>) -> Self {
                Event::Flipped(kind)
            }
        }

        impl From<OwnedFlipped> for Event<'_> {
            fn from(kind: OwnedFlipped) -> Self {
                Event::OwnedFlipped(kind)
            }
        }

        impl ink_core::env2::Topics<Env> for Event<'_> {
            fn topics(&self) -> &'static [Hash] {
                match self {
                    Event::Flipped(event) => event.topics(),
                    Event::OwnedFlipped(event) => event.topics(),
                }
            }
        }

        pub trait EmitEvent {
            fn emit<'b , E>(self, event: E)
            where
                E: Into<Event<'b>>;
        }

        impl<'a> EmitEvent for &'a mut ink_core::env2::EnvAccessMut<Env> {
            fn emit<'b, E>(self, event: E)
            where
                E: Into<Event<'b>>,
            {
                self.emit_event(event.into())
            }
        }
    }
}

use __ink_private::{Flipped, OwnedFlipped};
pub type Flipper = __ink_private::StorageAndEnv;

const _: () =
    {
        use ink_core::env2::AccessEnv as _;
        use __ink_private::EmitEvent as _;

        impl Flipper {
            pub fn new(&mut self, init_value: bool) {
                self.value.set(init_value);
            }

            pub fn default(&mut self) {
                self.new(false)
            }

            pub fn flip(&mut self) {
                let caller = self.env().caller();
                let result = !self.get();
                self.env().emit(Flipped {
                    caller: &caller,
                    result,
                });
                self.env().emit(OwnedFlipped { caller, result });
                *self.value = !self.get();
            }

            pub fn get(&self) -> bool {
                let caller = self.env().caller();
                self.env().emit(Flipped {
                    caller: &caller,
                    result: false,
                });
                self.env().emit(OwnedFlipped { caller, result: true });
                *self.value
            }
        }
    };

const _: () = {
    // A concrete instance of a dispatchable message.
    pub struct Msg<S> {
        // We need to wrap inner because of Rust's orphan rules.
        marker: core::marker::PhantomData<fn() -> S>,
    }

    pub struct Constr<S> {
        // We need to wrap inner because of Rust's orphan rules.
        marker: core::marker::PhantomData<fn() -> S>,
    }

    const NEW_ID: usize = 0;

    const DEFAULT_ID: usize = 1;

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

    impl ink_lang2::FnInput for Constr<[(); NEW_ID]> {
        type Input = bool;
    }

    impl ink_lang2::FnOutput for Constr<[(); NEW_ID]> {
        type Output = ();
    }

    impl ink_lang2::FnSelector for Constr<[(); NEW_ID]> {
        const SELECTOR: ink_core::env2::call::Selector =
            ink_core::env2::call::Selector::from_bytes([0x00; 4]);
    }

    impl ink_lang2::Message for Constr<[(); NEW_ID]> {
        const IS_MUT: bool = true;
    }

    impl ink_lang2::FnInput for Constr<[(); DEFAULT_ID]> {
        type Input = ();
    }

    impl ink_lang2::FnOutput for Constr<[(); DEFAULT_ID]> {
        type Output = ();
    }

    impl ink_lang2::FnSelector for Constr<[(); DEFAULT_ID]> {
        const SELECTOR: ink_core::env2::call::Selector =
            ink_core::env2::call::Selector::from_bytes([0x01; 4]);
    }

    impl ink_lang2::Message for Constr<[(); DEFAULT_ID]> {
        const IS_MUT: bool = true;
    }

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

    impl ink_lang2::DispatchUsingMode for Flipper {
        fn dispatch_using_mode(
            mode: ink_lang2::DispatchMode,
        ) -> core::result::Result<(), ink_lang2::DispatchError> {
            ink_lang2::Contract::with_storage::<(__ink_private::StorageAndEnv)>()
                .on_instantiate::<Constr<[(); NEW_ID]>>(|storage, arg| storage.new(arg))
                .on_instantiate::<Constr<[(); DEFAULT_ID]>>(|storage, _| {
                    storage.default()
                })
                .on_msg_mut::<Msg<[(); FLIP_ID]>>(|storage, _| storage.flip())
                .on_msg::<Msg<[(); GET_ID]>>(|storage, _| storage.get())
                .done()
                .dispatch_using_mode(mode)
        }
    }

    #[cfg(not(test))]
    #[no_mangle]
    fn deploy() -> u32 {
        ink_lang2::DispatchRetCode::from(
            <Flipper as ink_lang2::DispatchUsingMode>::dispatch_using_mode(
                ink_lang2::DispatchMode::Instantiate,
            ),
        )
        .to_u32()
    }

    #[cfg(not(test))]
    #[no_mangle]
    fn call() -> u32 {
        ink_lang2::DispatchRetCode::from(
            <Flipper as ink_lang2::DispatchUsingMode>::dispatch_using_mode(
                ink_lang2::DispatchMode::Call,
            ),
        )
        .to_u32()
    }
};

#[cfg(feature = "ink-generate-abi")]
const _: () = {
    impl ink_lang2::GenerateAbi for Flipper {
        fn generate_abi() -> ink_abi::InkProject {
            let contract: ink_abi::ContractSpec = {
                ink_abi::ContractSpec::new("Flipper")
                    .constructors(vec![
                        ink_abi::ConstructorSpec::new("new")
                            .selector([0x00; 4])
                            .args(vec![ink_abi::MessageParamSpec::new("init_value")
                                .of_type(ink_abi::TypeSpec::with_name_segs::<u32, _>(
                                    vec!["u32"].into_iter().map(AsRef::as_ref),
                                ))
                                .done()])
                            .docs(vec![])
                            .done(),
                        ink_abi::ConstructorSpec::new("default")
                            .selector([0x00, 0x00, 0x00, 0x01])
                            .args(vec![])
                            .docs(vec![])
                            .done(),
                    ])
                    .messages(vec![
                        ink_abi::MessageSpec::new("flip")
                            .selector([140u8, 151u8, 219u8, 57u8])
                            .mutates(true)
                            .args(vec![])
                            .docs(vec!["Flips the current state of our smart contract."])
                            .returns(ink_abi::ReturnTypeSpec::new(None))
                            .done(),
                        ink_abi::MessageSpec::new("get")
                            .selector([37u8, 68u8, 74u8, 254u8])
                            .mutates(false)
                            .args(vec![])
                            .docs(vec!["Returns the current state."])
                            .returns(ink_abi::ReturnTypeSpec::new(
                                ink_abi::TypeSpec::with_name_segs::<bool, _>(
                                    vec!["bool"].into_iter().map(AsRef::as_ref),
                                ),
                            ))
                            .done(),
                    ])
                    .events(vec![])
                    .docs(vec![])
                    .done()
            };
            let layout: ink_abi::StorageLayout = {
                unsafe {
                    use ink_abi::HasLayout as _;
                    use ink_core::storage::alloc::AllocateUsing as _;
                    core::mem::ManuallyDrop::new(Flipper::allocate_using(
                        &mut ink_core::storage::alloc::BumpAlloc::from_raw_parts(
                            ink_core::storage::Key([0x0; 32]),
                        ),
                    ))
                    .layout()
                }
            };
            ink_abi::InkProject::new(layout, contract)
        }
    }
};
