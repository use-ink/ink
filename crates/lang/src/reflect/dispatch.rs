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

use core::fmt::Display;

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
/// # use ink_lang::selector_id;
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
///         #[ink(message)]
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
///         [1234, 0xC0DECAFE, selector_id!("message3")],
///     );
/// }
/// ```
pub trait ContractDispatchableMessages<const AMOUNT: usize> {
    /// The sequence stores selector IDs of all ink! messages dispatchable by the ink! smart contract.
    const IDS: [u32; AMOUNT];
}

/// Reflects the sequence of all dispatchable ink! constructors of the ink! smart contract.
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
/// # use ink_lang::reflect::ContractDispatchableConstructors;
/// # use ink_lang::selector_id;
///
/// #[ink::contract]
/// pub mod contract {
///     #[ink(storage)]
///     pub struct Contract {}
///
///     impl Contract {
///         #[ink(constructor, selector = 1234)]
///         pub fn constructor1() -> Self { Contract {} }
///
///         #[ink(constructor, selector = 0xC0DECAFE)]
///         pub fn constructor2() -> Self { Contract {} }
///
///         #[ink(constructor)]
///         pub fn constructor3() -> Self { Contract {} }
///
///         #[ink(message)]
///         pub fn message1(&self) {}
///     }
/// }
///
/// use contract::Contract;
///
/// fn main() {
///     assert_eq!(
///         <Contract as ContractDispatchableConstructors<{
///             <Contract as ContractAmountDispatchables>::CONSTRUCTORS
///         }>>::IDS,
///         [1234, 0xC0DECAFE, selector_id!("constructor3")],
///     );
/// }
/// ```
pub trait ContractDispatchableConstructors<const AMOUNT: usize> {
    /// The sequence stores selector IDs of all ink! constructors dispatchable by the ink! smart contract.
    const IDS: [u32; AMOUNT];
}

/// Stores various information of the respective dispatchable ink! message.
///
/// # Note
///
/// This trait is implemented by ink! for every dispatchable ink! message
/// of the root ink! smart contract. The `ID` used in the trait reflects the
/// chosen or derived selector of the dispatchable ink! message.
///
/// # Usage
///
/// ```
/// use ink_lang as ink;
/// # use ink_lang::reflect::DispatchableMessageInfo;
/// # use ink_lang::{selector_id, selector_bytes};
///
/// #[ink::contract]
/// pub mod contract {
///     #[ink(storage)]
///     pub struct Contract {}
///
///     impl Contract {
///         #[ink(constructor)]
///         pub fn constructor() -> Self { Contract {} }
///
///         #[ink(message)]
///         pub fn message1(&self) {}
///
///         #[ink(message, payable, selector = 0xC0DECAFE)]
///         pub fn message2(&mut self, input1: i32, input2: i64) -> (bool, i32) {
///             unimplemented!()
///         }
///     }
/// }
///
/// use contract::Contract;
///
/// /// Asserts that the message with the selector `ID` has the following properties.
/// ///
/// /// # Note
/// ///
/// /// The `In` and `Out` generic parameters describe the input and output types.
/// fn assert_message_info<In, Out, const ID: u32>(
///     mutates: bool,
///     payable: bool,
///     selector: [u8; 4],
///     label: &str,
/// )
/// where
///     Contract: DispatchableMessageInfo<{ID}, Input = In, Output = Out>,
/// {
///     assert_eq!(<Contract as DispatchableMessageInfo<{ID}>>::MUTATES, mutates);
///     assert_eq!(<Contract as DispatchableMessageInfo<{ID}>>::PAYABLE, payable);
///     assert_eq!(
///         <Contract as DispatchableMessageInfo<{ID}>>::SELECTOR,
///         selector,
///     );
///     assert_eq!(
///         <Contract as DispatchableMessageInfo<{ID}>>::LABEL,
///         label,
///     );
/// }
///
/// fn main() {
///     assert_message_info::<(), (), {selector_id!("message1")}>(
///         false, false, selector_bytes!("message1"), "message1"
///     );
///     assert_message_info::<(i32, i64), (bool, i32), 0xC0DECAFE_u32>(
///         true, true, [0xC0, 0xDE, 0xCA, 0xFE], "message2"
///     );
/// }
/// ```
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

/// Stores various information of the respective dispatchable ink! constructor.
///
/// # Note
///
/// This trait is implemented by ink! for every dispatchable ink! constructor
/// of the root ink! smart contract. The `ID` used in the trait reflects the
/// chosen or derived selector of the dispatchable ink! constructor.
///
/// # Usage
///
/// ```
/// use ink_lang as ink;
/// # use ink_lang::reflect::DispatchableConstructorInfo;
/// # use ink_lang::{selector_id, selector_bytes};
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
///         #[ink(constructor, selector = 0xC0DECAFE)]
///         pub fn constructor2(input1: i32, input2: i64) -> Self {
///             Contract {}
///         }
///
///         #[ink(message)]
///         pub fn message(&self) {}
///     }
/// }
///
/// use contract::Contract;
///
/// /// Asserts that the constructor with the selector `ID` has the following properties.
/// ///
/// /// # Note
/// ///
/// /// The `In` and `Out` generic parameters describe the input and output types.
/// fn assert_constructor_info<In, const ID: u32>(
///     selector: [u8; 4],
///     label: &str,
/// )
/// where
///     Contract: DispatchableConstructorInfo<{ID}, Input = In>,
/// {
///     assert_eq!(
///         <Contract as DispatchableConstructorInfo<{ID}>>::SELECTOR,
///         selector,
///     );
///     assert_eq!(
///         <Contract as DispatchableConstructorInfo<{ID}>>::LABEL,
///         label,
///     );
/// }
///
/// fn main() {
///     assert_constructor_info::<(), {selector_id!("constructor1")}>(
///         selector_bytes!("constructor1"), "constructor1"
///     );
///     assert_constructor_info::<(i32, i64), 0xC0DECAFE_u32>(
///         [0xC0, 0xDE, 0xCA, 0xFE], "constructor2"
///     );
/// }
/// ```
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
///
/// # Note
///
/// The decoder follows the ink! calling ABI where all ink! message calls start with
/// 4 bytes dedicated to the ink! message selector followed by the SCALE encoded parameters.
///
/// # Usage
///
/// ```
/// use ink_lang as ink;
/// # use ink_lang::reflect::ContractMessageDecoder;
/// # use ink_lang::selector_bytes;
/// # use scale::{Encode, Decode};
///
/// #[ink::contract]
/// pub mod contract {
///     #[ink(storage)]
///     pub struct Contract {}
///
///     impl Contract {
///         #[ink(constructor)]
///         pub fn constructor() -> Self { Self {} }
///
///         #[ink(message)]
///         pub fn message1(&self) {}
///
///         #[ink(message)]
///         pub fn message2(&self, input1: bool, input2: i32) {}
///     }
/// }
///
/// use contract::Contract;
///
/// fn main() {
///     // Call to `message1` without input parameters.
///     {
///         let mut input_bytes = Vec::new();
///         input_bytes.extend(selector_bytes!("message1"));
///         assert!(
///             <<Contract as ContractMessageDecoder>::Type as Decode>::decode(
///                 &mut &input_bytes[..]).is_ok()
///         );
///     }
///     // Call to `message2` with 2 parameters.
///     {
///         let mut input_bytes = Vec::new();
///         input_bytes.extend(selector_bytes!("message2"));
///         input_bytes.extend(true.encode());
///         input_bytes.extend(42i32.encode());
///         assert!(
///             <<Contract as ContractMessageDecoder>::Type as Decode>::decode(
///                 &mut &input_bytes[..]).is_ok()
///         );
///     }
///     // Call with invalid ink! message selector.
///     {
///         let mut input_bytes = Vec::new();
///         input_bytes.extend(selector_bytes!("non_existing_message"));
///         assert!(
///             <<Contract as ContractMessageDecoder>::Type as Decode>::decode(
///                 &mut &input_bytes[..]).is_err()
///         );
///     }
///     // Call with invalid ink! message parameters.
///     {
///         let mut input_bytes = Vec::new();
///         input_bytes.extend(selector_bytes!("message2"));
///         assert!(
///             <<Contract as ContractMessageDecoder>::Type as Decode>::decode(
///                 &mut &input_bytes[..]).is_err()
///         );
///     }
/// }
/// ```
pub trait ContractMessageDecoder {
    /// The ink! smart contract message decoder type.
    type Type: scale::Decode + ExecuteDispatchable;
}

/// Generated type used to decode all dispatchable ink! constructors of the ink! smart contract.
///
/// # Note
///
/// The decoder follows the ink! calling ABI where all ink! constructor calls start with
/// 4 bytes dedicated to the ink! constructor selector followed by the SCALE encoded parameters.
///
/// # Usage
///
/// ```
/// use ink_lang as ink;
/// # use ink_lang::reflect::ContractConstructorDecoder;
/// # use ink_lang::selector_bytes;
/// # use scale::{Encode, Decode};
///
/// #[ink::contract]
/// pub mod contract {
///     #[ink(storage)]
///     pub struct Contract {}
///
///     impl Contract {
///         #[ink(constructor)]
///         pub fn constructor1() -> Self { Self {} }
///
///         #[ink(constructor)]
///         pub fn constructor2(input1: bool, input2: i32) -> Self { Self {} }
///
///         #[ink(message)]
///         pub fn message(&self) {}
///     }
/// }
///
/// use contract::Contract;
///
/// fn main() {
///     // Call to `constructor1` without input parameters.
///     {
///         let mut input_bytes = Vec::new();
///         input_bytes.extend(selector_bytes!("constructor1"));
///         assert!(
///             <<Contract as ContractConstructorDecoder>::Type as Decode>::decode(
///                 &mut &input_bytes[..]).is_ok()
///         );
///     }
///     // Call to `constructor2` with 2 parameters.
///     {
///         let mut input_bytes = Vec::new();
///         input_bytes.extend(selector_bytes!("constructor2"));
///         input_bytes.extend(true.encode());
///         input_bytes.extend(42i32.encode());
///         assert!(
///             <<Contract as ContractConstructorDecoder>::Type as Decode>::decode(
///                 &mut &input_bytes[..]).is_ok()
///         );
///     }
///     // Call with invalid ink! constructor selector.
///     {
///         let mut input_bytes = Vec::new();
///         input_bytes.extend(selector_bytes!("non_existing_constructor"));
///         assert!(
///             <<Contract as ContractConstructorDecoder>::Type as Decode>::decode(
///                 &mut &input_bytes[..]).is_err()
///         );
///     }
///     // Call with invalid ink! constructor parameters.
///     {
///         let mut input_bytes = Vec::new();
///         input_bytes.extend(selector_bytes!("constructor2"));
///         assert!(
///             <<Contract as ContractConstructorDecoder>::Type as Decode>::decode(
///                 &mut &input_bytes[..]).is_err()
///         );
///     }
/// }
/// ```
pub trait ContractConstructorDecoder {
    /// The ink! smart contract constructor decoder type.
    type Type: DecodeDispatch + ExecuteDispatchable;
}

/// Starts the execution of the respective ink! message or constructor call.
///
/// # Note
///
/// Implemented by the ink! smart contract message or constructor decoder.
pub trait ExecuteDispatchable {
    /// Executes the ink! smart contract message or constructor.
    fn execute_dispatchable(self) -> Result<(), DispatchError>;
}

/// An error that can occur during dispatch of ink! dispatchables.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum DispatchError {
    /// Failed to decode into a valid dispatch selector.
    InvalidSelector,
    /// The decoded selector is not known to the dispatch decoder.
    UnknownSelector,
    /// Failed to decode the parameters for the selected dispatchable.
    InvalidParameters,
    /// Failed to read execution input for the dispatchable.
    CouldNotReadInput,
    /// Invalidly paid an unpayable dispatchable.
    PaidUnpayableMessage,
}

impl Display for DispatchError {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl DispatchError {
    /// Returns a string representation of the error.
    #[inline]
    fn as_str(&self) -> &'static str {
        match self {
            Self::InvalidSelector => "unable to decode selector",
            Self::UnknownSelector => "encountered unknown selector",
            Self::InvalidParameters => "unable to decode input",
            Self::CouldNotReadInput => "could not read input",
            Self::PaidUnpayableMessage => "paid an unpayable message",
        }
    }
}

impl From<DispatchError> for scale::Error {
    #[inline]
    fn from(error: DispatchError) -> Self {
        Self::from(error.as_str())
    }
}

/// Decodes an ink! dispatch input into a known selector and its expected parameters.
///
/// # Note
///
/// This trait is automatically implemented for ink! message and constructor decoders.
///
/// # Errors
///
/// Returns an error if any of the decode steps failed:
///
/// - `InvalidSelector`: The first four bytes could not properly decoded into the selector.
/// - `UnknownSelector`: The decoded selector did not match any of the expected ones.
/// - `InvalidParameters`: Failed to decoded the parameters for the selected dispatchable.
///
/// The other dispatch errors are handled by other structures usually.
///
/// # Usage
///
/// ```
/// use ink_lang as ink;
/// # use ink_lang::reflect::{ContractMessageDecoder, DecodeDispatch, DispatchError};
/// # use ink_lang::selector_bytes;
/// # use scale::Encode;
///
/// #[ink::contract]
/// pub mod contract {
///     #[ink(storage)]
///     pub struct Contract {}
///
///     impl Contract {
///         #[ink(constructor)]
///         pub fn constructor() -> Self { Self {} }
///
///         #[ink(message)]
///         pub fn message(&self, input_1: bool, input_2: i32) {}
///     }
/// }
///
/// use contract::Contract;
///
/// fn main() {
///     // Valid call to `message`:
///     {
///         let mut input_bytes = Vec::new();
///         input_bytes.extend(selector_bytes!("message"));
///         input_bytes.extend(true.encode());
///         input_bytes.extend(42i32.encode());
///         assert!(
///             <<Contract as ContractMessageDecoder>::Type as DecodeDispatch>::decode_dispatch(
///                 &mut &input_bytes[..]).is_ok()
///         );
///     }
///     // Invalid call with invalid selector (or empty input).
///     {
///         let mut input_bytes = Vec::new();
///         assert_eq!(
///             <<Contract as ContractMessageDecoder>::Type
///                 as DecodeDispatch>::decode_dispatch(&mut &input_bytes[..])
///                 # .map(|_| ())
///                 .unwrap_err(),
///             DispatchError::InvalidSelector,
///         );
///     }
///     // Invalid call to `message` with unknown selector.
///     {
///         let mut input_bytes = Vec::new();
///         input_bytes.extend(selector_bytes!("unknown_selector"));
///         assert_eq!(
///             <<Contract as ContractMessageDecoder>::Type
///                 as DecodeDispatch>::decode_dispatch(&mut &input_bytes[..])
///                 # .map(|_| ())
///                 .unwrap_err(),
///             DispatchError::UnknownSelector,
///         );
///     }
///     // Invalid call to `message` with invalid (or missing) parameters.
///     {
///         let mut input_bytes = Vec::new();
///         input_bytes.extend(selector_bytes!("message"));
///         assert_eq!(
///             <<Contract as ContractMessageDecoder>::Type
///                 as DecodeDispatch>::decode_dispatch(&mut &input_bytes[..])
///                 # .map(|_| ())
///                 .unwrap_err(),
///             DispatchError::InvalidParameters,
///         );
///     }
/// }
/// ```
pub trait DecodeDispatch: scale::Decode {
    fn decode_dispatch<I: scale::Input>(input: &mut I) -> Result<Self, DispatchError>;
}
