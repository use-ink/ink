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
        property,
        GetProperty,
        SetProperty,
        Env,
        EnvTypes,
    },
};
use core::marker::PhantomData;

/// A wrapper around environments to make accessing them more efficient.
pub struct EnvAccess<E> {
    /// The wrapped environment to access.
    env: PhantomData<E>,
    /// A buffer to make environment accesses
    ///  more efficient by avoiding allocations.
    buffer: Vec<u8>,
}

impl<E> Default for EnvAccess<E> {
    fn default() -> Self {
        Self {
            env: Default::default(),
            buffer: Default::default(),
        }
    }
}

impl<T> EnvTypes for EnvAccess<T>
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

impl<T> EnvAccess<T>
where
    T: Env,
{
    /// Returns the address of the caller of the executed contract.
    pub fn caller(&mut self) -> T::AccountId {
        <T as GetProperty<property::Caller<T>>>::get_property(&mut self.buffer)
            .expect("call to `ext_caller` failed")
    }

    /// Returns the transferred balance for the contract execution.
    pub fn transferred_balance(&mut self) -> T::Balance {
        <T as GetProperty<property::TransferredBalance<T>>>::get_property(&mut self.buffer)
            .expect("call to `ext_transferred_balance` failed")
    }

    /// Returns the current price for gas.
    pub fn gas_price(&mut self) -> T::Balance {
        <T as GetProperty<property::GasPrice<T>>>::get_property(&mut self.buffer)
            .expect("call to `ext_gas_price` failed")
    }

    /// Returns the amount of gas left for the contract execution.
    pub fn gas_left(&mut self) -> T::Balance {
        <T as GetProperty<property::GasLeft<T>>>::get_property(&mut self.buffer)
            .expect("call to `ext_gas_left` failed")
    }

    /// Returns the current block time in milliseconds.
    pub fn now_in_ms(&mut self) -> T::Moment {
        <T as GetProperty<property::NowInMs<T>>>::get_property(&mut self.buffer)
            .expect("call to `ext_now` failed")
    }

    /// Returns the address of the executed contract.
    pub fn address(&mut self) -> T::AccountId {
        <T as GetProperty<property::Address<T>>>::get_property(&mut self.buffer)
            .expect("call to `ext_address` failed")
    }

    /// Returns the balance of the executed contract.
    pub fn balance(&mut self) -> T::Balance {
        <T as GetProperty<property::Balance<T>>>::get_property(&mut self.buffer)
            .expect("call to `ext_balance` failed")
    }

    /// Returns the current rent allowance for the executed contract.
    pub fn rent_allowance(&mut self) -> T::Balance {
        <T as GetProperty<property::RentAllowance<T>>>::get_property(&mut self.buffer)
            .expect("call to `ext_rent_allowance` failed")
    }

    /// Sets the rent allowance of the executed contract to the new value.
    pub fn set_rent_allowance(&mut self, new_value: T::Balance) {
        <T as SetProperty<property::RentAllowance<T>>>::set_property(&mut self.buffer, &new_value)
            .expect("couldn't encode for `ext_set_rent_allowance` failed")
    }

    /// Returns the current block number.
    pub fn block_number(&mut self) -> T::BlockNumber {
        <T as GetProperty<property::BlockNumber<T>>>::get_property(&mut self.buffer)
            .expect("call to `ext_block_number` failed")
    }

    /// Returns the minimum balance of the executed contract.
    pub fn minimum_balance(&mut self) -> T::Balance {
        <T as GetProperty<property::MinimumBalance<T>>>::get_property(&mut self.buffer)
            .expect("call to `ext_minimum_balance` failed")
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
        <T as GetProperty<property::Input<T>>>::get_property(&mut self.buffer)
            .expect("call to `ext_minimum_balance` failed")
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
        T::output(&mut self.buffer, &return_value)
    }
}
