// Copyright 2018-2020 Parity Technologies (UK) Ltd.
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

#![allow(clippy::new_ret_no_self)]

use crate::utils::serialize_as_byte_str;
#[cfg(not(feature = "std"))]
use alloc::{
    format,
    vec,
    vec::Vec,
};
use core::marker::PhantomData;
use scale_info::{
    form::{
        CompactForm,
        Form,
        MetaForm,
    },
    meta_type,
    IntoCompact,
    Registry,
    TypeInfo,
};
use serde::Serialize;

/// Describes a contract.
#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(bound = "F::TypeId: Serialize")]
pub struct ContractSpec<F: Form = MetaForm> {
    /// The set of constructors of the contract.
    constructors: Vec<ConstructorSpec<F>>,
    /// The external messages of the contract.
    messages: Vec<MessageSpec<F>>,
    /// The events of the contract.
    events: Vec<EventSpec<F>>,
    /// The contract documentation.
    docs: Vec<&'static str>,
}

impl IntoCompact for ContractSpec {
    type Output = ContractSpec<CompactForm>;

    fn into_compact(self, registry: &mut Registry) -> Self::Output {
        ContractSpec {
            constructors: self
                .constructors
                .into_iter()
                .map(|constructor| constructor.into_compact(registry))
                .collect::<Vec<_>>(),
            messages: self
                .messages
                .into_iter()
                .map(|msg| msg.into_compact(registry))
                .collect::<Vec<_>>(),
            events: self
                .events
                .into_iter()
                .map(|event| event.into_compact(registry))
                .collect::<Vec<_>>(),
            docs: self.docs,
        }
    }
}

/// The message builder is ready to finalize construction.
pub enum Valid {}
/// The message builder is not ready to finalize construction.
pub enum Invalid {}

/// A builder for contracts.
pub struct ContractSpecBuilder<S = Invalid> {
    /// The to-be-constructed contract specification.
    spec: ContractSpec,
    /// Marker for compile-time checking of valid contract specifications.
    marker: PhantomData<fn() -> S>,
}

impl ContractSpecBuilder<Invalid> {
    /// Sets the constructors of the contract specification.
    pub fn constructors<C>(self, constructors: C) -> ContractSpecBuilder<Valid>
    where
        C: IntoIterator<Item = ConstructorSpec>,
    {
        debug_assert!(self.spec.constructors.is_empty());
        ContractSpecBuilder {
            spec: ContractSpec {
                constructors: constructors.into_iter().collect::<Vec<_>>(),
                ..self.spec
            },
            marker: Default::default(),
        }
    }
}

impl<S> ContractSpecBuilder<S> {
    /// Sets the messages of the contract specification.
    pub fn messages<M>(self, messages: M) -> Self
    where
        M: IntoIterator<Item = MessageSpec>,
    {
        debug_assert!(self.spec.messages.is_empty());
        Self {
            spec: ContractSpec {
                messages: messages.into_iter().collect::<Vec<_>>(),
                ..self.spec
            },
            ..self
        }
    }

    /// Sets the events of the contract specification.
    pub fn events<E>(self, events: E) -> Self
    where
        E: IntoIterator<Item = EventSpec>,
    {
        debug_assert!(self.spec.events.is_empty());
        Self {
            spec: ContractSpec {
                events: events.into_iter().collect::<Vec<_>>(),
                ..self.spec
            },
            ..self
        }
    }

    /// Sets the documentation of the contract specification.
    pub fn docs<D>(self, docs: D) -> Self
    where
        D: IntoIterator<Item = &'static str>,
    {
        debug_assert!(self.spec.docs.is_empty());
        Self {
            spec: ContractSpec {
                docs: docs.into_iter().collect::<Vec<_>>(),
                ..self.spec
            },
            ..self
        }
    }
}

impl ContractSpecBuilder<Valid> {
    /// Finalizes construction of the contract specification.
    pub fn done(self) -> ContractSpec {
        assert!(
            !self.spec.constructors.is_empty(),
            "must have at least one constructor"
        );
        assert!(
            !self.spec.messages.is_empty(),
            "must have at least one message"
        );
        self.spec
    }
}

impl ContractSpec {
    /// Creates a new contract specification.
    pub fn new() -> ContractSpecBuilder {
        ContractSpecBuilder {
            spec: Self {
                constructors: Vec::new(),
                messages: Vec::new(),
                events: Vec::new(),
                docs: Vec::new(),
            },
            marker: PhantomData,
        }
    }
}

/// Describes a constructor of a contract.
#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(bound = "F::TypeId: Serialize")]
pub struct ConstructorSpec<F: Form = MetaForm> {
    /// The name of the message.
    ///
    /// In case of a trait provided constructor the trait name is prefixed.
    name: Vec<&'static str>,
    /// The selector hash of the message.
    #[serde(serialize_with = "serialize_as_byte_str")]
    selector: [u8; 4],
    /// The parameters of the deploy handler.
    args: Vec<MessageParamSpec<F>>,
    /// The deploy handler documentation.
    docs: Vec<&'static str>,
}

impl IntoCompact for ConstructorSpec {
    type Output = ConstructorSpec<CompactForm>;

    fn into_compact(self, registry: &mut Registry) -> Self::Output {
        ConstructorSpec {
            name: self.name,
            selector: self.selector,
            args: self
                .args
                .into_iter()
                .map(|arg| arg.into_compact(registry))
                .collect::<Vec<_>>(),
            docs: self.docs,
        }
    }
}

/// A builder for constructors.
///
/// # Dev
///
/// Some of the fields are guarded by a type-state pattern to
/// fail at compile-time instead of at run-time. This is useful
/// to better debug code-gen macros.
pub struct ConstructorSpecBuilder<Selector> {
    spec: ConstructorSpec,
    marker: PhantomData<fn() -> Selector>,
}

impl ConstructorSpec {
    /// Creates a new constructor spec builder.
    fn from_name_segments(
        segments: Vec<&'static str>,
    ) -> ConstructorSpecBuilder<Missing<state::Selector>> {
        ConstructorSpecBuilder {
            spec: Self {
                name: segments,
                selector: [0u8; 4],
                args: Vec::new(),
                docs: Vec::new(),
            },
            marker: PhantomData,
        }
    }

    /// Creates a new constructor spec builder.
    pub fn name(name: &'static str) -> ConstructorSpecBuilder<Missing<state::Selector>> {
        Self::from_name_segments(vec![name])
    }

    /// Creates a new constructor spec builder for a trait provided constructor.
    pub fn trait_and_name(
        trait_name: &'static str,
        constructor_name: &'static str,
    ) -> ConstructorSpecBuilder<Missing<state::Selector>> {
        Self::from_name_segments(vec![trait_name, constructor_name])
    }
}

impl ConstructorSpecBuilder<Missing<state::Selector>> {
    /// Sets the function selector of the message.
    pub fn selector(self, selector: [u8; 4]) -> ConstructorSpecBuilder<state::Selector> {
        ConstructorSpecBuilder {
            spec: ConstructorSpec {
                selector,
                ..self.spec
            },
            marker: PhantomData,
        }
    }
}

impl<S> ConstructorSpecBuilder<S> {
    /// Sets the input arguments of the message specification.
    pub fn args<A>(self, args: A) -> Self
    where
        A: IntoIterator<Item = MessageParamSpec>,
    {
        let mut this = self;
        debug_assert!(this.spec.args.is_empty());
        this.spec.args = args.into_iter().collect::<Vec<_>>();
        this
    }

    /// Sets the documentation of the message specification.
    pub fn docs<D>(self, docs: D) -> Self
    where
        D: IntoIterator<Item = &'static str>,
    {
        let mut this = self;
        debug_assert!(this.spec.docs.is_empty());
        this.spec.docs = docs.into_iter().collect::<Vec<_>>();
        this
    }
}

impl ConstructorSpecBuilder<state::Selector> {
    /// Finishes construction of the constructor.
    pub fn done(self) -> ConstructorSpec {
        self.spec
    }
}

/// Describes a contract message.
#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(bound = "F::TypeId: Serialize")]
#[serde(rename_all = "camelCase")]
pub struct MessageSpec<F: Form = MetaForm> {
    /// The name of the message and some optional prefixes.
    ///
    /// In case of trait provided messages and constructors the prefix
    /// by convention in ink! is the name of the trait.
    name: Vec<&'static str>,
    /// The selector hash of the message.
    #[serde(serialize_with = "serialize_as_byte_str")]
    selector: [u8; 4],
    /// If the message is allowed to mutate the contract state.
    mutates: bool,
    /// If the message is payable by the caller.
    payable: bool,
    /// The parameters of the message.
    args: Vec<MessageParamSpec<F>>,
    /// The return type of the message.
    return_type: ReturnTypeSpec<F>,
    /// The message documentation.
    docs: Vec<&'static str>,
}

/// Type state for builders to tell that some mandatory state has not yet been set
/// yet or to fail upon setting the same state multiple times.
pub struct Missing<S>(PhantomData<fn() -> S>);

mod state {
    //! Type states that tell what state of a message has not
    //! yet been set properly for a valid construction.

    /// Type state for the message selector of a message.
    pub struct Selector;
    /// Type state for the mutability of a message.
    pub struct Mutates;
    /// Type state for telling if the message is payable.
    pub struct IsPayable;
    /// Type state for the return type of a message.
    pub struct Returns;
}

impl MessageSpec {
    /// Creates a new message spec from the given name segments.
    fn from_name_segments(
        segments: Vec<&'static str>,
    ) -> MessageSpecBuilder<
        Missing<state::Selector>,
        Missing<state::Mutates>,
        Missing<state::IsPayable>,
        Missing<state::Returns>,
    > {
        MessageSpecBuilder {
            spec: Self {
                name: segments,
                selector: [0u8; 4],
                mutates: false,
                payable: false,
                args: Vec::new(),
                return_type: ReturnTypeSpec::new(None),
                docs: Vec::new(),
            },
            marker: PhantomData,
        }
    }

    /// Creates a new message spec builder.
    pub fn name(
        name: &'static str,
    ) -> MessageSpecBuilder<
        Missing<state::Selector>,
        Missing<state::Mutates>,
        Missing<state::IsPayable>,
        Missing<state::Returns>,
    > {
        Self::from_name_segments(vec![name])
    }

    /// Creates a new message spec builder for a trait provided message.
    pub fn trait_and_name(
        trait_name: &'static str,
        message_name: &'static str,
    ) -> MessageSpecBuilder<
        Missing<state::Selector>,
        Missing<state::Mutates>,
        Missing<state::IsPayable>,
        Missing<state::Returns>,
    > {
        Self::from_name_segments(vec![trait_name, message_name])
    }
}

/// A builder for messages.
///
/// # Dev
///
/// Some of the fields are guarded by a type-state pattern to
/// fail at compile-time instead of at run-time. This is useful
/// to better debug code-gen macros.
#[allow(clippy::type_complexity)]
pub struct MessageSpecBuilder<Selector, Mutates, IsPayable, Returns> {
    spec: MessageSpec,
    marker: PhantomData<fn() -> (Selector, Mutates, IsPayable, Returns)>,
}

impl<M, P, R> MessageSpecBuilder<Missing<state::Selector>, M, P, R> {
    /// Sets the function selector of the message.
    pub fn selector(
        self,
        selector: [u8; 4],
    ) -> MessageSpecBuilder<state::Selector, M, P, R> {
        MessageSpecBuilder {
            spec: MessageSpec {
                selector,
                ..self.spec
            },
            marker: PhantomData,
        }
    }
}

impl<S, P, R> MessageSpecBuilder<S, Missing<state::Mutates>, P, R> {
    /// Sets if the message is mutable, thus taking `&mut self` or not thus taking `&self`.
    pub fn mutates(self, mutates: bool) -> MessageSpecBuilder<S, state::Mutates, P, R> {
        MessageSpecBuilder {
            spec: MessageSpec {
                mutates,
                ..self.spec
            },
            marker: PhantomData,
        }
    }
}

impl<S, M, R> MessageSpecBuilder<S, M, Missing<state::IsPayable>, R> {
    /// Sets if the message is mutable, thus taking `&mut self` or not thus taking `&self`.
    pub fn payable(
        self,
        is_payable: bool,
    ) -> MessageSpecBuilder<S, M, state::IsPayable, R> {
        MessageSpecBuilder {
            spec: MessageSpec {
                payable: is_payable,
                ..self.spec
            },
            marker: PhantomData,
        }
    }
}

impl<M, S, P> MessageSpecBuilder<S, M, P, Missing<state::Returns>> {
    /// Sets the return type of the message.
    pub fn returns(
        self,
        return_type: ReturnTypeSpec,
    ) -> MessageSpecBuilder<S, M, P, state::Returns> {
        MessageSpecBuilder {
            spec: MessageSpec {
                return_type,
                ..self.spec
            },
            marker: PhantomData,
        }
    }
}

impl<S, M, P, R> MessageSpecBuilder<S, M, P, R> {
    /// Sets the input arguments of the message specification.
    pub fn args<A>(self, args: A) -> Self
    where
        A: IntoIterator<Item = MessageParamSpec>,
    {
        let mut this = self;
        debug_assert!(this.spec.args.is_empty());
        this.spec.args = args.into_iter().collect::<Vec<_>>();
        this
    }

    /// Sets the documentation of the message specification.
    pub fn docs<D>(self, docs: D) -> Self
    where
        D: IntoIterator<Item = &'static str>,
    {
        let mut this = self;
        debug_assert!(this.spec.docs.is_empty());
        this.spec.docs = docs.into_iter().collect::<Vec<_>>();
        this
    }
}

impl
    MessageSpecBuilder<state::Selector, state::Mutates, state::IsPayable, state::Returns>
{
    /// Finishes construction of the message.
    pub fn done(self) -> MessageSpec {
        self.spec
    }
}

impl IntoCompact for MessageSpec {
    type Output = MessageSpec<CompactForm>;

    fn into_compact(self, registry: &mut Registry) -> Self::Output {
        MessageSpec {
            name: self.name,
            selector: self.selector,
            mutates: self.mutates,
            payable: self.payable,
            args: self
                .args
                .into_iter()
                .map(|arg| arg.into_compact(registry))
                .collect::<Vec<_>>(),
            return_type: self.return_type.into_compact(registry),
            docs: self.docs,
        }
    }
}

/// Describes an event definition.
#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(bound = "F::TypeId: Serialize")]
pub struct EventSpec<F: Form = MetaForm> {
    /// The name of the event.
    name: &'static str,
    /// The event arguments.
    args: Vec<EventParamSpec<F>>,
    /// The event documentation.
    docs: Vec<&'static str>,
}

/// An event specification builder.
pub struct EventSpecBuilder {
    spec: EventSpec,
}

impl EventSpecBuilder {
    /// Sets the input arguments of the event specification.
    pub fn args<A>(self, args: A) -> Self
    where
        A: IntoIterator<Item = EventParamSpec>,
    {
        let mut this = self;
        debug_assert!(this.spec.args.is_empty());
        this.spec.args = args.into_iter().collect::<Vec<_>>();
        this
    }

    /// Sets the input arguments of the event specification.
    pub fn docs<D>(self, docs: D) -> Self
    where
        D: IntoIterator<Item = &'static str>,
    {
        let mut this = self;
        debug_assert!(this.spec.docs.is_empty());
        this.spec.docs = docs.into_iter().collect::<Vec<_>>();
        this
    }

    /// Finalizes building the event specification.
    pub fn done(self) -> EventSpec {
        self.spec
    }
}

impl IntoCompact for EventSpec {
    type Output = EventSpec<CompactForm>;

    fn into_compact(self, registry: &mut Registry) -> Self::Output {
        EventSpec {
            name: self.name,
            args: self
                .args
                .into_iter()
                .map(|arg| arg.into_compact(registry))
                .collect::<Vec<_>>(),
            docs: self.docs,
        }
    }
}

impl EventSpec {
    /// Creates a new event specification builder.
    pub fn new(name: &'static str) -> EventSpecBuilder {
        EventSpecBuilder {
            spec: Self {
                name,
                args: Vec::new(),
                docs: Vec::new(),
            },
        }
    }
}

/// Describes the syntactical name of a type at a given type position.
///
/// This is important when trying to work with type aliases.
/// Normally a type alias is transparent and so scenarios such as
/// ```no_compile
/// type Foo = i32;
/// fn bar(foo: Foo);
/// ```
/// Will only communicate that `foo` is of type `i32` which is correct,
/// however, it will miss the potentially important information that it
/// is being used through a type alias named `Foo`.
///
/// In ink! we current experience this problem with environmental types
/// such as the `Balance` type that is just a type alias to `u128` in the
/// default setup. Even though it would be useful for third party tools
/// such as the Polkadot UI to know that we are handling with `Balance`
/// types, we currently cannot communicate this without display names.
pub type DisplayName = scale_info::Path;

/// A type specification.
///
/// This contains the actual type as well as an optional compile-time
/// known displayed representation of the type. This is useful for cases
/// where the type is used through a type alias in order to provide
/// information about the alias name.
///
/// # Examples
///
/// Consider the following Rust function:
/// ```no_compile
/// fn is_sorted(input: &[i32], pred: Predicate) -> bool;
/// ```
/// In this above example `input` would have no displayable name,
/// `pred`'s display name is `Predicate` and the display name of
/// the return type is simply `bool`. Note that `Predicate` could
/// simply be a type alias to `fn(i32, i32) -> Ordering`.
#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(bound = "F::TypeId: Serialize")]
#[serde(rename_all = "camelCase")]
pub struct TypeSpec<F: Form = MetaForm> {
    /// The actual type.
    id: F::TypeId,
    /// The compile-time known displayed representation of the type.
    display_name: DisplayName,
}

impl IntoCompact for TypeSpec {
    type Output = TypeSpec<CompactForm>;

    fn into_compact(self, registry: &mut Registry) -> Self::Output {
        TypeSpec {
            id: registry.register_type(&self.id),
            display_name: self.display_name,
        }
    }
}

impl TypeSpec {
    /// Creates a new type specification with a display name.
    ///
    /// The name is any valid Rust identifier or path.
    ///
    /// # Examples
    ///
    /// Valid display names are `foo`, `foo::bar`, `foo::bar::Baz`, etc.
    ///
    /// # Panics
    ///
    /// Panics if the given display name is invalid.
    pub fn with_name_str<T>(display_name: &'static str) -> Self
    where
        T: TypeInfo + 'static,
    {
        Self::with_name_segs::<T, _>(display_name.split("::"))
    }

    /// Creates a new type specification with a display name
    /// represented by the given path segments.
    ///
    /// The display name segments all must be valid Rust identifiers.
    ///
    /// # Examples
    ///
    /// Valid display names are `foo`, `foo::bar`, `foo::bar::Baz`, etc.
    ///
    /// # Panics
    ///
    /// Panics if the given display name is invalid.
    pub fn with_name_segs<T, S>(segments: S) -> Self
    where
        T: TypeInfo + 'static,
        S: IntoIterator<Item = &'static str>,
    {
        Self {
            id: meta_type::<T>(),
            display_name: DisplayName::from_segments(segments)
                .expect("display name is invalid"),
        }
    }

    /// Creates a new type specification without a display name.
    pub fn new<T>() -> Self
    where
        T: TypeInfo + 'static,
    {
        Self {
            id: meta_type::<T>(),
            display_name: DisplayName::default(),
        }
    }
}

/// Describes a pair of parameter name and type.
#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(bound = "F::TypeId: Serialize")]
pub struct EventParamSpec<F: Form = MetaForm> {
    /// The name of the parameter.
    name: &'static str,
    /// If the event parameter is indexed.
    indexed: bool,
    /// The type of the parameter.
    #[serde(rename = "type")]
    ty: TypeSpec<F>,
    /// The documentation associated with the arguments.
    docs: Vec<&'static str>,
}

impl IntoCompact for EventParamSpec {
    type Output = EventParamSpec<CompactForm>;

    fn into_compact(self, registry: &mut Registry) -> Self::Output {
        EventParamSpec {
            name: self.name,
            indexed: self.indexed,
            ty: self.ty.into_compact(registry),
            docs: self.docs,
        }
    }
}

impl EventParamSpec {
    /// Creates a new event parameter specification builder.
    pub fn new(name: &'static str) -> EventParamSpecBuilder {
        EventParamSpecBuilder {
            spec: Self {
                name,
                // By default event parameters are not indexed.
                indexed: false,
                // We initialize every parameter type as `()`.
                ty: TypeSpec::new::<()>(),
                // We start with empty docs.
                docs: vec![],
            },
        }
    }
}

/// Used to construct an event parameter specification.
pub struct EventParamSpecBuilder {
    /// The built-up event parameter specification.
    spec: EventParamSpec,
}

impl EventParamSpecBuilder {
    /// Sets the type of the event parameter.
    pub fn of_type(self, spec: TypeSpec) -> Self {
        let mut this = self;
        this.spec.ty = spec;
        this
    }

    /// If the event parameter is indexed.
    pub fn indexed(self, is_indexed: bool) -> Self {
        let mut this = self;
        this.spec.indexed = is_indexed;
        this
    }

    /// Sets the documentation of the event parameter.
    pub fn docs<D>(self, docs: D) -> Self
    where
        D: IntoIterator<Item = &'static str>,
    {
        debug_assert!(self.spec.docs.is_empty());
        Self {
            spec: EventParamSpec {
                docs: docs.into_iter().collect::<Vec<_>>(),
                ..self.spec
            },
        }
    }

    /// Finishes constructing the event parameter spec.
    pub fn done(self) -> EventParamSpec {
        self.spec
    }
}

/// Describes the return type of a contract message.
#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(transparent)]
#[serde(bound = "F::TypeId: Serialize")]
pub struct ReturnTypeSpec<F: Form = MetaForm> {
    #[serde(rename = "type")]
    opt_type: Option<TypeSpec<F>>,
}

impl IntoCompact for ReturnTypeSpec {
    type Output = ReturnTypeSpec<CompactForm>;

    fn into_compact(self, registry: &mut Registry) -> Self::Output {
        ReturnTypeSpec {
            opt_type: self
                .opt_type
                .map(|opt_type| opt_type.into_compact(registry)),
        }
    }
}

impl ReturnTypeSpec {
    /// Creates a new return type specification from the given type or `None`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use ink_metadata::{TypeSpec, ReturnTypeSpec};
    /// ReturnTypeSpec::new(None); // no return type;
    /// ReturnTypeSpec::new(TypeSpec::new::<i32>()); // return type of `i32`
    /// ```
    pub fn new<T>(ty: T) -> Self
    where
        T: Into<Option<TypeSpec>>,
    {
        Self {
            opt_type: ty.into(),
        }
    }
}

/// Describes a pair of parameter name and type.
#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(bound = "F::TypeId: Serialize")]
pub struct MessageParamSpec<F: Form = MetaForm> {
    /// The name of the parameter.
    name: &'static str,
    /// The type of the parameter.
    #[serde(rename = "type")]
    ty: TypeSpec<F>,
}

impl IntoCompact for MessageParamSpec {
    type Output = MessageParamSpec<CompactForm>;

    fn into_compact(self, registry: &mut Registry) -> Self::Output {
        MessageParamSpec {
            name: self.name,
            ty: self.ty.into_compact(registry),
        }
    }
}

impl MessageParamSpec {
    /// Constructs a new message parameter specification via builder.
    pub fn new(name: &'static str) -> MessageParamSpecBuilder {
        MessageParamSpecBuilder {
            spec: Self {
                name,
                // Uses `()` type by default.
                ty: TypeSpec::new::<()>(),
            },
        }
    }
}

/// Used to construct a message parameter specification.
pub struct MessageParamSpecBuilder {
    /// The to-be-constructed message parameter specification.
    spec: MessageParamSpec,
}

impl MessageParamSpecBuilder {
    /// Sets the type of the message parameter.
    pub fn of_type(self, ty: TypeSpec) -> Self {
        let mut this = self;
        this.spec.ty = ty;
        this
    }

    /// Finishes construction of the message parameter.
    pub fn done(self) -> MessageParamSpec {
        self.spec
    }
}
