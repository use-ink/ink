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

use crate::DispatchError;

/// Reflects the number of dispatchable ink! messages and constructors respectively.
///
/// # Note
///
/// - This is automatically implemented by all ink! smart contracts.
/// - All ink! constructors and ink! messages of an ink! smart contract are dispatchables.  
///   This explicitly includes ink! messages from ink! trait implementations.
///
/// # Usage
///
/// ```
/// use ink_lang as ink;
/// # use ink_lang::reflect::ContractAmountDispatchables;
///
/// #[ink::contract]
/// pub mod contract {
///     #[ink(storage)]
///     pub struct Contract {}
///
///     impl Contract {
///         #[ink(constructor)]
///         pub fn constructor1() -> Self { Contract {} }
///
///         #[ink(constructor)]
///         pub fn constructor2() -> Self { Contract {} }
///
///         #[ink(message)]
///         pub fn message1(&self) {}
///
///         #[ink(message)]
///         pub fn message2(&self) {}
///
///         #[ink(message)]
///         pub fn message3(&self) {}
///     }
/// }
///
/// use contract::Contract;
///
/// fn main() {
///     assert_eq!(<Contract as ContractAmountDispatchables>::CONSTRUCTORS, 2);
///     assert_eq!(<Contract as ContractAmountDispatchables>::MESSAGES, 3);
/// }
/// ```
pub trait ContractAmountDispatchables {
    /// The number of dispatchable ink! messages.
    const MESSAGES: usize;
    /// The number of dispatchable ink! constructors.
    const CONSTRUCTORS: usize;
}

/// Reflects the sequence of all dispatchable ink! messages of the ink! smart contract.
///
/// # Note
///
/// This is automatically implemented by all ink! smart contracts.
///
/// # Usage
///
/// ```
/// use ink_lang as ink;
/// # use ink_lang::reflect::ContractAmountDispatchables;
/// # use ink_lang::reflect::ContractDispatchableMessages;
///
/// #[ink::contract]
/// pub mod contract {
///     #[ink(storage)]
///     pub struct Contract {}
///
///     impl Contract {
///         #[ink(constructor)]
///         pub fn constructor1() -> Self { Contract {} }
///
///         #[ink(message, selector = 1234)]
///         pub fn message1(&self) {}
///
///         #[ink(message, selector = 0xC0DECAFE)]
///         pub fn message2(&self) {}
///
///         #[ink(message, selector = 42)]
///         pub fn message3(&self) {}
///     }
/// }
///
/// use contract::Contract;
///
/// fn main() {
///     assert_eq!(
///         <Contract as ContractDispatchableMessages<{
///             <Contract as ContractAmountDispatchables>::MESSAGES
///         }>>::IDS,
///         [1234, 0xC0DECAFE, 42],
///     );
/// }
/// ```
pub trait ContractDispatchableMessages<const AMOUNT: usize> {
    /// The sequence stores selector IDs of all ink! messages dispatchable by the ink! smart contract.
    const IDS: [u32; AMOUNT];
}

/// Implemented by all ink! smart contracts.
///
/// Stores a sequence of all dispatchable ink! constructors of the ink! smart contract.
///
/// # Note
///
/// Implemented for the amount of dispatchable ink! constructors of the ink! smart contract.
pub trait ContractDispatchableConstructors<const AMOUNT: usize> {
    /// The sequence stores selector IDs of all ink! constructors dispatchable by the ink! smart contract.
    const IDS: [u32; AMOUNT];
}

/// Implemented by the ink! message namespace type for every ink! message selector ID.
///
/// Stores various information properties of the respective dispatchable ink! message.
pub trait DispatchableMessageInfo<const ID: u32> {
    /// Reflects the input types of the dispatchable ink! message.
    type Input;
    /// Reflects the output type of the dispatchable ink! message.
    type Output;
    /// The ink! storage struct type.
    type Storage;

    /// The closure that can be used to dispatch into the dispatchable ink! message.
    ///
    /// # Note
    ///
    /// We unify `&self` and `&mut self` ink! messages here and always take a `&mut self`.
    /// This is mainly done for simplification but also because we can easily convert from
    /// `&mut self` to `&self` with our current dispatch codegen architecture.
    const CALLABLE: fn(&mut Self::Storage, Self::Input) -> Self::Output;

    /// Yields `true` if the dispatchable ink! message mutates the ink! storage.
    const MUTATES: bool;
    /// Yields `true` if the dispatchable ink! message is payable.
    const PAYABLE: bool;
    /// The selectors of the dispatchable ink! message.
    const SELECTOR: [u8; 4];
    /// The label of the dispatchable ink! message.
    const LABEL: &'static str;
}

/// Implemented by the ink! constructor namespace type for every ink! constructor selector ID.
///
/// Stores various information of the respective dispatchable ink! constructor.
pub trait DispatchableConstructorInfo<const ID: u32> {
    /// Reflects the input types of the dispatchable ink! constructor.
    type Input;
    /// The ink! storage struct type.
    type Storage;

    /// The closure that can be used to dispatch into the dispatchable ink! constructor.
    const CALLABLE: fn(Self::Input) -> Self::Storage;

    /// The selectors of the dispatchable ink! constructor.
    const SELECTOR: [u8; 4];
    /// The label of the dispatchable ink! constructor.
    const LABEL: &'static str;
}

/// Generated type used to decode all dispatchable ink! messages of the ink! smart contract.
pub trait ContractMessageDecoder {
    /// The ink! smart contract message decoder type.
    type Type: scale::Decode + ExecuteDispatchable;
}

/// Generated type used to decode all dispatchable ink! constructors of the ink! smart contract.
pub trait ContractConstructorDecoder {
    /// The ink! smart contract constructor decoder type.
    type Type: scale::Decode + ExecuteDispatchable;
}

/// Implemented by the ink! smart contract message or constructor decoder.
///
/// Starts the execution of the respective ink! message or constructor call.
pub trait ExecuteDispatchable {
    /// Executes the ink! smart contract message or constructor.
    fn execute_dispatchable(self) -> Result<(), DispatchError>;
}
