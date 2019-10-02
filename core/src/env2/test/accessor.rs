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

//! # For Developers
//!
//! For the implementation of the `Env` trait and all of its sub traits
//! we can ignore the `buffer` parameters that we use extensively in the
//! `SrmlEnv` implementation since here we already have our buffers in the
//! contract's memory.

use crate::{
    byte_utils,
    storage::{
        Key,
    },
    env2::{
        CallParams,
        CreateParams,
        EmitEventParams,
        Env,
        Result,
        DefaultSrmlTypes,
        call::{
            Selector,
            CallData,
        },
        property,
        test::{
            instance::TestEnvInstance,
            Storage,
        },
        utils::{
            EnlargeTo,
            Reset,
        },
        EnvTypes,
        GetProperty,
        SetProperty,
        types,
    },
};
use core::{
    marker::PhantomData,
    cell::{RefCell, Ref, RefMut},
};

thread_local! {
    /// The single thread-local test environment instance.
    ///
    /// # Note
    ///
    /// This needs to be thread local since tests are run
    /// in parallel by default which may lead to data races otherwise.
    pub static INSTANCE: RefCell<TestEnvInstance> = {
        RefCell::new(TestEnvInstance::default())
    };
}

/// Accessor to the test environment instance.
///
/// This acts as the real test environment to the outside.
pub struct TestEnv<T> {
    /// Needed to trick Rust into allowing `T`.
    marker: PhantomData<fn() -> T>,
}

impl<T> EnvTypes for TestEnv<T>
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

impl<T> SetProperty<property::RentAllowance<Self>> for TestEnv<T>
where
    T: EnvTypes,
{
    fn set_property<O>(
        _buffer: &mut O,
        value: &<property::RentAllowance<Self> as property::WriteProperty>::Out,
    ) where
        O: scale::Output + AsRef<[u8]> + Reset,
    {
        INSTANCE.with(|instance| {
            let mut account = RefMut::map(instance.borrow_mut(), |instance| {
                let account_id = &instance.exec_context.callee;
                instance.accounts
                    .get_mut(account_id)
                    .expect("callee is required to be in the accounts DB")
            });
            account.rent_allowance.assign(value);
        })
    }
}

macro_rules! impl_get_property_for {
    ( $prop_name:ident => |$name:ident| $body:tt $($rest:tt)* ) => {
        impl<T> GetProperty<property::$prop_name<Self>> for TestEnv<T>
        where
            T: EnvTypes,
        {
            fn get_property<I>(
                _buffer: &mut I,
            ) -> <property::$prop_name<Self> as property::ReadProperty>::In
            where
                I: AsMut<[u8]> + EnlargeTo,
            {
                INSTANCE.with(|$name| $body)
            }
        }

        impl_get_property_for!($($rest)*);
    };
    () => {};
}

impl_get_property_for! {
    Input => |instance| {
        instance.borrow().exec_context.call_data.clone()
    }

    Caller => |instance| {
        instance.borrow().exec_context.caller.to_origin()
    }

    TransferredBalance => |instance| {
        instance.borrow().exec_context.transferred_balance.to_origin()
    }

    GasPrice => |instance| {
        instance.borrow().state.gas_price.to_origin()
    }

    GasLeft => |instance| {
        instance.borrow().exec_context.gas_left.to_origin()
    }

    NowInMs => |instance| {
        instance.borrow().exec_context.gas_left.to_origin()
    }

    Address => |instance| {
        instance.borrow().exec_context.callee.to_origin()
    }

    Balance => |instance| {
        let account = Ref::map(instance.borrow(), |instance| {
            let account_id = &instance.exec_context.callee;
            instance.accounts
                .get(&account_id)
                .expect("callee is required to be in the accounts DB")
        });
        account.balance.to_origin()
    }

    RentAllowance => |instance| {
        let account = Ref::map(instance.borrow(), |instance| {
            let account_id = &instance.exec_context.callee;
            instance.accounts
                .get(&account_id)
                .expect("callee is required to be in the accounts DB")
        });
        account.rent_allowance.to_origin()
    }

    BlockNumber => |instance| {
        instance.borrow().block.number.to_origin()
    }

    MinimumBalance => |instance| {
        instance.borrow().state.minimum_balance.to_origin()
    }
}

impl<T> Env for TestEnv<T>
where
    T: EnvTypes,
{
    fn get_contract_storage<I, R>(buffer: &mut I, key: Key) -> Result<R>
    where
        I: AsMut<[u8]> + EnlargeTo,
        R: scale::Decode,
    {
        unimplemented!()
    }

    fn set_contract_storage<O, V>(buffer: &mut O, key: Key, val: &V)
    where
        O: scale::Output + AsRef<[u8]> + Reset,
        V: scale::Encode,
    {
        unimplemented!()
    }

    fn clear_contract_storage(key: Key) {
        unimplemented!()
    }

    fn invoke_contract<O, D>(buffer: &mut O, call_data: &D) -> Result<()>
    where
        O: scale::Output + AsRef<[u8]> + Reset,
        D: CallParams<Self>,
    {
        unimplemented!()
    }

    fn eval_contract<IO, D, R>(buffer: &mut IO, call_data: &D) -> Result<R>
    where
        IO: scale::Output + AsRef<[u8]> + AsMut<[u8]> + EnlargeTo + Reset,
        R: scale::Decode,
        D: CallParams<Self>,
    {
        unimplemented!()
    }

    fn create_contract<IO, D>(buffer: &mut IO, create_data: &D) -> Result<Self::AccountId>
    where
        IO: scale::Output + AsRef<[u8]> + AsMut<[u8]> + EnlargeTo + Reset,
        D: CreateParams<Self>,
    {
        unimplemented!()
    }

    fn emit_event<I, D>(buffer: &mut I, event_data: &D)
    where
        I: scale::Output + AsRef<[u8]> + Reset,
        D: EmitEventParams<Self>,
    {
        unimplemented!()
    }

    fn invoke_runtime<O, V>(buffer: &mut O, call_data: &V)
    where
        O: scale::Output + AsRef<[u8]> + Reset,
        V: scale::Encode,
    {
        unimplemented!()
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
        unimplemented!()
    }

    fn output<O, R>(buffer: &mut O, return_value: &R)
    where
        O: scale::Output + AsRef<[u8]> + Reset,
        R: scale::Encode,
    {
        unimplemented!()
    }

    fn random<I>(buffer: &mut I, subject: &[u8]) -> Self::Hash
    where
        I: AsMut<[u8]> + EnlargeTo,
    {
        unimplemented!()
    }

    fn println(content: &str) {
        println!("{}", content)
    }
}
