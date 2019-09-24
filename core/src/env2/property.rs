use crate::{
    env2::{
        call::CallData,
        EnvTypes,
    },
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
    ( struct $name:ident { read: Some<$in:ty>, write: Some<$out:ty> }, $($tt:tt)* ) => {
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
    ( struct $name:ident { read: Some<$in:ty>, write: None }, $($tt:tt)* ) => {
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
    ( struct $name:ident { read: None, write: Some<$out:ty> }, $($tt:tt)* ) => {
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
    struct Caller { read: Some<E::AccountId>, write: None },
    struct TransferredBalance { read: Some<E::Balance>, write: None },
    struct GasPrice { read: Some<E::Balance>, write: None },
    struct GasLeft { read: Some<E::Balance>, write: None },
    struct NowInMs { read: Some<E::Moment>, write: None },
    struct Address { read: Some<E::AccountId>, write: None },
    struct Balance { read: Some<E::Balance>, write: None },
    struct RentAllowance { read: Some<E::Balance>, write: Some<E::Balance> },
    struct BlockNumber { read: Some<E::BlockNumber>, write: None },
    struct MinimumBalance { read: Some<E::Balance>, write: None },
    struct Input { read: Some<CallData>, write: None },
    struct Output { read: None, write: Some<[u8]> },
}
