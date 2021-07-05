// Copyright 2018-2021 Parity Technologies (UK) Ltd.
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

/// A dispatch result.
#[doc(hidden)]
pub type DispatchResult = core::result::Result<(), DispatchError>;

/// A dispatch error.
#[derive(Debug, Copy, Clone)]
#[doc(hidden)]
pub enum DispatchError {
    UnknownSelector,
    UnknownInstantiateSelector,
    UnknownCallSelector,

    InvalidParameters,
    InvalidInstantiateParameters,
    InvalidCallParameters,

    CouldNotReadInput,
    PaidUnpayableMessage,
}

impl DispatchError {
    /// Converts `self` into an associated `u32` that FRAME contracts can handle.
    #[inline]
    pub fn to_u32(self) -> u32 {
        DispatchRetCode::from(self).to_u32()
    }
}

/// A return code indicating success or error in a compact form.
#[derive(Copy, Clone)]
#[doc(hidden)]
pub struct DispatchRetCode(u32);

impl DispatchRetCode {
    /// Creates a return code indicating success.
    #[inline]
    pub fn success() -> Self {
        Self(0)
    }

    /// Returns the `u32` representation of `self`.
    ///
    /// # Note
    ///
    /// This is useful to communicate back to FRAME contracts.
    #[inline]
    pub fn to_u32(self) -> u32 {
        self.0
    }
}

impl From<DispatchError> for DispatchRetCode {
    #[inline]
    fn from(err: DispatchError) -> Self {
        match err {
            DispatchError::UnknownSelector => Self(0x01),
            DispatchError::UnknownInstantiateSelector => Self(0x02),
            DispatchError::UnknownCallSelector => Self(0x03),
            DispatchError::InvalidParameters => Self(0x04),
            DispatchError::InvalidInstantiateParameters => Self(0x05),
            DispatchError::InvalidCallParameters => Self(0x06),
            DispatchError::CouldNotReadInput => Self(0x07),
            DispatchError::PaidUnpayableMessage => Self(0x08),
        }
    }
}

impl From<DispatchResult> for DispatchRetCode {
    #[inline]
    fn from(res: DispatchResult) -> Self {
        match res {
            Ok(_) => Self::success(),
            Err(err) => Self::from(err),
        }
    }
}
