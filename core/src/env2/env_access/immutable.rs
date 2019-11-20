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

use crate::{
    env2::{
        call::{
            CallData,
            CallParams,
            CreateParams,
            ReturnType,
        },
        env_access::EmitEvent as _,
        AccessEnv,
        Env,
        EnvAccessMut,
        Result,
        Topics,
    },
    storage::{
        alloc::{
            Allocate,
            AllocateUsing,
            Initialize,
        },
        Flush,
        Key,
    },
};
use core::cell::RefCell;

/// A `&self` accessor to `EnvAccessMut`.
///
/// This allows ink! `&self` messages to make use of the environment efficiently
/// while also maintaining access invariants through runtime checks.
/// A wrapper arround `EnvAccessMut` allowing for `&self` accesses to make it
/// usable in `&self` ink! messages.
///
/// # Note
///
/// Using `EnvAccessMut` is preferable since it performs these access checks at
/// compile-time.
pub struct EnvAccess<E> {
    /// Allows accessing the inner environment by `&self` instead of `&mut self`.
    ///
    /// This is important to make `DynEnv` work also in conjunction with `&self` messages.
    pub(crate) access: RefCell<EnvAccessMut<E>>,
}

impl<E> core::fmt::Debug for EnvAccess<E> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.debug_struct("EnvAccess").finish()
    }
}

#[cfg(feature = "ink-generate-abi")]
impl<E> type_metadata::HasTypeId for EnvAccess<E>
where
    E: type_metadata::Metadata,
{
    fn type_id() -> type_metadata::TypeId {
        type_metadata::TypeIdCustom::new(
            "EnvAccess",
            type_metadata::Namespace::from_module_path(module_path!())
                .expect("namespace from module path cannot fail"),
            vec![E::meta_type()],
        )
        .into()
    }
}

#[cfg(feature = "ink-generate-abi")]
impl<E> type_metadata::HasTypeDef for EnvAccess<E>
where
    crate::env2::EnvAccessMut<E>: type_metadata::Metadata,
{
    fn type_def() -> type_metadata::TypeDef {
        type_metadata::TypeDefStruct::new(vec![type_metadata::NamedField::new(
            "access",
            <crate::env2::EnvAccessMut<E> as type_metadata::Metadata>::meta_type(),
        )])
        .into()
    }
}

impl<'a, E> AccessEnv for &'a EnvAccess<E> {
    type Target = core::cell::RefMut<'a, EnvAccessMut<E>>;

    #[inline]
    fn env(self) -> Self::Target {
        self.access.borrow_mut()
    }
}

impl<'a, E> AccessEnv for &'a mut EnvAccess<E> {
    type Target = &'a mut EnvAccessMut<E>;

    #[inline]
    fn env(self) -> Self::Target {
        self.access.get_mut()
    }
}

impl<E> AllocateUsing for EnvAccess<E> {
    #[inline]
    unsafe fn allocate_using<A>(_alloc: &mut A) -> Self
    where
        A: Allocate,
    {
        Self::default()
    }
}

impl<E> Flush for EnvAccess<E> {
    #[inline(always)]
    fn flush(&mut self) {}
}

impl<E> Initialize for EnvAccess<E> {
    type Args = ();

    #[inline(always)]
    fn initialize(&mut self, _args: Self::Args) {}
}

impl<T> Default for EnvAccess<T> {
    fn default() -> Self {
        Self {
            access: RefCell::new(Default::default()),
        }
    }
}

impl<T> From<EnvAccessMut<T>> for EnvAccess<T> {
    fn from(env_access_mut: EnvAccessMut<T>) -> Self {
        Self {
            access: RefCell::new(env_access_mut),
        }
    }
}

macro_rules! impl_forward_for {
    (
        $( #[$meta:meta] )*
        fn $fn_name:ident $( < $($gen_arg:ident),* > )? ( &self $(, $arg_name:ident : $arg_ty:ty )* )
        $(
            where
                $(
                    $bound_ident:ident : $bound_ty:path
                ),*
        )?
        ;

        $($tt:tt)*
    ) => {
        impl_forward_for!(
            $( #[$meta] )*
            fn $fn_name $( < $($gen_arg),* > )? ( &self $(, $arg_name : $arg_ty )* ) -> ()
            $(
                where
                    $(
                        $bound_ident : $bound_ty
                    ),*
            )?
            ;

            $($tt)*
        );
    };
    (
        $( #[$meta:meta] )*
        fn $fn_name:ident $( < $($gen_arg:ident),* > )? ( &self $(, $arg_name:ident : $arg_ty:ty )* ) -> $ret:ty
        $(
            where
                $(
                    $bound_ident:ident : $bound_ty:path
                ),*
        )?
        ;

        $($tt:tt)*
    ) => {
        $( #[$meta] )*
        pub fn $fn_name $( < $($gen_arg),* > )? (&self $(, $arg_name : $arg_ty)* ) -> $ret
        $(
            where
                $(
                    $bound_ident : $bound_ty
                ),*
        )?
        {
            self.access.borrow_mut().$fn_name( $($arg_name),* )
        }

        impl_forward_for!($($tt)*);
    };
    () => {};
}

impl<T> EnvAccess<T>
where
    T: Env,
{
    impl_forward_for! {
        /// Returns the address of the caller of the executed contract.
        fn caller(&self) -> T::AccountId;
        /// Returns the transferred balance for the contract execution.
        fn transferred_balance(&self) -> T::Balance;
        /// Returns the current price for gas.
        fn gas_price(&self) -> T::Balance;
        /// Returns the amount of gas left for the contract execution.
        fn gas_left(&self) -> T::Balance;
        /// Returns the current block time in milliseconds.
        fn now_in_ms(&self) -> T::Moment;
        /// Returns the address of the executed contract.
        fn address(&self) -> T::AccountId;
        /// Returns the balance of the executed contract.
        fn balance(&self) -> T::Balance;
        /// Returns the current rent allowance for the executed contract.
        fn rent_allowance(&self) -> T::Balance;
        /// Returns the current block number.
        fn block_number(&self) -> T::BlockNumber;
        /// Returns the minimum balance of the executed contract.
        fn minimum_balance(&self) -> T::Balance;

        /// Sets the rent allowance of the executed contract to the new value.
        fn set_rent_allowance(&self, new_value: T::Balance);

        /// Writes the value to the contract storage under the given key.
        fn set_contract_storage<V>(&self, key: Key, value: &V)
        where
            V: scale::Encode;

        /// Returns the value stored under the given key in the contract's storage.
        ///
        /// # Errors
        ///
        /// - If the key's entry is empty
        /// - If the decoding of the typed value failed
        fn get_contract_storage<R>(&self, key: Key) -> Result<R>
        where
            R: scale::Decode;

        /// Clears the contract's storage key entry.
        fn clear_contract_storage(&self, key: Key);

        /// Invokes a contract message.
        ///
        /// # Errors
        ///
        /// If the called contract has trapped.
        fn invoke_contract(&self, call_data: &CallParams<T, ()>) -> Result<()>;

        /// Evaluates a contract message and returns its result.
        ///
        /// # Errors
        ///
        /// - If the called contract traps.
        /// - If the account ID is invalid.
        /// - If given too little endowment.
        /// - If arguments passed to the called contract are invalid.
        /// - If the called contract runs out of gas.
        fn eval_contract<R>(&self, call_data: &CallParams<T, ReturnType<R>>) -> Result<R>
        where
            R: scale::Decode;

        /// Instantiates another contract.
        ///
        /// # Errors
        ///
        /// - If the instantiation process traps.
        /// - If the code hash is invalid.
        /// - If given too little endowment.
        /// - If the instantiation process runs out of gas.
        fn create_contract<C>(&self, params: &CreateParams<T, C>) -> Result<T::AccountId>;

        /// Emits an event with the given event data.
        fn emit_event<Event>(&self, event: Event)
        where
            Event: Topics<T>,
            Event: scale::Encode;

        /// Returns the input to the executed contract.
        ///
        /// # Note
        ///
        /// - The input is the 4-bytes selector followed by the arguments
        ///   of the called function in their SCALE encoded representation.
        /// - This property must be received as the first action an executed
        ///   contract to its environment and can only be queried once.
        ///   The environment access asserts this guarantee.
        fn input(&self) -> CallData;

        /// Returns the value back to the caller of the executed contract.
        ///
        /// # Note
        ///
        /// The setting of this property must be the last interaction between
        /// the executed contract and its environment.
        /// The environment access asserts this guarantee.
        fn output<R>(&self, return_value: &R)
        where
            R: scale::Encode;

        /// Returns a random hash.
        ///
        /// # Note
        ///
        /// The subject buffer can be used to further randomize the hash.
        fn random(&self, subject: &[u8]) -> T::Hash;

        /// Prints the given contents to the environmental log.
        fn println(&self, content: &str);
    }
}
