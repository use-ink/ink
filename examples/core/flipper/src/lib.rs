#![feature(proc_macro_hygiene)]
#![cfg_attr(not(feature = "std"), no_std)]

use ink_core::storage;

type Env = ink_core::env2::EnvImpl<ink_core::env2::DefaultSrmlTypes>;
// type AccountId = <DefaultSrmlTypes as EnvTypes>::AccountId;
// type Balance = <DefaultSrmlTypes as EnvTypes>::Balance;
// type Hash = <DefaultSrmlTypes as EnvTypes>::Hash;
// type Moment = <DefaultSrmlTypes as EnvTypes>::Moment;
// type BlockNumber = <DefaultSrmlTypes as EnvTypes>::BlockNumber;

#[doc(hidden)]
mod __ink_storage {
    use super::*;

    pub struct StorageAndEnv<E> {
        storage: Flipper,
        env: E,
    }

    #[cfg(feature = "ink-dyn-alloc")]
    pub type FlipperAndEnvMut =
        StorageAndEnv<ink_core::env2::DynEnv<ink_core::env2::EnvAccessMut<Env>>>;

    #[cfg(feature = "ink-dyn-alloc")]
    pub type FlipperAndEnv =
        StorageAndEnv<ink_core::env2::DynEnv<ink_core::env2::EnvAccess<Env>>>;

    #[cfg(not(feature = "ink-dyn-alloc"))]
    pub type FlipperAndEnvMut = StorageAndEnv<ink_core::env2::EnvAccessMut<Env>>;

    #[cfg(not(feature = "ink-dyn-alloc"))]
    pub type FlipperAndEnv = StorageAndEnv<ink_core::env2::EnvAccess<Env>>;

    impl<E> core::ops::Deref for StorageAndEnv<E> {
        type Target = Flipper;

        fn deref(&self) -> &Self::Target {
            &self.storage
        }
    }

    impl<E> core::ops::DerefMut for StorageAndEnv<E> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.storage
        }
    }

    impl From<FlipperAndEnvMut> for FlipperAndEnv {
        fn from(flipper: FlipperAndEnvMut) -> Self {
            Self {
                storage: flipper.storage,
                env: flipper.env.into(),
            }
        }
    }

    impl ink_core::storage::alloc::AllocateUsing for Flipper {
        unsafe fn allocate_using<A>(alloc: &mut A) -> Self
        where
            A: ink_core::storage::alloc::Allocate,
        {
            Self {
                value: ink_core::storage::alloc::AllocateUsing::allocate_using(alloc),
            }
        }
    }

    impl ink_core::storage::Flush for Flipper {
        fn flush(&mut self) {
            self.value.flush();
        }
    }

    impl ink_core::storage::alloc::Initialize for Flipper {
        type Args = ();

        fn default_value() -> Option<Self::Args> {
            Some(())
        }

        fn initialize(&mut self, _args: Self::Args) {
            self.value.try_default_initialize();
        }
    }

    impl ink_core::storage::alloc::AllocateUsing for FlipperAndEnvMut {
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

    impl ink_core::storage::Flush for FlipperAndEnvMut {
        fn flush(&mut self) {
            self.storage.flush();
            self.env.flush();
        }
    }

    impl ink_core::storage::alloc::Initialize for FlipperAndEnvMut {
        type Args = ();

        fn default_value() -> Option<Self::Args> {
            Some(())
        }

        fn initialize(&mut self, _args: Self::Args) {
            self.storage.try_default_initialize();
            self.env.try_default_initialize();
        }
    }

    impl ink_lang2::Storage for FlipperAndEnvMut {}

    #[cfg(feature = "ink-dyn-alloc")]
    impl ink_lang2::AccessEnv for FlipperAndEnvMut {
        type Target = ink_core::env2::DynEnv<ink_core::env2::EnvAccessMut<Env>>;

        #[inline]
        fn env(&self) -> &Self::Target {
            &self.env
        }
    }

    #[cfg(not(feature = "ink-dyn-alloc"))]
    impl ink_lang2::AccessEnv for FlipperAndEnvMut {
        type Target = ink_core::env2::EnvAccessMut<Env>;

        #[inline]
        fn env(&self) -> &Self::Target {
            &self.env
        }
    }

    impl ink_lang2::AccessEnvMut for FlipperAndEnvMut {
        #[inline]
        fn env_mut(&mut self) -> &mut Self::Target {
            &mut self.env
        }
    }

    #[cfg(feature = "ink-dyn-alloc")]
    impl ink_lang2::AccessEnv for FlipperAndEnv {
        type Target = ink_core::env2::DynEnv<ink_core::env2::EnvAccess<Env>>;

        #[inline]
        fn env(&self) -> &Self::Target {
            &self.env
        }
    }

    #[cfg(not(feature = "ink-dyn-alloc"))]
    impl ink_lang2::AccessEnv for FlipperAndEnv {
        type Target = ink_core::env2::EnvAccess<Env>;

        #[inline]
        fn env(&self) -> &Self::Target {
            &self.env
        }
    }

    impl FlipperAndEnvMut {
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

    impl FlipperAndEnv {
        pub fn get(&self) -> bool {
            *self.value
        }
    }
}

pub struct Flipper {
    value: storage::Value<bool>,
}

const _: () = {
    /// A concrete instance of a dispatchable message.
    pub struct Msg<S> {
        /// We need to wrap inner because of Rust's orphan rules.
        marker: core::marker::PhantomData<fn () -> S>,
    }

    pub struct Constr<S> {
        /// We need to wrap inner because of Rust's orphan rules.
        marker: core::marker::PhantomData<fn () -> S>,
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
            ink_lang2::Contract::with_storage::<(
                __ink_storage::FlipperAndEnvMut,
                __ink_storage::FlipperAndEnv,
            )>()
            .on_instantiate::<Constr<[(); 0]>>(|storage, arg| storage.new(arg))
            .on_instantiate::<Constr<[(); 1]>>(|storage, _| storage.default())
            .on_msg_mut::<Msg<[(); FLIP_ID]>>(|storage, _| storage.flip())
            .on_msg::<Msg<[(); GET_ID]>>(|storage, _| storage.get())
            .done()
            .dispatch_using_mode(mode)
        }
    }
};

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
