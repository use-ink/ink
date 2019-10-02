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
    env2::{
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
    cell::{RefCell, RefMut},
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

impl<T> GetProperty<property::Input<Self>> for TestEnv<T>
where
    T: EnvTypes,
{
    fn get_property<I>(
        _buffer: &mut I,
    ) -> <property::Input<Self> as property::ReadProperty>::In
    where
        I: AsMut<[u8]> + EnlargeTo,
    {
        INSTANCE.with(|instance| instance.borrow().exec_context.call_data.clone())
    }
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
                let account_id = &mut instance.exec_context.callee;
                instance.accounts
                    .get_mut(&account_id)
                    .expect("callee is required to be in the accounts DB")
            });
            account.rent_allowance.assign(value);
        })
    }
}
