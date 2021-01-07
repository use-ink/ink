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

/// Trait implemented by chain extensions.
///
/// Allows to use the `self.env().extension().my_chain_extension(..)` syntax.
///
/// # Note
///
/// This trait is automatically implemented when using `#[ink::chain_extension]` proc. macro.
pub trait ChainExtensionInstance {
    /// The type of the chain extension instance.
    type Instance;

    /// Creates a new instance of the chain extension to use methods with method chaining syntax.
    fn instantiate() -> Self::Instance;
}

/// Implemented by chain extension types.
///
/// Every chain extension defines a set of chain extension methods
/// that share a common error code type.
pub trait ChainExtension {
    /// The error code that determines whether a chain extension method call was successful.
    type ErrorCode: ink_env::chain_extension::FromStatusCode;
}

/// Only implemented for `Result<T, E>`.
///
/// Used to check at compile time if the return type of a chain extension method
/// is a `Result` type using the type system instead of the syntactic structure.
#[doc(hidden)]
pub trait IsResultType: private::Sealed {
    /// The `T` type of the `Result<T, E>`.
    type Ok;
    /// The `E` type of the `Result<T, E>`.
    type Err;
}

impl<T, E> private::Sealed for Result<T, E> {}
impl<T, E> IsResultType for Result<T, E> {
    type Ok = T;
    type Err = E;
}

mod private {
    /// Seals the `IsResultType` trait so that it cannot be implemented outside of this module.
    pub trait Sealed {}
}
