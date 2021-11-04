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

/// Stores information for every ink! trait message of an ink! trait definition.
///
/// This information includes if the ink! trait message is payable
/// as well as its derived or manually specified selector.
///
/// In the future this info trait might be extended to contain
/// more information about a single ink! trait message.
///
/// The information provided through this trait can be used on the
/// implementer side of an ink! trait to check and guard certain
/// properties on a Rust type system level. This is important since
/// ink! cannot be guaranteed to have both the ink! trait definition
/// and all of its implementers under its scope and radar.
///
/// # Note
///
/// - The `TraitMessageInfo<LOCAL_ID>` is implemented by the
///   automatically generated ink! trait definition information object
///   associated to the ink! trait definition at hand.
/// - For every ink! trait message defined by the ink! trait definition
///   the associated ink! trait definition information object implements
///   this trait given the `TRAIT_LOCAL_MESSAGE_ID` of each ink! trait
///   message respectively.
/// - The local IDs uniquely identifying all the ink! trait messages
///   of the ink! trait definition are computed solely using the Rust
///   identifier of the ink! trait message which can be derived from
///   ink! implementation blocks in order to query the information
///   stored by this ink! trait information object trait implementation.
///
/// # Usage
///
/// ```
/// use ink_lang as ink;
///
/// #[ink::trait_definition]
/// pub trait InkTrait {
///     #[ink(message)]
///     fn trait_message_1(&self);
///
///     #[ink(message, payable, selector = 0xC0DECAFE)]
///     fn trait_message_2(&mut self);
/// }
///
/// #[ink::trait_definition(namespace = "foo")]
/// pub trait InkTrait2 {
///     #[ink(message)]
///     fn trait_message(&self);
/// }
///
/// #[ink::contract]
/// pub mod contract {
///     use super::{InkTrait, InkTrait2};
///
///     #[ink(storage)]
///     pub struct Contract {}
///
///     impl Contract {
///         #[ink(constructor)]
///         pub fn constructor() -> Self { Contract {} }
///     }
///
///     impl InkTrait for Contract {
///         #[ink(message)]
///         fn trait_message_1(&self) {}
///
///         #[ink(message)]
///         fn trait_message_2(&mut self) {}
///     }
///
///     impl InkTrait2 for Contract {
///         #[ink(message)]
///         fn trait_message(&self) {}
///     }
/// }
///
/// # use ink_lang::reflect::TraitDefinitionRegistry;
/// # use ink_lang::reflect::TraitMessageInfo;
/// # use ink_env::DefaultEnvironment;
/// # use ink_lang::{selector_id, selector_bytes};
///
/// fn main() {
///     assert_eq!(
///         <<TraitDefinitionRegistry<DefaultEnvironment> as InkTrait>::__ink_TraitInfo
///             as TraitMessageInfo<{selector_id!("trait_message_1")}>>::PAYABLE,
///         false,
///     );
///     assert_eq!(
///         <<TraitDefinitionRegistry<DefaultEnvironment> as InkTrait>::__ink_TraitInfo
///             as TraitMessageInfo<{selector_id!("trait_message_2")}>>::PAYABLE,
///         true,
///     );
///     assert_eq!(
///         <<TraitDefinitionRegistry<DefaultEnvironment> as InkTrait2>::__ink_TraitInfo
///             as TraitMessageInfo<{selector_id!("trait_message")}>>::PAYABLE,
///         false,
///     );
///     assert_eq!(
///         <<TraitDefinitionRegistry<DefaultEnvironment> as InkTrait>::__ink_TraitInfo
///             as TraitMessageInfo<{selector_id!("trait_message_1")}>>::SELECTOR,
///         selector_bytes!("InkTrait::trait_message_1")
///     );
///     assert_eq!(
///         <<TraitDefinitionRegistry<DefaultEnvironment> as InkTrait>::__ink_TraitInfo
///             as TraitMessageInfo<{selector_id!("trait_message_2")}>>::SELECTOR,
///         [0xC0, 0xDE, 0xCA, 0xFE]
///     );
///     assert_eq!(
///         <<TraitDefinitionRegistry<DefaultEnvironment> as InkTrait2>::__ink_TraitInfo
///             as TraitMessageInfo<{selector_id!("trait_message")}>>::SELECTOR,
///         selector_bytes!("foo::InkTrait2::trait_message")
///     );
/// }
/// ```
pub trait TraitMessageInfo<const TRAIT_LOCAL_MESSAGE_ID: u32> {
    /// Is `true` if the ink! trait message has been annotated with `#[ink(payable)]`.
    const PAYABLE: bool;

    /// The unique selector of the ink! trait message.
    ///
    /// This might have been adjusted using `#[ink(selector = N:u32)]` at the
    /// ink! trait definition site.
    const SELECTOR: [u8; 4];
}

/// Captures the module path of the ink! trait definition.
///
/// This can be used to differentiate between two equally named
/// ink! trait definitions and also for metadata.
pub trait TraitModulePath {
    /// The module path of the ink! trait definition.
    ///
    /// This is equivalent to Rust's builtin `module_path!` macro
    /// invocation at the definition site of the ink! trait.
    const PATH: &'static str;

    /// The name of the ink! trait.
    ///
    /// This is just for convenience.
    const NAME: &'static str;
}
