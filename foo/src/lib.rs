// Copyright (C) Parity Technologies (UK) Ltd.
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

//! External C API to communicate with substrate contracts runtime module.
//!
//! Refer to substrate FRAME contract module for more documentation.

#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg))]

#[derive(PartialEq, Eq)]
#[repr(u32)]
pub enum BarPlain {
    /// API call successful.
    Success = 0,
    /// The called function trapped and has its state changes reverted.
    /// In this case no output buffer is returned.
    /// Can only be returned from `call` and `instantiate`.
    CalleeTrapped = 1,
    /// Returns if an unknown error was received from the host module.
    Unknown,
}

impl ::core::fmt::Debug for BarPlain {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::write_str(
            f,
            match self {
                BarPlain::Success => "Success",
                BarPlain::CalleeTrapped => "CalleeTrapped",
                BarPlain::Unknown => "Unknown",
            },
        )
    }
}
