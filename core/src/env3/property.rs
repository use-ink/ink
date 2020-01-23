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

//! Smart contract properties.
//!
//! Allow to query and mutate certain properties of a smart contract.

use core::marker::PhantomData;

use crate::env3::{
    call::CallData,
    EnvTypes,
};

pub(crate) mod private {
    /// A seal to prevent outside crates to implement new properties.
    pub trait PropertySeal {}
    /// A seal to prevent outside crates to change the read flags of existing properties.
    pub trait ReadPropertySeal {}
    /// A seal to prevent outside crates to change the write flags of existing properties.
    pub trait WritePropertySeal {}
}

/// A property.
///
/// # Note
///
/// Properties are information a contract can query or mutate.
/// Properties can either be read, write or both.
pub trait Property: private::PropertySeal {}
/// A read property can be read from a contract.
///
/// # Examples
///
/// The `AccountId` property can be read from a contract.
pub trait ReadProperty: Property + private::ReadPropertySeal {
    type In: scale::Decode + ?Sized;
}

/// A write property can be mutated by a contract.
///
/// # Examples
///
/// The `MinimumDeposit` can be mutated by a contract.
pub trait WriteProperty: Property + private::WritePropertySeal {
    type Out: scale::Encode + ?Sized;
}

macro_rules! impl_read_property_for {
    (
        $( #[$meta:meta] )*
        struct $name:ident { read: Some<$in:ty> }, $($tt:tt)*
    ) => {
        $( #[$meta] )*
        pub struct $name<E> { marker: PhantomData<fn () -> E>, }
        impl<E> $crate::env3::property::Property for $name<E> {}
        impl<E> $crate::env3::property::private::PropertySeal for $name<E> {}
        impl<E> $crate::env3::property::ReadProperty for $name<E>
        where
            E: EnvTypes,
        {
            type In = $in;
        }
        impl<E> $crate::env3::property::private::ReadPropertySeal for $name<E> {}

        impl_read_property_for! { $($tt)* }
    };
    () => {};
}

impl_read_property_for! {
    /// The caller of an executed contract.
    struct Caller { read: Some<E::AccountId> },
    /// The transferred balance for the contract execution.
    struct TransferredBalance { read: Some<E::Balance> },
    /// The current gas price.
    struct GasPrice { read: Some<E::Balance> },
    /// The amount of gas left for the current contract execution.
    struct GasLeft { read: Some<E::Balance> },
    /// The block time in milli seconds.
    struct NowInMs { read: Some<E::Moment> },
    /// The account ID of the executed contract.
    struct Address { read: Some<E::AccountId> },
    /// The balance of the executed contract.
    struct Balance { read: Some<E::Balance> },
    /// The current block number.
    struct BlockNumber { read: Some<E::BlockNumber> },
    /// The minimum possible balance for an account on the chain.
    struct MinimumBalance { read: Some<E::Balance> },
    /// The tombstone deposit on the contract chain.
    struct TombstoneDeposit { read: Some<E::Balance> },
}

/// The input data for this contract execution.
pub struct Input;

impl Property for Input {}
impl ReadProperty for Input {
    type In = CallData;
}
impl private::PropertySeal for Input {}
impl private::ReadPropertySeal for Input {}

/// The rent allowance of the executed contract.
pub struct RentAllowance<T> {
    marker: PhantomData<fn() -> T>,
}

impl<T> Property for RentAllowance<T> {}
impl<T> WriteProperty for RentAllowance<T>
where
    T: EnvTypes,
{
    type Out = T::Balance;
}
impl<T> ReadProperty for RentAllowance<T>
where
    T: EnvTypes,
{
    type In = T::Balance;
}
impl<T> private::PropertySeal for RentAllowance<T> {}
impl<T> private::WritePropertySeal for RentAllowance<T> {}
impl<T> private::ReadPropertySeal for RentAllowance<T> {}
