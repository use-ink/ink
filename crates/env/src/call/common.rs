// Copyright (C) Use Ink (UK) Ltd.
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

//! Utilities, types and abstractions common to call and instantiation routines.

use core::marker::PhantomData;

use ink_primitives::{
    LangError,
    MessageResult,
    abi::{
        Ink,
        Sol,
    },
    sol::{
        SolErrorDecode,
        SolResultDecode,
        SolResultDecodeError,
    },
};
use pallet_revive_uapi::ReturnErrorCode;
use scale::{
    Decode,
    DecodeAll,
};

/// Represents a return type.
///
/// Used as a marker type to define the return type of an ink! message in call builders.
#[derive(Debug)]
pub struct ReturnType<T>(PhantomData<fn() -> T>);

impl<T> Clone for ReturnType<T> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for ReturnType<T> {}

impl<T> Default for ReturnType<T> {
    #[inline]
    fn default() -> Self {
        Self(Default::default())
    }
}

/// A parameter that has been set to some value.
#[derive(Debug, Copy, Clone)]
pub struct Set<T>(pub T);

impl<T> Set<T> {
    /// Returns the set value.
    #[inline]
    pub fn value(self) -> T {
        self.0
    }
}

/// A parameter that has not been set, yet.
#[derive(Debug)]
pub struct Unset<T>(PhantomData<fn() -> T>);

impl<T> Clone for Unset<T> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Unset<T> {}

impl<T> Default for Unset<T> {
    #[inline]
    fn default() -> Self {
        Self(Default::default())
    }
}

/// Implemented by [`Set`] and [`Unset`] in order to unwrap their value.
///
/// This is useful in case the use-site does not know if it is working with
/// a set or an unset value generically unwrap it using a closure for fallback.
pub trait Unwrap {
    /// The output type of the `unwrap_or_else` operation.
    type Output;

    /// Returns the set value or evaluates the given closure.
    fn unwrap_or_else<F>(self, f: F) -> Self::Output
    where
        F: FnOnce() -> Self::Output;
}

impl<T> Unwrap for Unset<T> {
    type Output = T;

    #[inline]
    fn unwrap_or_else<F>(self, f: F) -> Self::Output
    where
        F: FnOnce() -> Self::Output,
    {
        f()
    }
}

impl<T> Unwrap for Set<T> {
    type Output = T;

    #[inline]
    fn unwrap_or_else<F>(self, _: F) -> Self::Output
    where
        F: FnOnce() -> Self::Output,
    {
        self.value()
    }
}

/// A trait for decoding the output of a message based on the ABI.
///
/// # Note
///
/// This is necessary because messages supporting different ABI have different return
/// types. For example, Solidity ABI encoded messages return the output directly without
/// `MessageResult`.
pub trait DecodeMessageResult<Abi>: Sized {
    /// Decodes the output of a message call, requiring the output
    /// to be wrapped with `MessageResult` (if not included in the output).
    fn decode_output(
        buffer: &[u8],
        did_revert: bool,
    ) -> crate::Result<MessageResult<Self>>;
}

impl<R> DecodeMessageResult<Ink> for R
where
    R: Decode,
    MessageResult<R>: Decode,
{
    fn decode_output(mut buffer: &[u8], _: bool) -> crate::Result<MessageResult<Self>> {
        let decoded = MessageResult::<R>::decode_all(&mut buffer)?;
        Ok(decoded)
    }
}

impl<R> DecodeMessageResult<Sol> for R
where
    R: SolResultDecode,
{
    fn decode_output(
        buffer: &[u8],
        did_revert: bool,
    ) -> crate::Result<MessageResult<Self>> {
        // Solidity ABI Encoded contracts return the data without
        // `MessageResult`.
        let decoded = R::decode(buffer, did_revert)?;
        Ok(Ok(decoded))
    }
}

impl From<SolResultDecodeError> for crate::Error {
    fn from(value: SolResultDecodeError) -> Self {
        match value {
            SolResultDecodeError::NonResultFromRevert => {
                Self::ReturnError(ReturnErrorCode::CalleeReverted)
            }
            SolResultDecodeError::Decode => Self::DecodeSol(ink_primitives::sol::Error),
        }
    }
}

/// A trait for decoding constructor error data based on ABI.
///
/// # Note
///
/// This is necessary because constructors supporting different ABIs encode return data
/// differently.
///
/// For example, ink! ABI encoded constructors return data encoded as
/// `ConstructorResult<Result<_, Error>, LangErr>` where `Error` is either the user
/// defined error for fallible constructors, or unit (i.e. `()`) for infallible
/// constructors. On the other hand, Solidity ABI encoded constructors always return the
/// output data directly and the state of the revert flag determines whether its "normal"
/// return data or error data.
///
/// This trait assumes the caller has already checked that the revert flag is set.
pub trait DecodeConstructorError<Abi>: Sized {
    /// Decodes constructor error data.
    fn decode_error_output(buffer: &[u8]) -> ConstructorError<Self>;
}

/// A decoded constructor error.
pub enum ConstructorError<E> {
    /// A user defined error.
    Contract(E),
    /// A `LangError`.
    Lang(LangError),
    /// An environmental error.
    Env(crate::Error),
}

impl<E> DecodeConstructorError<Ink> for E
where
    E: Decode,
{
    fn decode_error_output(mut buffer: &[u8]) -> ConstructorError<Self> {
        // ink! ABI encoded constructors return data encoded as
        // `ConstructorResult<Result<_, Error>, LangErr>` where `Error` is either the user
        // defined error for fallible entry points or unit (i.e. `()`) for
        // infallible entry points.
        let out_return_value = &mut buffer;

        // Debug friendly SCALE decode errors.
        const INVALID_OUTER_RESULT: &str = "Invalid outer constructor Result encoding, \
        expected 0 or 1 as the first byte";
        const INVALID_INNER_RESULT: &str = "Invalid inner constructor Result encoding, \
        expected 0 or 1 as the first byte";
        const REVERT_BUT_NOT_ERROR_DATA: &str =
            "The callee reverted, but did not encode an error in the output buffer.";
        fn scale_decode_err<T>(desc: &'static str) -> ConstructorError<T> {
            ConstructorError::Env(crate::Error::Decode(desc.into()))
        }

        let Ok(lang_result_variant) = <_ as scale::Input>::read_byte(out_return_value)
        else {
            return scale_decode_err(INVALID_OUTER_RESULT);
        };
        match lang_result_variant {
            // 0 == `ConstructorResult::Ok` variant
            0 => {
                let Ok(inner_result_variant) =
                    <_ as scale::Input>::read_byte(out_return_value)
                else {
                    return scale_decode_err(INVALID_INNER_RESULT);
                };
                match inner_result_variant {
                    // 0 == `Ok` variant
                    0 => scale_decode_err(REVERT_BUT_NOT_ERROR_DATA),
                    // 1 == `Err` variant
                    1 => {
                        let decoded = <E as scale::Decode>::decode(out_return_value);
                        match decoded {
                            Ok(contract_err) => ConstructorError::Contract(contract_err),
                            Err(error) => {
                                ConstructorError::Env(crate::Error::Decode(error))
                            }
                        }
                    }
                    _ => scale_decode_err(INVALID_INNER_RESULT),
                }
            }
            // 1 == `ConstructorResult::Err` variant
            1 => {
                let decoded = <LangError as scale::Decode>::decode(out_return_value);
                match decoded {
                    Ok(lang_err) => ConstructorError::Lang(lang_err),
                    Err(error) => ConstructorError::Env(crate::Error::Decode(error)),
                }
            }
            _ => scale_decode_err(INVALID_OUTER_RESULT),
        }
    }
}

impl<E> DecodeConstructorError<Sol> for E
where
    E: SolErrorDecode,
{
    fn decode_error_output(buffer: &[u8]) -> ConstructorError<Self> {
        // Solidity ABI encoded entry points return error data directly.
        let decoded = SolErrorDecode::decode(buffer);
        match decoded {
            Ok(contract_err) => ConstructorError::Contract(contract_err),
            Err(error) => ConstructorError::Env(crate::Error::DecodeSol(error)),
        }
    }
}
