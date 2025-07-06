// Copyright (C) ink! contributors.
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

use alloy_sol_types::Error;
use ink_prelude::vec::Vec;

use crate::sol::{
    SolParamsDecode,
    SolParamsEncode,
};

/// Maps an arbitrary Rust/ink! type to a [Solidity custom error][sol-error] equivalent
/// for Solidity ABI encoding/decoding.
///
/// [sol-error]: https://soliditylang.org/blog/2021/04/21/custom-errors/
///
/// # Note
///
/// Implementing this trait entails:
/// - Specifying the equivalent Solidity custom error name.
/// - Declaring the equivalent Solidity ABI tuple type for the custom error parameters via
///   the `Params` associated type. Note that each tuple element type must implement
///   [`SolDecode`][crate::SolDecode] and [`SolEncode`][crate::SolEncode].
/// - Implementing the `from_params` method which defines how to convert from the Solidity
///   ABI error data representation (i.e. `Self::Params`) to this type.
/// - Implementing the `to_params` method which defines how to convert (preferably via a
///   borrow) from `&self` to `&Self::Params` (i.e. the Solidity ABI error data
///   representation).
///
/// # Example
///
/// ```
/// use ink_primitives::sol::SolCustomError;
///
/// pub struct MyError(u8);
///
/// impl SolCustomError for MyError {
///     const NAME: &'static str = "MyError";
///
///     type Params = (u8,);
///
///     fn from_params(value: Self::Params) -> Self {
///         Self(value.0)
///     }
///
///     fn to_params(&self) -> Self::Params {
///         (self.0,)
///     }
/// }
/// ```
pub trait SolCustomError {
    /// Name of the Solidity custom error.
    const NAME: &'static str;

    /// A tuple type representing the parameters of the Solidity custom error.
    ///
    /// # Note
    ///
    /// Each tuple element type must implement [`SolDecode`][crate::SolDecode] and
    /// [`SolEncode`][crate::SolEncode].
    type Params: SolParamsDecode + SolParamsEncode;

    /// Converts to `Self` from `Self::Params`.
    fn from_params(value: Self::Params) -> Self;

    /// Converts from `Self` to `Self::Params` via either a borrow (if possible), or
    /// a possibly expensive conversion otherwise.
    fn to_params(&self) -> Self::Params;
}

/// Solidity ABI decode error data (if possible).
pub trait SolErrorDecode {
    /// Solidity ABI decode error data into this type.
    fn decode(data: &[u8]) -> Result<Self, Error>
    where
        Self: Sized;
}

/// Solidity ABI encode as error data.
pub trait SolErrorEncode {
    /// Solidity ABI encode the value into Solidity error data.
    fn encode(&self) -> Vec<u8>;
}

/// Returns the selector of the equivalent [Solidity custom error][sol-error] for given
/// type which must implement [`SolCustomError`].
///
/// [sol-error]: https://soliditylang.org/blog/2021/04/21/custom-errors/
#[macro_export]
macro_rules! sol_error_selector {
    ($ty: ty) => {
        const {
            $crate::sol::selector_bytes(const_format::concatc!(
                <$ty as $crate::sol::SolCustomError>::NAME,
                <<$ty as $crate::sol::SolCustomError>::Params as $crate::sol::SolParamsDecode>::SOL_NAME
            ))
        }
    };
}

/// Implements [`SolErrorEncode`] and [`SolErrorDecode`] for the give type
/// which must implement [`SolCustomError`].
// Note: We use a macro instead of a generic implementation because we want to compute the
// selector at compile time, so we need a concrete type not a generic type (or Self).
#[macro_export]
macro_rules! impl_sol_error_codec {
    ($ty: ty) => {
        impl $crate::sol::SolErrorDecode for $ty {
            fn decode(data: &[u8]) -> Result<Self, $crate::sol::Error>
            where
                Self: Sized,
            {
                const SELECTOR: [u8; 4] = $crate::sol_error_selector!($ty);
                if data[..4] == SELECTOR {
                    <<Self as $crate::sol::SolCustomError>::Params as $crate::sol::SolParamsDecode>::decode(
                        &data[4..],
                    )
                    .map(<Self as $crate::sol::SolCustomError>::from_params)
                } else {
                    Err($crate::sol::Error::UnknownSelector {
                        name: <Self as $crate::sol::SolCustomError>::NAME,
                        selector: SELECTOR.into(),
                    })
                }
            }
        }

        impl $crate::sol::SolErrorEncode for $ty {
            fn encode(&self) -> Vec<u8> {
                const SELECTOR: [u8; 4] = $crate::sol_error_selector!($ty);
                let mut results = Vec::from(SELECTOR);
                results.extend(
                    <<Self as $crate::sol::SolCustomError>::Params as $crate::sol::SolParamsEncode>::encode(
                        &<Self as $crate::sol::SolCustomError>::to_params(self),
                    ),
                );
                results
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    // Equivalent to a Solidity custom error with no params.
    #[derive(Debug, PartialEq, Eq)]
    struct UnitError;
    impl SolCustomError for UnitError {
        const NAME: &'static str = "UnitError";
        type Params = ();

        fn from_params(_: Self::Params) -> Self {
            Self
        }

        fn to_params(&self) -> Self::Params {}
    }
    impl_sol_error_codec!(UnitError);

    // Equivalent to a Solidity custom error with params.
    #[derive(Debug, PartialEq, Eq)]
    struct ErrorWithParams(bool);
    impl SolCustomError for ErrorWithParams {
        const NAME: &'static str = "ErrorWithParams";
        type Params = (bool,);

        fn from_params(value: Self::Params) -> Self {
            Self(value.0)
        }

        fn to_params(&self) -> Self::Params {
            (self.0,)
        }
    }
    impl_sol_error_codec!(ErrorWithParams);

    #[test]
    fn error_selector_works() {
        // `keccak256("UnitError()")` == `0xe930c64c`
        assert_eq!(sol_error_selector!(UnitError), [0xe9, 0x30, 0xc6, 0x4c]);

        // `keccak256("ComplexError(bool)")` == `0xac3a6266`
        assert_eq!(
            sol_error_selector!(ErrorWithParams),
            [0xac, 0x3a, 0x62, 0x66]
        );
    }

    #[test]
    fn unit_error_works() {
        let error = UnitError;

        // `keccak256("UnitError()")` == `0xe930c64c`
        let encoded = vec![0xe9, 0x30, 0xc6, 0x4c];
        assert_eq!(SolErrorEncode::encode(&error), encoded);

        let decoded: UnitError = SolErrorDecode::decode(&encoded).unwrap();
        assert_eq!(error, decoded);
    }

    #[test]
    fn error_with_params_works() {
        let error = ErrorWithParams(true);

        // `keccak256("ComplexError(bool)")` == `0xac3a6266`
        let mut encoded = vec![0xac, 0x3a, 0x62, 0x66];
        // SolEncode(true) i.e. `0x1` preceded by 31 `0x0`
        let mut encoded_params = [0x0; 32];
        encoded_params[31] = 0x1;
        encoded.extend(encoded_params);
        assert_eq!(SolErrorEncode::encode(&error), encoded);

        let decoded: ErrorWithParams = SolErrorDecode::decode(&encoded).unwrap();
        assert_eq!(error, decoded);
    }
}
