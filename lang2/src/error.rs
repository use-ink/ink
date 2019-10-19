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

/// A dispatch result.
pub type DispatchResult = core::result::Result<(), DispatchError>;

/// A dispatch error.
#[derive(Copy, Clone)]
pub enum DispatchError {
    UnknownSelector,
    UnknownInstantiateSelector,
    UnknownCallSelector,

    InvalidParameters,
    InvalidInstantiateParameters,
    InvalidCallParameters,
}

impl DispatchError {
    /// Converts `self` into an associated `u32` that SRML contracts can handle.
    #[inline]
    pub fn to_u32(self) -> u32 {
        match self {
            DispatchError::UnknownSelector => 0x01,
            DispatchError::UnknownInstantiateSelector => 0x02,
            DispatchError::UnknownCallSelector => 0x03,
            DispatchError::InvalidParameters => 0x04,
            DispatchError::InvalidInstantiateParameters => 0x05,
            DispatchError::InvalidCallParameters => 0x06,
        }
    }
}

/// A return code indicating success or error in a compact form.
#[derive(Copy, Clone)]
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
    /// This is useful to communicate back to SRML contracts.
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
