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

/// An error that can occur during dispatch of ink! dispatchables.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum DispatchError {
    /// Failed to decode into a valid dispatch selector.
    InvalidSelector,
    /// The decoded selector is not known to the dispatch decoder.
    UnknownSelector,
    /// Failed to decode the parameters for the selected dispatchable.
    InvalidParameters,
    /// Failed to read execution input for the dispatchable.
    CouldNotReadInput,
    /// Invalidly paid an unpayable dispatchable.
    PaidUnpayableMessage,
}

impl core::fmt::Display for DispatchError {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl DispatchError {
    /// Returns a string representation of the error.
    #[inline]
    fn as_str(&self) -> &'static str {
        match self {
            Self::InvalidSelector => "unable to decode selector",
            Self::UnknownSelector => "encountered unknown selector",
            Self::InvalidParameters => "unable to decode input",
            Self::CouldNotReadInput => "could not read input",
            Self::PaidUnpayableMessage => "paid an unpayable message",
        }
    }
}

impl From<DispatchError> for scale::Error {
    #[inline]
    fn from(error: DispatchError) -> Self {
        Self::from(error.as_str())
    }
}

/// Decodes an ink! dispatch input into a known selector and its expected parameters.
///
/// # Note
///
/// This trait is automatically implemented for ink! message and constructor decoders.
///
/// # Errors
///
/// Returns an error if any of the decode steps failed:
///
/// - `InvalidSelector`: The first four bytes could not properly decoded into the
///   selector.
/// - `UnknownSelector`: The decoded selector did not match any of the expected ones.
/// - `InvalidParameters`: Failed to decoded the parameters for the selected dispatchable.
///
/// The other dispatch errors are handled by other structures usually.
///
/// # Usage
///
/// todo: prev doc test used a contract instance, it was in the `ink!` crate.
pub trait DecodeDispatch: Sized {
    /// todo: docs
    fn decode_dispatch(input: &mut &[u8]) -> Result<Self, DispatchError>;
}
