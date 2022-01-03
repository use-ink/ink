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

/// Takes a generic type as input and just consumes it while doing nothing.
///
/// # Note
///
/// This can be used to trigger some compile time checks due to the fact
/// that the type consumed this way is type checked. We usually use this
/// to make the Rust compiler check the trait bounds in particular.
///
/// # Usage: Compiles
///
/// ```
/// # use ink_lang::codegen::utils::consume_type;
/// # use core::marker::PhantomData;
/// #
/// pub struct RequiresCopy<T: Copy>(PhantomData<T>);
///
/// // The following line of code works because `i32: Copy`.
/// let _: () = consume_type::<RequiresCopy<i32>>();
/// ```
///
/// # Usage: Compile Error
///
/// ```compile_fail
/// # use ink_lang::codegen::utils::consume_type;
/// # use core::marker::PhantomData;
/// #
/// pub struct RequiresCopy<T: Copy>(PhantomData<T>);
///
/// // The following line of code fails to compile because
/// // `String` does not implement `Copy`.
/// let _: () = consume_type::<RequiresCopy<String>>();
/// ```
pub const fn consume_type<T>() {}
