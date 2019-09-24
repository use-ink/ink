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
        property,
        srml::ext,
        BuildCall,
        BuildCreate,
        BuildEvent,
        EnlargeTo,
        Env,
        EnvTypes,
        Error,
        GetProperty,
        Reset,
        Result,
        SetProperty,
    },
    storage::Key,
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
    fn get_property_impl<P, I>(buffer: &mut I, ext_fn: fn()) -> Result<P::In>
    where
        P: property::ReadProperty,
        I: scale::Input + AsMut<[u8]> + EnlargeTo,
    {
        ext_fn();
        let req_len = ext::scratch_size();
        buffer.enlarge_to(req_len as usize);
        let ret = ext::scratch_read(&mut buffer.as_mut()[0..req_len], 0);
        if !ret.is_success() {
            return Err(Error::InvalidPropertyRead)
        }
        Decode::decode(&mut &buffer.as_mut()[0..req_len]).map_err(Into::into)
    }
}

macro_rules! impl_get_property_for {
    ( $( ($name:ident use $sys_fn:expr), )* ) => {
        $(
            impl<T> GetProperty<property::$name<Self>> for SrmlEnv<T>
            where
                T: EnvTypes,
            {
                fn get_property<I>(
                    buffer: &mut I,
                ) -> Result<<property::$name<Self> as property::ReadProperty>::In>
                where
                    I: scale::Input + AsMut<[u8]> + EnlargeTo,
                {
                    Self::get_property_impl::<property::$name::<T>, _>(buffer, $sys_fn)
                }
            }
        )*
    }
}

impl<T> GetProperty<property::Input<Self>> for SrmlEnv<T>
where
    T: EnvTypes,
{
    fn get_property<I>(
        buffer: &mut I,
    ) -> Result<<property::Input<Self> as property::ReadProperty>::In>
    where
        I: scale::Input + AsMut<[u8]> + EnlargeTo,
    {
        Self::get_property_impl::<property::Input<T>, _>(buffer, || ())
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

impl<T> SetProperty<property::RentAllowance<Self>> for SrmlEnv<T>
where
    T: EnvTypes,
{
    fn set_property<O>(
        buffer: &mut O,
        value: &<property::RentAllowance<Self> as property::WriteProperty>::Out,
    ) -> Result<()>
    where
        O: scale::Output + AsRef<[u8]> + Reset,
    {
        buffer.reset();
        value.encode_to(buffer);
        ext::set_rent_allowance(buffer.as_ref());
        Ok(())
    }
}

impl<T> SetProperty<property::Output<Self>> for SrmlEnv<T>
where
    T: EnvTypes,
{
    fn set_property<O>(
        buffer: &mut O,
        value: &<property::Output<Self> as property::WriteProperty>::Out,
    ) -> Result<()>
    where
        O: scale::Output + AsRef<[u8]> + Reset,
    {
        buffer.reset();
        value.encode_to(buffer);
        ext::scratch_write(buffer.as_ref());
        Ok(())
    }
}

impl<T> Env for SrmlEnv<T>
where
    T: EnvTypes,
{
    fn get_contract_storage<I, R>(key: Key, buffer: &mut I) -> Result<R>
    where
        I: scale::Input + AsMut<[u8]> + EnlargeTo,
        R: scale::Decode,
    {
        let ret = ext::get_storage(key.as_bytes());
        if !ret.is_success() {
            return Err(Error::InvalidStorageKey)
        }
        let req_len = ext::scratch_size();
        buffer.enlarge_to(req_len);
        let ret = ext::scratch_read(&mut buffer.as_mut()[0..req_len], 0);
        if !ret.is_success() {
            return Err(Error::InvalidStorageRead)
        }
        Decode::decode(&mut &buffer.as_mut()[0..req_len]).map_err(Into::into)
    }

    fn set_contract_storage<O, V>(key: Key, buffer: &mut O, val: &V)
    where
        O: scale::Output + AsRef<[u8]> + Reset,
        V: scale::Encode,
    {
        buffer.reset();
        val.encode_to(buffer);
        ext::set_storage(key.as_bytes(), Some(buffer.as_ref()));
    }

    fn clear_contract_storage(key: Key) {
        ext::set_storage(key.as_bytes(), None);
    }

    fn invoke_contract<O, D>(buffer: &mut O, call_data: &D) -> Result<()>
    where
        O: scale::Output + AsRef<[u8]> + Reset,
        D: BuildCall<Self>,
    {
        // First we reset the buffer to start from a clean slate.
        buffer.reset();
        // Now we encode `call_data`, `endowment` and `input_data`
        // each after one another into our buffer and remember their
        // boundaries using the guards.
        call_data.callee().encode_to(buffer);
        let callee_guard = buffer.as_ref().len();
        call_data.endowment().encode_to(buffer);
        let endowment_guard = buffer.as_ref().len();
        call_data.input_data().encode_to(buffer);
        // We now use the guards in order to split the buffer into
        // some read-only slices that each store their respective
        // encoded value and call the actual routine.
        let callee = &buffer.as_ref()[0..callee_guard];
        let endowment = &buffer.as_ref()[callee_guard..endowment_guard];
        let gas_limit = call_data.gas_limit();
        let call_data = &buffer.as_ref()[endowment_guard..];
        let ret = ext::call(callee, gas_limit, endowment, call_data);
        if !ret.is_success() {
            // Maybe the called contract trapped.
            return Err(Error::InvalidContractCall)
        }
        Ok(())
    }

    fn eval_contract<IO, D, R>(buffer: &mut IO, call_data: &D) -> Result<R>
    where
        IO: scale::Input + scale::Output + AsRef<[u8]> + AsMut<[u8]> + EnlargeTo + Reset,
        R: scale::Decode,
        D: BuildCall<Self>,
    {
        Self::invoke_contract(buffer, call_data)?;
        // At this point our call was successful and we can now fetch
        // the returned data and decode it for the result value.
        let req_len = ext::scratch_size();
        buffer.enlarge_to(req_len);
        let ret = ext::scratch_read(buffer.as_mut(), 0);
        if !ret.is_success() {
            return Err(Error::InvalidContractCallReturn)
        }
        Decode::decode(&mut &buffer.as_mut()[..]).map_err(Into::into)
    }

    fn create_contract<D>(create_data: &D) -> Result<Self::AccountId>
    where
        D: BuildCreate<Self>,
    {
        unimplemented!()
    }

    fn emit_event<D>(event_data: &D) -> Result<()>
    where
        D: BuildEvent<Self>,
    {
        unimplemented!()
    }

    fn invoke_runtime<V>(call_data: &V) -> Result<()>
    where
        V: scale::Encode,
    {
        unimplemented!()
    }

    fn random<I>(mut buffer: I, subject: &[u8]) -> Result<Self::Hash>
    where
        I: scale::Input + AsMut<[u8]> + EnlargeTo,
    {
        ext::random_seed(subject);
        let req_len = ext::scratch_size();
        buffer.enlarge_to(req_len);
        let ret = ext::scratch_read(&mut buffer.as_mut(), 0);
        Decode::decode(&mut &buffer.as_mut()[..]).map_err(Into::into)
    }

    fn println(content: &str) {
        ext::println(content)
    }
}
