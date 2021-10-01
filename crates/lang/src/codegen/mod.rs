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

//! Definitions and facilities mainly used by the ink! codegen.

mod dispatch;
mod implies_return;
mod is_same_type;
mod trait_def;
mod trait_message;

pub use self::{
    dispatch::{
        deny_payment,
        execute_constructor,
        execute_message,
        DispatchInput,
        DispatchOutput,
        ExecuteConstructorConfig,
        ExecuteMessageConfig,
    },
    implies_return::ImpliesReturn,
    is_same_type::IsSameType,
    trait_def::TraitImplementedById,
    trait_message::{
        TraitMessagePayable,
        TraitMessageSelector,
    },
};

/// Takes a generic type as input and does nothing.
///
/// # Note
///
/// Used to trigger some compile time checks.
pub const fn identity_type<T>() {}
