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

/// Defines a base event type for the contract.
///
/// This is usually the event enum that comprises all defined event types.
///
/// # Usage
///
/// ```
/// use ink_lang as ink;
///
/// #[ink::contract]
/// pub mod contract {
///     #[ink(storage)]
///     pub struct Contract {}
///
///     #[ink(event)]
///     pub struct Event1 {}
///
///     #[ink(event)]
///     pub struct Event2 {}
///
///     impl Contract {
///         #[ink(constructor)]
///         pub fn constructor() -> Self { Self {} }
///
///         #[ink(message)]
///         pub fn message(&self) {}
///     }
/// }
///
/// use contract::Contract;
/// # use ink_lang::reflect::ContractEventBase;
///
/// type BaseEvent = <Contract as ContractEventBase>::Type;
/// ```
pub trait ContractEventBase {
    /// The generated base event enum.
    type Type;
}

/// todo: docs
pub trait EventInfo {
    /// The complete path of the ink! event definition.
    ///
    /// This is equivalent to Rust's builtin `module_path!` macro
    /// invocation at the definition site of the ink! event, concatenated with
    /// the event identifier.
    ///
    /// todo: rename?
    const PATH: &'static str;


}
