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
use core::marker::PhantomData;
use ink_env::{
    call::{
        utils::{
            ReturnType,
            Set,
        },
        CallBuilder,
        ExecutionInput,
    },
    Environment,
};

/// Trait used to indicate that an ink! trait definition has been checked
/// by the `#[ink::trait_definition]` procedural macro.
#[doc(hidden)]
pub unsafe trait TraitImplementer<const TRAIT_ID: u32> {}

/// Trait used by `#[ink::trait_definition]` to ensure that the associated
/// return type for each trait message is correct.
#[doc(hidden)]
pub trait ImpliesReturn<T> {}

impl<T> ImpliesReturn<T> for T {}
impl<T, E, Callee, GasCost, TransferredValue, Args> ImpliesReturn<T>
    for CallBuilder<
        E,
        Callee,
        GasCost,
        TransferredValue,
        Set<ExecutionInput<Args>>,
        Set<ReturnType<T>>,
    >
where
    E: Environment,
{
}

impl<E, Callee, GasCost, TransferredValue, Args> ImpliesReturn<()>
    for CallBuilder<
        E,
        Callee,
        GasCost,
        TransferredValue,
        Set<ExecutionInput<Args>>,
        Set<()>,
    >
where
    E: Environment,
{
}

/// This type is known to ink! to implement all defined ink! trait definitions.
/// This property can be guaranteed by `#[ink::trait_definition]` procedural macro.
///
/// By the introduction of an new internal and hidden associated type called
/// `__ink_DynamicCallForwarder` for all ink! trait definitions it is possible
/// for ink! to map from any given ink! trait definition back to a concrete
/// Rust type.
/// Whenever the `ChainExtensionRegistry` implements an ink! trait definition
/// all calls are defaulted to produce linker errors (ideally compiler errors
/// if that was possible) and the only relevant implementation is the new
/// `__ink_DynamicCallForwarder` associated type that links to a concrete
/// type implementing `FromAccountId` and the ink! trait definition with
/// proper implementations.
///
/// Then ink! can map from the ink! trait definition `MyTrait` to this concrete
/// dynamic call forwarder type by:
/// ```no_compile
/// <::ink_lang::TraitDefinitionRegistry as MyTrait>::__ink_DynamicCallForwarder
/// ```
/// Normal implementations of ink! trait definitions default the new
/// `__ink_DynamicCallForwarder` associated type to `::ink_lang::NoDynamicCallForwarder`.
///
/// This is the technique used by ink! to resolve `&dyn MyTrait`, `&mut dyn MyTrait`
/// in message parameters or `dyn MyTrait` in ink! storage fields to concrete types
/// that ink! can serialize and deserialize as if it was an `AccountId` and call
/// ink! messages on it according to the ink! trait definition interface.
#[doc(hidden)]
pub struct TraitDefinitionRegistry<E> {
    marker: PhantomData<fn() -> E>,
}

impl<E> crate::ContractEnv for TraitDefinitionRegistry<E>
where
    E: ink_env::Environment,
{
    type Env = E;
}

unsafe impl<E, const N: u32> TraitImplementer<N> for TraitDefinitionRegistry<E> {}

/// The global call builder type for an ink! trait definition.
pub trait TraitCallBuilder {
    /// The call builder type.
    type Builder;

    /// Returns a shared reference to the global call builder type.
    ///
    /// This allows to call `&self` ink! trait messages.
    fn call(&self) -> &Self::Builder;

    /// Returns an exclusive reference to the global call builder type.
    ///
    /// This allows to call any ink! trait message.
    fn call_mut(&mut self) -> &mut Self::Builder;
}

/// Implemented by the global trait info provider.
///
/// This communicates the `u32` number that uniquely identifies
/// the ink! trait definition.
pub trait TraitUniqueId {
    /// The unique trait `u32` identifier.
    const ID: u32;
}

/// Implemented by the global trait info provider.
///
/// It is used to query the global trait call forwarder.
/// There is one global trait call forwarder that implements
/// the call forwarding (short- and long-form) for all calls
/// to this trait in `ink-as-dependency` configuration.
pub trait TraitCallForwarder {
    /// The call forwarder type.
    type Forwarder: TraitCallBuilder;
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

/// Implemented by call builders of smart contracts.
///
/// These might be implementing multiple different ink! traits.
/// The codegen makes them implement this trait once for every
/// ink! trait they have to implement.
///
/// While the trait is not necessary it encapsulates a lot of
/// utility and auxiliary code required for the actual ink! trait
/// implementations.
pub trait TraitCallForwarderFor<const ID: u32> {
    type Forwarder: TraitCallBuilder;

    /// Forwards the `&self` call.
    ///
    /// # Note
    ///
    /// This is used for the short-hand calling syntax.
    fn forward(&self) -> &Self::Forwarder;

    /// Forwards the `&mut self` call.
    ///
    /// # Note
    ///
    /// This is used for the short-hand calling syntax.
    fn forward_mut(&mut self) -> &mut Self::Forwarder;

    /// Builds up the `&self` call.
    ///
    /// # Note
    ///
    /// This is used for the long-hand calling syntax.
    fn build(&self) -> &<Self::Forwarder as TraitCallBuilder>::Builder;

    /// Builds up the `&mut self` call.
    ///
    /// # Note
    ///
    /// This is used for the long-hand calling syntax.
    fn build_mut(&mut self) -> &mut <Self::Forwarder as TraitCallBuilder>::Builder;
}

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
pub trait TraitMessageInfo<const TRAIT_LOCAL_MESSAGE_ID: u32> {
    /// Is `true` if the ink! trait message has been annotated with `#[ink(payable)]`.
    const PAYABLE: bool;

    /// The unique selector of the ink! trait message.
    ///
    /// This might have been adjusted using `#[ink(selector = N:u32)]` at the
    /// ink! trait definition site.
    const SELECTOR: [u8; 4];
}

/// Implemented by all ink! smart contracts.
///
/// Reflects the number of dispatchable ink! messages and constructors respectively.
pub trait ContractAmountDispatchables {
    /// The number of dispatchable ink! messages.
    const MESSAGES: usize;
    /// The number of dispatchable ink! constructors.
    const CONSTRUCTORS: usize;
}

/// Implemented by all ink! smart contracts.
///
/// Stores a sequence of all dispatchable ink! message of the ink! smart contract.
///
/// # Note
///
/// Implemented for the amount of dispatchable ink! messages of the ink! smart contract.
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
