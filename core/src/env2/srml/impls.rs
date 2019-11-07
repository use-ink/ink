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
            CallParams,
            CreateParams,
            ReturnType,
        },
        property,
        srml::ext,
        utils::{
            EnlargeTo,
            Reset,
        },
        Env,
        EnvTypes,
        Error,
        GetProperty,
        Result,
        SetProperty,
        Topics,
    },
    storage::Key,
};
use core::marker::PhantomData;
use scale::{
    Decode,
    Encode,
};

/// The SRML contract environment.
///
/// # Dev Notes
///
/// The implementation of environmental access routines have to be as efficient
/// as possible to not waste gas by any means. This has the effect that implementation
/// of some environmental access routines defined here are suboptimal in respect
/// to readability.
///
/// Any avoidable inefficiency is regarded by us as a bug.
/// If you spot such an inefficiency please report an issue at:
/// https://github.com/paritytech/ink/issues.
pub struct SrmlEnv<T> {
    marker: PhantomData<fn() -> T>,
}

#[cfg(feature = "ink-generate-abi")]
impl<E> type_metadata::HasTypeId for SrmlEnv<E>
where
    E: type_metadata::Metadata,
{
    fn type_id() -> type_metadata::TypeId {
        type_metadata::TypeIdCustom::new(
            "SrmlEnv",
            type_metadata::Namespace::from_module_path(module_path!())
                .expect("namespace from module path cannot fail"),
            vec![E::meta_type()],
        )
        .into()
    }
}

#[cfg(feature = "ink-generate-abi")]
impl<E> type_metadata::HasTypeDef for SrmlEnv<E> {
    fn type_def() -> type_metadata::TypeDef {
        type_metadata::TypeDefStruct::new(vec![]).into()
    }
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
    fn get_property_impl<P, I>(buffer: &mut I, ext_fn: fn()) -> P::In
    where
        P: property::ReadProperty,
        I: AsMut<[u8]> + EnlargeTo,
    {
        ext_fn();
        let req_len = ext::scratch_size();
        buffer.enlarge_to(req_len);
        ext::scratch_read(&mut buffer.as_mut()[0..req_len], 0);
        Decode::decode(&mut &buffer.as_mut()[0..req_len])
            .expect("failed at decoding the property")
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
                ) -> <property::$name<Self> as property::ReadProperty>::In
                where
                    I: AsMut<[u8]> + EnlargeTo,
                {
                    Self::get_property_impl::<property::$name::<T>, _>(buffer, $sys_fn)
                }
            }
        )*
    }
}

impl_get_property_for! {
    (Caller use ext::caller),
    (TransferredBalance use ext::value_transferred),
    (GasPrice use ext::gas_price),
    (GasLeft use ext::gas_left),
    (NowInMs use ext::now),
    (Address use ext::address),
    (Balance use ext::balance),
    (RentAllowance use ext::rent_allowance),
    (BlockNumber use ext::block_number),
    (MinimumBalance use ext::minimum_balance),
}

impl<T> GetProperty<property::Input<Self>> for SrmlEnv<T>
where
    T: EnvTypes,
{
    fn get_property<I>(
        buffer: &mut I,
    ) -> <property::Input<Self> as property::ReadProperty>::In
    where
        I: AsMut<[u8]> + EnlargeTo,
    {
        Self::get_property_impl::<property::Input<T>, _>(buffer, || ())
    }
}

impl<T> SetProperty<property::RentAllowance<Self>> for SrmlEnv<T>
where
    T: EnvTypes,
{
    fn set_property<O>(
        buffer: &mut O,
        value: &<property::RentAllowance<Self> as property::WriteProperty>::Out,
    ) where
        O: scale::Output + AsRef<[u8]> + Reset,
    {
        buffer.reset();
        value.encode_to(buffer);
        ext::set_rent_allowance(buffer.as_ref());
    }
}

impl<T> SrmlEnv<T>
where
    T: EnvTypes,
{
    /// Invokes the cross-contract call from the given parameters.
    ///
    /// Uses the given buffer as cache for intermediate data.
    fn invoke_contract_impl<O, Any>(
        buffer: &mut O,
        call_data: &CallParams<Self, Any>,
    ) -> Result<()>
    where
        O: scale::Output + AsRef<[u8]> + Reset,
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
        // Do the actual contract call.
        let ret = ext::call(callee, gas_limit, endowment, call_data);
        if !ret.is_success() {
            // Maybe the called contract trapped.
            return Err(Error::InvalidContractCall)
        }
        Ok(())
    }
}

impl<T> Env for SrmlEnv<T>
where
    T: EnvTypes,
{
    fn get_contract_storage<I, R>(buffer: &mut I, key: Key) -> Result<R>
    where
        I: AsMut<[u8]> + EnlargeTo,
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

    fn set_contract_storage<O, V>(buffer: &mut O, key: Key, val: &V)
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

    fn invoke_contract<O>(buffer: &mut O, call_data: &CallParams<Self, ()>) -> Result<()>
    where
        O: scale::Output + AsRef<[u8]> + Reset,
    {
        Self::invoke_contract_impl(buffer, call_data)
    }

    fn eval_contract<IO, R>(
        buffer: &mut IO,
        call_data: &CallParams<Self, ReturnType<R>>,
    ) -> Result<R>
    where
        IO: scale::Output + AsRef<[u8]> + AsMut<[u8]> + EnlargeTo + Reset,
        R: scale::Decode,
    {
        Self::invoke_contract_impl(buffer, call_data)?;
        // At this point our call was successful and we can now fetch
        // the returned data and decode it for the result value.
        let req_len = ext::scratch_size();
        buffer.enlarge_to(req_len);
        let ret = ext::scratch_read(&mut buffer.as_mut()[0..req_len], 0);
        if !ret.is_success() {
            return Err(Error::InvalidContractCallReturn)
        }
        Decode::decode(&mut &buffer.as_ref()[0..req_len]).map_err(Into::into)
    }

    fn create_contract<IO, C>(
        buffer: &mut IO,
        create_data: &CreateParams<Self, C>,
    ) -> Result<Self::AccountId>
    where
        IO: scale::Output + AsRef<[u8]> + AsMut<[u8]> + EnlargeTo + Reset,
    {
        // First we reset the buffer to start from a clean slate.
        buffer.reset();
        // Now we encode `code_hash`, `endowment` and `input_data`
        // each after one another into our buffer and remember their
        // boundaries using the guards.
        create_data.code_hash().encode_to(buffer);
        let code_hash_guard = buffer.as_ref().len();
        create_data.endowment().encode_to(buffer);
        let endowment_guard = buffer.as_ref().len();
        create_data.input_data().encode_to(buffer);
        // We now use the guards in order to split the buffer into
        // some read-only slices that each store their respective
        // encoded value and call the actual routine.
        let code_hash = &buffer.as_ref()[0..code_hash_guard];
        let endowment = &buffer.as_ref()[code_hash_guard..endowment_guard];
        let gas_limit = create_data.gas_limit();
        let call_data = &buffer.as_ref()[endowment_guard..];
        // Do the actual contract instantiation.
        let ret = ext::create(code_hash, gas_limit, endowment, call_data);
        if !ret.is_success() {
            // Maybe the called contract trapped.
            return Err(Error::InvalidContractInstantiation)
        }
        // At this point our contract instantiation was successful
        // and we can now fetch the returned data and decode it for
        // the result value.
        let req_len = ext::scratch_size();
        buffer.enlarge_to(req_len);
        let ret = ext::scratch_read(&mut buffer.as_mut()[0..req_len], 0);
        if !ret.is_success() {
            return Err(Error::InvalidContractInstantiationReturn)
        }
        Decode::decode(&mut &buffer.as_ref()[0..req_len]).map_err(Into::into)
    }

    fn emit_event<O, Event>(buffer: &mut O, event: Event)
    where
        O: scale::Output + AsRef<[u8]> + Reset,
        Event: Topics<Self> + scale::Encode,
    {
        // First we reset the buffer to start from a clean slate.
        buffer.reset();
        // Now we encode `topics` and the raw encoded `data`
        // each after one another into our buffer and remember their
        // boundaries using guards respectively.
        event.topics().encode_to(buffer);
        let topics_guard = buffer.as_ref().len();
        event.encode_to(buffer);
        // We now use the guards in order to split the buffer into
        // some read-only slices that each store their respective
        // encoded value and call the actual routine.
        let topics = &buffer.as_ref()[0..topics_guard];
        let data = &buffer.as_ref()[topics_guard..];
        // Do the actual depositing of the event.
        ext::deposit_event(topics, data);
    }

    fn invoke_runtime<O, V>(buffer: &mut O, call_data: &V)
    where
        O: scale::Output + AsRef<[u8]> + Reset,
        V: scale::Encode,
    {
        buffer.reset();
        call_data.encode_to(buffer);
        ext::dispatch_call(buffer.as_ref());
    }

    fn restore_to<O>(
        buffer: &mut O,
        dest: Self::AccountId,
        code_hash: Self::Hash,
        rent_allowance: Self::Balance,
        filtered_keys: &[Key],
    ) where
        O: scale::Output + AsRef<[u8]> + Reset,
    {
        // First we reset the buffer to start from a clean slate.
        buffer.reset();
        // Now we encode `dest`, `code_hash` and `rent_allowance`
        // each after one another into our buffer and remember their
        // boundaries using guards respectively.
        dest.encode_to(buffer);
        let dest_guard = buffer.as_ref().len();
        code_hash.encode_to(buffer);
        let code_hash_guard = buffer.as_ref().len();
        rent_allowance.encode_to(buffer);
        // We now use the guards in order to split the buffer into
        // some read-only slices that each store their respective
        // encoded value and call the actual routine.
        let dest = &buffer.as_ref()[0..dest_guard];
        let code_hash = &buffer.as_ref()[dest_guard..code_hash_guard];
        let rent_allowance = &buffer.as_ref()[code_hash_guard..];
        // Perform the actual restoration process.
        ext::restore_to(dest, code_hash, rent_allowance, filtered_keys);
    }

    fn output<O, R>(buffer: &mut O, return_value: &R)
    where
        O: scale::Output + AsRef<[u8]> + Reset,
        R: scale::Encode,
    {
        buffer.reset();
        return_value.encode_to(buffer);
        ext::scratch_write(buffer.as_ref());
    }

    fn random<I>(buffer: &mut I, subject: &[u8]) -> Self::Hash
    where
        I: AsMut<[u8]> + EnlargeTo,
    {
        ext::random_seed(subject);
        let req_len = ext::scratch_size();
        buffer.enlarge_to(req_len);
        ext::scratch_read(&mut buffer.as_mut()[0..req_len], 0);
        Decode::decode(&mut &buffer.as_mut()[0..req_len])
            .expect("failed at decoding value returned by `ext_random_seed`")
    }

    fn println(content: &str) {
        ext::println(content)
    }
}
