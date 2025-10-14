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

use ink_primitives::{
    SolDecode,
    sol::SolResultEncode,
};

/// Used to check if `T` is allowed as ink! input parameter type.
///
/// # Note
///
/// An ink! input parameter type must implement [`scale::Decode`]
/// and must have a `'static` lifetime.
///
/// # Example
///
/// This compiles since `i32` fulfills the requirements of an ink! input.
///
/// ```
/// # use ink::codegen::DispatchInput;
/// const _: () = ink::codegen::utils::consume_type::<DispatchInput<i32>>();
/// ```
///
/// This fails to compile since `Foo` does not fulfill all requirements.
///
/// ```compile_fail
/// # use ink::codegen::DispatchInput;
/// // Foo is missing scale codec implementations.
/// struct Foo {}
/// const _: () = ink::codegen::utils::consume_type::<DispatchInput<Foo>>();
/// ```
pub struct DispatchInput<T>(T)
where
    T: scale::Decode + 'static;

/// Used to check if `T` is allowed as ink! output parameter type.
///
/// # Note
///
/// An ink! input parameter type must implement [`scale::Encode`]
/// and must have a `'static` lifetime.
///
/// # Example
///
/// This compiles since `i32` fulfills the requirements of an ink! output.
///
/// ```
/// # use ink::codegen::DispatchOutput;
/// const _: () = ink::codegen::utils::consume_type::<DispatchOutput<i32>>();
/// ```
///
/// This fails to compile since `Foo` does not fulfill all requirements.
///
/// ```compile_fail
/// # use ink::codegen::DispatchOutput;
/// // Foo is missing scale codec implementations.
/// struct Foo {}
/// const _: () = ink::codegen::utils::consume_type::<DispatchOutput<Foo>>();
/// ```
pub struct DispatchOutput<T>(T)
where
    T: scale::Encode + 'static;

/// Used to check if `T` is allowed as ink! input parameter type.
///
/// # Note
///
/// An ink! input parameter type must implement [`ink::SolDecode`][crate::SolDecode]
/// and must have a `'static` lifetime.
///
/// # Example
///
/// This compiles since `i32` fulfills the requirements of an ink! input.
///
/// ```
/// # use ink::codegen::DispatchInputSol;
/// const _: () = ink::codegen::utils::consume_type::<DispatchInputSol<i32>>();
/// ```
///
/// This fails to compile since `Foo` does not fulfill all requirements.
///
/// ```compile_fail
/// # use ink::codegen::DispatchInputSol;
/// // Foo is missing scale codec implementations.
/// struct Foo {}
/// const _: () = ink::codegen::utils::consume_type::<DispatchInputSol<Foo>>();
/// ```
pub struct DispatchInputSol<T>(T)
where
    T: SolDecode + 'static;

/// Used to check if `T` is allowed as ink! output parameter type.
///
/// # Note
///
/// An ink! input parameter type must implement
/// [`ink::sol::SolResultEncode`][crate::sol::SolResultEncode] and must have a `'static`
/// lifetime.
///
/// # Example
///
/// This compiles since `i32` fulfills the requirements of an ink! output.
///
/// ```
/// # use ink::codegen::DispatchOutputSol;
/// const _: () = ink::codegen::utils::consume_type::<DispatchOutputSol<i32>>();
/// ```
///
/// This fails to compile since `Foo` does not fulfill all requirements.
///
/// ```compile_fail
/// # use ink::codegen::DispatchOutputSol;
/// // Foo is missing scale codec implementations.
/// struct Foo {}
/// const _: () = ink::codegen::utils::consume_type::<DispatchOutputSol<Foo>>();
/// ```
pub struct DispatchOutputSol<T>(T)
where
    T: for<'a> SolResultEncode<'a> + 'static;
