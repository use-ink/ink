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
/// # use ink_lang::codegen::DispatchInput;
/// const _: () = ink_lang::codegen::identity_type::<DispatchInput<i32>>();
/// ```
///
/// This fails to compile since `Foo` does not fulfill all requirements.
///
/// ```compile_fail
/// # use ink_lang::codegen::DispatchInput;
/// // Foo is missing scale codec implementations.
/// struct Foo {}
/// const _: () = ink_lang::codegen::identity_type::<DispatchInput<Foo>>();
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
/// # use ink_lang::codegen::DispatchOutput;
/// const _: () = ink_lang::codegen::identity_type::<DispatchOutput<i32>>();
/// ```
///
/// This fails to compile since `Foo` does not fulfill all requirements.
///
/// ```compile_fail
/// # use ink_lang::codegen::DispatchOutput;
/// // Foo is missing scale codec implementations.
/// struct Foo {}
/// const _: () = ink_lang::codegen::identity_type::<DispatchOutput<Foo>>();
/// ```
pub struct DispatchOutput<T>(T)
where
    T: scale::Encode + 'static;
