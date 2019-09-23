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

use crate::env2::{
    property,
    srml::ext,
    Env,
    EnvTypes,
    Error,
    GetProperty,
    EnlargeTo,
    Result,
    SetProperty,
};
use core::marker::PhantomData;
use scale::{
    Decode,
    Encode,
};

/// The SRML contract environment.
pub struct SrmlEnv<T> {
    marker: PhantomData<fn() -> T>,
}

impl<T> EnvTypes for SrmlEnv<T>
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

impl<T> SrmlEnv<T>
where
    T: EnvTypes,
{
    fn get_property_impl<P, I>(
        mut buffer: I,
        ext_fn: fn(),
    ) -> Result<P::In>
    where
        P: property::ReadProperty,
        I: scale::Input + AsMut<[u8]> + EnlargeTo,
    {
        ext_fn();
        let req_len = ext::scratch_size();
        buffer.enlarge_to(req_len as usize);
        ext::scratch_read(&mut buffer.as_mut()[0..req_len], 0);
        Decode::decode(&mut &buffer.as_mut()[0..req_len]).map_err(Into::into)
    }
}

macro_rules! impl_get_property_for {
    ( $( ($name:ident use $sys_fn:expr), )* ) => {
        $(
            impl<T> GetProperty<property::$name<T>> for SrmlEnv<T>
            where
                T: EnvTypes,
            {
                fn get_property<I>(
                    buffer: I,
                ) -> Result<<property::$name<T> as property::ReadProperty>::In>
                where
                    I: scale::Input + AsMut<[u8]> + EnlargeTo,
                {
                    Self::get_property_impl::<property::$name::<T>, _>(buffer, $sys_fn)
                }
            }
        )*
    }
}

impl<T> GetProperty<property::Input<T>> for SrmlEnv<T>
where
    T: EnvTypes,
{
    fn get_property<I>(
        buffer: I,
    ) -> Result<<property::Input<T> as property::ReadProperty>::In>
    where
        I: scale::Input + AsMut<[u8]> + EnlargeTo,
    {
        Self::get_property_impl::<property::Input::<T>, _>(buffer, ||())
    }
}

impl_get_property_for! {
    (Caller use ext::caller),
    (TransferredBalance use ext::value_transferred),
    (GasPrice use ext::gas_price),
    (GasLeft use ext::gas_left),
    (NowInMs use ext::now),
    (AccountId use ext::address),
    (Balance use ext::balance),
    (RentAllowance use ext::rent_allowance),
    (BlockNumber use ext::block_number),
    (MinimumBalance use ext::minimum_balance),
}

impl<T> SetProperty<property::RentAllowance<T>> for SrmlEnv<T>
where
    T: EnvTypes,
{
    fn set_property<O>(
        mut buffer: O,
        value: &<property::RentAllowance<T> as property::WriteProperty>::Out,
    ) -> Result<()>
    where
        O: scale::Output + AsRef<[u8]>,
    {
        value.encode_to(&mut buffer);
        ext::set_rent_allowance(buffer.as_ref());
        Ok(())
    }
}

impl<T> SetProperty<property::Output<T>> for SrmlEnv<T>
where
    T: EnvTypes,
{
    fn set_property<O>(
        mut buffer: O,
        value: &<property::Output<T> as property::WriteProperty>::Out,
    ) -> Result<()>
    where
        O: scale::Output + AsRef<[u8]>,
    {
        value.encode_to(&mut buffer);
        ext::scratch_write(buffer.as_ref());
        Ok(())
    }
}
