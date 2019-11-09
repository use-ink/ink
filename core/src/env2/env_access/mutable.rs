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
        call::{
            CallData,
            CallParams,
            CreateParams,
            ReturnType,
        },
        property,
        Env,
        EnvTypes,
        GetProperty,
        Result,
        SetProperty,
        Topics,
    },
    memory::vec::Vec,
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
use core::marker::PhantomData;

#[cfg_attr(feature = "ink-generate-abi", derive(type_metadata::Metadata))]
#[derive(Debug)]
/// A wrapper around environments to make accessing them more efficient.
pub struct EnvAccessMut<E> {
    /// The wrapped environment to access.
    env: PhantomData<E>,
    /// A buffer to make environment accesses
    ///  more efficient by avoiding allocations.
    buffer: Vec<u8>,
    /// False as long as there has been no interaction between
    /// the executed contract and the environment.
    ///
    /// This flag is used to check at runtime if the environment
    /// is used correctly in respect to accessing its input.
    has_interacted: bool,
    /// True as long as the return value has not yet been set.
    ///
    /// This flag is used to check at runtime if the environment
    /// is used correctly in respect to returning its value.
    has_returned_value: bool,
}

impl<E> AllocateUsing for EnvAccessMut<E> {
    #[inline]
    unsafe fn allocate_using<A>(_alloc: &mut A) -> Self
    where
        A: Allocate,
    {
        Self::default()
    }
}

impl<E> Flush for EnvAccessMut<E> {
    #[inline]
    fn flush(&mut self) {}
}

impl<E> Initialize for EnvAccessMut<E> {
    type Args = ();

    #[inline(always)]
    fn initialize(&mut self, _args: Self::Args) {}
}

impl<E> Default for EnvAccessMut<E> {
    fn default() -> Self {
        Self {
            env: Default::default(),
            buffer: Default::default(),
            has_interacted: false,
            has_returned_value: false,
        }
    }
}

impl<T> EnvTypes for EnvAccessMut<T>
where
    T: EnvTypes,
{
    /// The type of an address.
    type AccountId = T::AccountId;
    /// The type of balances.
    type Balance = T::Balance;
    /// The type of hash.
    type Hash = T::Hash;
    /// The type of timestamps.
    type Moment = T::Moment;
    /// The type of block number.
    type BlockNumber = T::BlockNumber;
    /// The type of a call into the runtime
    type Call = T::Call;
}

macro_rules! impl_get_property_for {
    (
        $( #[$meta:meta] )*
        fn $fn_name:ident< $prop_name:ident >() -> $ret:ty; $($tt:tt)*
    ) => {
        $( #[$meta] )*
        pub fn $fn_name(&mut self) -> $ret {
            self.assert_not_yet_returned();
            self.set_has_interacted();
            <T as GetProperty<property::$prop_name<T>>>::get_property(&mut self.buffer)
        }

        impl_get_property_for!($($tt)*);
    };
    () => {}
}

/// Allow emitting generic events.
pub trait EmitEvent<T>
where
    T: Env,
{
    /// Emits an event with the given event data.
    fn emit_event<Event>(&mut self, event: Event)
    where
        Event: Topics<T> + scale::Encode;
}

impl<T> EmitEvent<T> for EnvAccessMut<T>
where
    T: Env,
{
    /// Emits an event with the given event data.
    fn emit_event<Event>(&mut self, event: Event)
    where
        Event: Topics<T> + scale::Encode,
    {
        T::emit_event(&mut self.buffer, event)
    }
}

impl<T> EnvAccessMut<T>
where
    T: Env,
{
    /// Asserts that no value has been returned yet by the contract execution.
    fn assert_not_yet_returned(&self) {
        assert!(!self.has_returned_value)
    }

    /// Sets the flag for recording interaction between executed contract
    /// and environment to `true`.
    fn set_has_interacted(&mut self) {
        self.has_interacted = true;
    }

    impl_get_property_for! {
        /// Returns the address of the caller of the executed contract.
        fn caller<Caller>() -> T::AccountId;
        /// Returns the transferred balance for the contract execution.
        fn transferred_balance<TransferredBalance>() -> T::Balance;
        /// Returns the current price for gas.
        fn gas_price<GasPrice>() -> T::Balance;
        /// Returns the amount of gas left for the contract execution.
        fn gas_left<GasLeft>() -> T::Balance;
        /// Returns the current block time in milliseconds.
        fn now_in_ms<NowInMs>() -> T::Moment;
        /// Returns the address of the executed contract.
        fn address<Address>() -> T::AccountId;
        /// Returns the balance of the executed contract.
        fn balance<Balance>() -> T::Balance;
        /// Returns the current rent allowance for the executed contract.
        fn rent_allowance<RentAllowance>() -> T::Balance;
        /// Returns the current block number.
        fn block_number<BlockNumber>() -> T::BlockNumber;
        /// Returns the minimum balance of the executed contract.
        fn minimum_balance<MinimumBalance>() -> T::Balance;
    }

    /// Sets the rent allowance of the executed contract to the new value.
    pub fn set_rent_allowance(&mut self, new_value: T::Balance) {
        self.assert_not_yet_returned();
        self.set_has_interacted();
        <T as SetProperty<property::RentAllowance<T>>>::set_property(
            &mut self.buffer,
            &new_value,
        )
    }

    /// Writes the value to the contract storage under the given key.
    pub fn set_contract_storage<V>(&mut self, key: Key, value: &V)
    where
        V: scale::Encode,
    {
        T::set_contract_storage(&mut self.buffer, key, value)
    }

    /// Returns the value stored under the given key in the contract's storage.
    ///
    /// # Errors
    ///
    /// - If the key's entry is empty
    /// - If the decoding of the typed value failed
    pub fn get_contract_storage<R>(&mut self, key: Key) -> Result<R>
    where
        R: scale::Decode,
    {
        T::get_contract_storage(&mut self.buffer, key)
    }

    /// Clears the contract's storage key entry.
    pub fn clear_contract_storage(&mut self, key: Key) {
        T::clear_contract_storage(key)
    }

    /// Invokes a contract message.
    ///
    /// # Errors
    ///
    /// If the called contract has trapped.
    pub fn invoke_contract(&mut self, call_data: &CallParams<T, ()>) -> Result<()> {
        T::invoke_contract(&mut self.buffer, call_data)
    }

    /// Evaluates a contract message and returns its result.
    ///
    /// # Errors
    ///
    /// - If the called contract traps.
    /// - If the account ID is invalid.
    /// - If given too few endowment.
    /// - If arguments passed to the called contract are invalid.
    /// - If the called contract runs out of gas.
    pub fn eval_contract<R>(
        &mut self,
        call_data: &CallParams<T, ReturnType<R>>,
    ) -> Result<R>
    where
        R: scale::Decode,
    {
        T::eval_contract(&mut self.buffer, call_data)
    }

    /// Instantiates another contract.
    ///
    /// # Errors
    ///
    /// - If the instantiation process traps.
    /// - If the code hash is invalid.
    /// - If given too few endowment.
    /// - If the instantiation process runs out of gas.
    pub fn create_contract<C>(
        &mut self,
        params: &CreateParams<T, C>,
    ) -> Result<T::AccountId> {
        T::create_contract(&mut self.buffer, params)
    }

    /// Returns the input to the executed contract.
    ///
    /// # Note
    ///
    /// - The input is the 4-bytes selector followed by the arguments
    ///   of the called function in their SCALE encoded representation.
    /// - This property must be received as the first action an executed
    ///   contract to its environment and can only be queried once.
    ///   The environment access asserts this guarantee.
    pub fn input(&mut self) -> CallData {
        assert!(!self.has_interacted);
        self.assert_not_yet_returned();
        self.set_has_interacted();
        <T as GetProperty<property::Input<T>>>::get_property(&mut self.buffer)
    }

    /// Returns the value back to the caller of the executed contract.
    ///
    /// # Note
    ///
    /// The setting of this property must be the last interaction between
    /// the executed contract and its environment.
    /// The environment access asserts this guarantee.
    pub fn output<R>(&mut self, return_value: &R)
    where
        R: scale::Encode,
    {
        self.assert_not_yet_returned();
        self.set_has_interacted();
        self.has_returned_value = true;
        T::output(&mut self.buffer, &return_value);
    }

    /// Returns a random hash.
    ///
    /// # Note
    ///
    /// The subject buffer can be used to further randomize the hash.
    pub fn random(&mut self, subject: &[u8]) -> T::Hash {
        self.assert_not_yet_returned();
        self.set_has_interacted();
        T::random(&mut self.buffer, subject)
    }

    /// Prints the given contents to the environmental log.
    pub fn println(&mut self, content: &str) {
        T::println(content)
    }
}
