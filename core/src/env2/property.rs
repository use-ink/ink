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

//! Smart contract properties.
//!
//! Allow to query and mutate certain properties of a smart contract.

use crate::env2::{
    call::CallData,
    EnvTypes,
};
use core::marker::PhantomData;

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

macro_rules! impl_property_for {
    (
        $( #[$meta:meta] )*
        struct $name:ident { read: Some<$in:ty>, write: Some<$out:ty> }, $($tt:tt)*
    ) => {
        $( #[$meta] )*
        pub struct $name<E> { marker: PhantomData<fn () -> E>, }
        impl<E> $crate::env2::property::Property for $name<E> {}
        impl<E> $crate::env2::property::private::PropertySeal for $name<E> {}
        impl<E> $crate::env2::property::ReadProperty for $name<E>
        where
            E: EnvTypes,
        {
            type In = $in;
        }
        impl<E> $crate::env2::property::private::ReadPropertySeal for $name<E> {}
        impl<E> $crate::env2::property::WriteProperty for $name<E>
        where
            E: EnvTypes,
        {
            type Out = $out;
        }
        impl<E> $crate::env2::property::private::WritePropertySeal for $name<E> {}

        impl_property_for! { $($tt)* }
    };
    (
        $( #[$meta:meta] )*
        struct $name:ident { read: Some<$in:ty>, write: None }, $($tt:tt)*
    ) => {
        $( #[$meta] )*
        pub struct $name<E> { marker: PhantomData<fn () -> E>, }
        impl<E> $crate::env2::property::Property for $name<E> {}
        impl<E> $crate::env2::property::private::PropertySeal for $name<E> {}
        impl<E> $crate::env2::property::ReadProperty for $name<E>
        where
            E: EnvTypes,
        {
            type In = $in;
        }
        impl<E> $crate::env2::property::private::ReadPropertySeal for $name<E> {}

        impl_property_for! { $($tt)* }
    };
    (
        $( #[$meta:meta] )*
        struct $name:ident { read: None, write: Some<$out:ty> }, $($tt:tt)*
    ) => {
        $( #[$meta] )*
        pub struct $name<E> { marker: PhantomData<fn () -> E>, }
        impl<E> $crate::env2::property::Property for $name<E> {}
        impl<E> $crate::env2::property::private::PropertySeal for $name<E> {}
        impl<E> $crate::env2::property::WriteProperty for $name<E>
        where
            E: EnvTypes,
        {
            type Out = $out;
        }
        impl<E> $crate::env2::property::private::WritePropertySeal for $name<E> {}

        impl_property_for! { $($tt)* }
    };
    () => {};
}

impl_property_for! {
    /// The caller of an executed contract.
    struct Caller { read: Some<E::AccountId>, write: None },
    /// The transferred balance for the contract execution.
    struct TransferredBalance { read: Some<E::Balance>, write: None },
    /// The current gas price.
    struct GasPrice { read: Some<E::Balance>, write: None },
    /// The amount of gas left for the current contract execution.
    struct GasLeft { read: Some<E::Balance>, write: None },
    /// The block time in milli seconds.
    struct NowInMs { read: Some<E::Moment>, write: None },
    /// The account ID of the executed contract.
    struct Address { read: Some<E::AccountId>, write: None },
    /// The balance of the executed contract.
    struct Balance { read: Some<E::Balance>, write: None },
    /// The rent allowance of the executed contract.
    struct RentAllowance { read: Some<E::Balance>, write: Some<E::Balance> },
    /// The current block number.
    struct BlockNumber { read: Some<E::BlockNumber>, write: None },
    /// The minimum possible balance for a contract.
    struct MinimumBalance { read: Some<E::Balance>, write: None },
    /// The input data for this contract execution.
    struct Input { read: Some<CallData>, write: None },
}
