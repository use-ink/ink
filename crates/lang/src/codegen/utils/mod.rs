// Copyright 2018-2022 Parity Technologies (UK) Ltd.
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

//! Utility types and definitions used by the ink! codegen.

use core::fmt::{
    Debug,
    Display,
};

mod identity_type;
mod same_type;

pub use self::{
    identity_type::consume_type,
    same_type::IsSameType,
};

#[cfg(any(feature = "ink-debug", feature = "std"))]
pub fn unwrap_constructor<T, E, DisplayError: Display + Debug>(
    res: Result<T, E>,
    err: DisplayError,
) -> T {
    res.unwrap_or_else(|_| ::core::panic!("dispatching ink! constructor failed: {}", err))
}

#[cfg(any(feature = "ink-debug", feature = "std"))]
pub fn unwrap_message<T, E, DisplayError: Display + Debug>(
    res: Result<T, E>,
    err: DisplayError,
) -> T {
    res.unwrap_or_else(|_| ::core::panic!("dispatching ink! message failed: {}", err))
}

#[cfg(not(any(feature = "ink-debug", feature = "std")))]
pub fn unwrap_constructor<T, E: Debug, DisplayError: Display>(
    res: Result<T, E>,
    _: DisplayError,
) -> T {
    res.unwrap()
}

#[cfg(not(any(feature = "ink-debug", feature = "std")))]
pub fn unwrap_message<T, E: Debug, DisplayError: Display>(
    res: Result<T, E>,
    _: DisplayError,
) -> T {
    res.unwrap()
}
