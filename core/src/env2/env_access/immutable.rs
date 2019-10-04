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

use crate::{
    env2::{
        call::CallData,
        CallParams,
        CreateParams,
        Env,
        EnvAccessMut,
        Result,
    },
    storage::Key,
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
pub struct EnvAccess<T> {
    /// Allows accessing the inner environment by `&self` instead of `&mut self`.
    ///
    /// This is important to make `DynEnv` work also in conjunction with `&self` messages.
    access: RefCell<EnvAccessMut<T>>,
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
        fn invoke_contract<D>(&self, call_data: &D) -> Result<()>
        where
            D: CallParams<T>;

        /// Evaluates a contract message and returns its result.
        ///
        /// # Errors
        ///
        /// - If the called contract traps.
        /// - If the account ID is invalid.
        /// - If given too little endowment.
        /// - If arguments passed to the called contract are invalid.
        /// - If the called contract runs out of gas.
        fn eval_contract<D, R>(&self, call_data: &D) -> Result<R>
        where
            D: CallParams<T>,
            R: scale::Decode;

        /// Instantiates another contract.
        ///
        /// # Errors
        ///
        /// - If the instantiation process traps.
        /// - If the code hash is invalid.
        /// - If given too little endowment.
        /// - If the instantiation process runs out of gas.
        fn create_contract<D>(&self, create_data: &D) -> Result<T::AccountId>
        where
            D: CreateParams<T>;

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
