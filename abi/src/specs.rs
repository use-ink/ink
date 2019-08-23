// Copyright 2018-2019 Parity Technologies (UK) Ltd.
// This file is part of ink!.
//
// ink! is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// ink! is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with ink!.  If not, see <http://www.gnu.org/licenses/>.

use core::marker::PhantomData;
use serde::Serialize;
use type_metadata::{
    form::{
        CompactForm,
        Form,
        MetaForm,
    },
    IntoCompact,
    Metadata,
    Registry,
};

#[cfg(not(feature = "std"))]
use alloc::{
    vec,
    vec::Vec,
};

/// Describes a contract.
#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(bound = "F::TypeId: Serialize")]
pub struct ContractSpec<F: Form = MetaForm> {
    /// The name of the contract.
    name: F::String,
    /// The deploy handler of the contract.
    deploy: DeploySpec<F>,
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
            name: registry.register_string(&self.name),
            deploy: self.deploy.into_compact(registry),
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

pub struct ContractSpecBuilder<S = Invalid> {
    /// The name of the to-be-constructed contract specification.
    name: <MetaForm as Form>::String,
    /// The deploy handler of the to-be-constructed contract specification.
    deploy: Option<DeploySpec>,
    /// The messages of the to-be-constructed contract specification.
    messages: Vec<MessageSpec>,
    /// The events of the to-be-constructed contract specification.
    events: Vec<EventSpec>,
    /// The documentation of the to-be-constructed contract specification.
    docs: Vec<<MetaForm as Form>::String>,
    /// Marker for compile-time checking of valid contract specifications.
    marker: PhantomData<fn() -> S>,
}

impl ContractSpecBuilder<Invalid> {
    /// Sets the deploy handler of the contract specification.
    pub fn on_deploy(self, deploy_handler: DeploySpec) -> ContractSpecBuilder<Valid> {
        ContractSpecBuilder {
            name: self.name,
            deploy: Some(deploy_handler),
            messages: self.messages,
            events: self.events,
            docs: self.docs,
            marker: PhantomData,
        }
    }
}

impl<S> ContractSpecBuilder<S> {
    /// Sets the messages of the contract specification.
    pub fn messages<M>(self, messages: M) -> Self
    where
        M: IntoIterator<Item = MessageSpec>,
    {
        debug_assert!(self.messages.is_empty());
        Self {
            messages: messages.into_iter().collect::<Vec<_>>(),
            ..self
        }
    }

    /// Sets the events of the contract specification.
    pub fn events<E>(self, events: E) -> Self
    where
        E: IntoIterator<Item = EventSpec>,
    {
        debug_assert!(self.events.is_empty());
        Self {
            events: events.into_iter().collect::<Vec<_>>(),
            ..self
        }
    }

    /// Sets the documentation of the contract specification.
    pub fn docs<D>(self, docs: D) -> Self
    where
        D: IntoIterator<Item = &'static str>,
    {
        debug_assert!(self.docs.is_empty());
        Self {
            docs: docs.into_iter().collect::<Vec<_>>(),
            ..self
        }
    }
}

impl ContractSpecBuilder<Valid> {
    /// Finalizes construction of the contract specification.
    pub fn done(self) -> ContractSpec {
        ContractSpec {
            name: self.name,
            deploy: self
                .deploy
                .expect("a valid contract spec build must have a deploy handler; qed"),
            messages: self.messages,
            events: self.events,
            docs: self.docs,
        }
    }
}

impl ContractSpec {
    /// Creates a new contract specification.
    pub fn new(name: <MetaForm as Form>::String) -> ContractSpecBuilder {
        ContractSpecBuilder {
            name,
            deploy: None,
            messages: vec![],
            events: vec![],
            docs: vec![],
            marker: PhantomData,
        }
    }
}

/// Describes the deploy handler of a contract.
#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(bound = "F::TypeId: Serialize")]
pub struct DeploySpec<F: Form = MetaForm> {
    /// The parameters of the deploy handler.
    args: Vec<MessageParamSpec<F>>,
    /// The deploy handler documentation.
    docs: Vec<&'static str>,
}

impl IntoCompact for DeploySpec {
    type Output = DeploySpec<CompactForm>;

    fn into_compact(self, registry: &mut Registry) -> Self::Output {
        DeploySpec {
            args: self
                .args
                .into_iter()
                .map(|arg| arg.into_compact(registry))
                .collect::<Vec<_>>(),
            docs: self.docs,
        }
    }
}

impl DeploySpec {
    /// Creates a new deploy specification builder.
    pub fn new() -> DeploySpecBuilder {
        DeploySpecBuilder {
            spec: Self {
                args: vec![],
                docs: vec![],
            },
        }
    }
}

/// A builder to construct a deploy specification.
pub struct DeploySpecBuilder {
    spec: DeploySpec,
}

impl DeploySpecBuilder {
    /// Sets the input arguments of the deploy spec.
    pub fn args<A>(self, args: A) -> DeploySpecBuilder
    where
        A: IntoIterator<Item = MessageParamSpec>,
    {
        let mut this = self;
        debug_assert!(this.spec.args.is_empty());
        this.spec.args = args.into_iter().collect::<Vec<_>>();
        this
    }

    /// Sets the documentation of the deploy spec.
    pub fn docs<D>(self, docs: D) -> DeploySpecBuilder
    where
        D: IntoIterator<Item = &'static str>,
    {
        let mut this = self;
        debug_assert!(this.spec.docs.is_empty());
        this.spec.docs = docs.into_iter().collect::<Vec<_>>();
        this
    }

    /// Finishes building the deploy spec.
    pub fn done(self) -> DeploySpec {
        self.spec
    }
}

/// Describes a contract message.
#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(bound = "F::TypeId: Serialize")]
pub struct MessageSpec<F: Form = MetaForm> {
    /// The name of the message.
    name: F::String,
    /// The selector hash of the message.
    selector: u32,
    /// If the message is allowed to mutate the contract state.
    mutates: bool,
    /// The parameters of the message.
    args: Vec<MessageParamSpec<F>>,
    /// The return type of the message.
    return_type: ReturnTypeSpec<F>,
    /// The message documentation.
    docs: Vec<&'static str>,
}

mod state {
    //! Type states that tell what state of a message has not
    //! yet been set properly for a valid construction.

    /// Type state for the message selector of a message.
    pub struct Selector;
    /// Type state for the mutability of a message.
    pub struct Mutates;
    /// Type state for the return type of a message.
    pub struct Returns;
}

/// Type state for the message builder to tell that some mandatory state has not yet been set
/// yet or to fail upon setting the same state multiple times.
pub struct Missing<S>(PhantomData<fn() -> S>);

impl MessageSpec {
    /// Creates a new message spec builder.
    pub fn new(
        name: <MetaForm as Form>::String,
    ) -> MessageSpecBuilder<
        Missing<state::Selector>,
        Missing<state::Mutates>,
        Missing<state::Returns>,
    > {
        MessageSpecBuilder {
            spec: Self {
                name,
                selector: 0,
                mutates: false,
                args: vec![],
                return_type: ReturnTypeSpec::none(),
                docs: vec![],
            },
            marker: PhantomData,
        }
    }
}

/// A builder for messages.
///
/// # Dev
///
/// Some of the fields are guarded by a type-state pattern to
/// fail at compile-time instead of at run-time. This is useful
/// to better debug code-gen macros.
pub struct MessageSpecBuilder<Selector, Mutates, Returns> {
    spec: MessageSpec,
    marker: PhantomData<fn() -> (Selector, Mutates, Returns)>,
}

impl<M, R> MessageSpecBuilder<Missing<state::Selector>, M, R> {
    /// Sets the function selector of the message.
    pub fn selector(self, selector: u32) -> MessageSpecBuilder<state::Selector, M, R> {
        MessageSpecBuilder {
            spec: MessageSpec {
                selector,
                ..self.spec
            },
            marker: PhantomData,
        }
    }
}

impl<S, R> MessageSpecBuilder<S, Missing<state::Mutates>, R> {
    /// Sets if the message is mutable, thus taking `&mut self` or not thus taking `&self`.
    pub fn mutates(self, mutates: bool) -> MessageSpecBuilder<S, state::Mutates, R> {
        MessageSpecBuilder {
            spec: MessageSpec {
                mutates,
                ..self.spec
            },
            marker: PhantomData,
        }
    }
}

impl<M, S> MessageSpecBuilder<S, M, Missing<state::Returns>> {
    /// Sets the return type of the message.
    pub fn returns(
        self,
        return_type: ReturnTypeSpec,
    ) -> MessageSpecBuilder<S, M, state::Returns> {
        MessageSpecBuilder {
            spec: MessageSpec {
                return_type,
                ..self.spec
            },
            marker: PhantomData,
        }
    }
}

impl<S, M, R> MessageSpecBuilder<S, M, R> {
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

impl MessageSpecBuilder<state::Selector, state::Mutates, state::Returns> {
    /// Finishes construction of the message.
    pub fn done(self) -> MessageSpec {
        self.spec
    }
}

impl IntoCompact for MessageSpec {
    type Output = MessageSpec<CompactForm>;

    fn into_compact(self, registry: &mut Registry) -> Self::Output {
        MessageSpec {
            name: registry.register_string(&self.name),
            selector: self.selector,
            mutates: self.mutates,
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
    name: F::String,
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
            name: registry.register_string(&self.name),
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
                args: vec![],
                docs: vec![],
            },
        }
    }
}

/// Describes a pair of parameter name and type.
#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(bound = "F::TypeId: Serialize")]
pub struct EventParamSpec<F: Form = MetaForm> {
    /// The name of the parameter.
    name: F::String,
    /// If the event parameter is indexed.
    indexed: bool,
    /// The type of the parameter.
    #[serde(rename = "type")]
    ty: F::TypeId,
}

impl IntoCompact for EventParamSpec {
    type Output = EventParamSpec<CompactForm>;

    fn into_compact(self, registry: &mut Registry) -> Self::Output {
        EventParamSpec {
            name: registry.register_string(self.name),
            indexed: self.indexed,
            ty: registry.register_type(&self.ty),
        }
    }
}

impl EventParamSpec {
    /// Creates a new event parameter specification.
    pub fn new<T>(name: &'static str, indexed: bool) -> Self
    where
        T: Metadata,
    {
        Self {
            name,
            indexed,
            ty: T::meta_type(),
        }
    }

    /// Creates a new event parameter specification.
    pub fn of<T>(name: &'static str, _ty: &T, indexed: bool) -> Self
    where
        T: Metadata,
    {
        Self::new::<T>(name, indexed)
    }
}

/// Describes the return type of a contract message.
#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(transparent)]
#[serde(bound = "F::TypeId: Serialize")]
pub struct ReturnTypeSpec<F: Form = MetaForm> {
    #[serde(rename = "type")]
    opt_type: Option<F::TypeId>,
}

impl IntoCompact for ReturnTypeSpec {
    type Output = ReturnTypeSpec<CompactForm>;

    fn into_compact(self, registry: &mut Registry) -> Self::Output {
        ReturnTypeSpec {
            opt_type: self.opt_type.map(|opt_ty| registry.register_type(&opt_ty)),
        }
    }
}

impl ReturnTypeSpec {
    /// Creates a new return type specification indicating no return type.
    pub fn none() -> Self {
        Self { opt_type: None }
    }

    /// Creates a new return type specification for the given type.
    pub fn new<T>() -> Self
    where
        T: Metadata,
    {
        Self {
            opt_type: Some(T::meta_type()),
        }
    }

    /// Creates a new return type specification for the given type.
    pub fn of<T>(_ty: &T) -> Self
    where
        T: Metadata,
    {
        Self::new::<T>()
    }
}

/// Describes a pair of parameter name and type.
#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(bound = "F::TypeId: Serialize")]
pub struct MessageParamSpec<F: Form = MetaForm> {
    /// The name of the parameter.
    name: F::String,
    /// The type of the parameter.
    #[serde(rename = "type")]
    ty: F::TypeId,
}

impl IntoCompact for MessageParamSpec {
    type Output = MessageParamSpec<CompactForm>;

    fn into_compact(self, registry: &mut Registry) -> Self::Output {
        MessageParamSpec {
            name: registry.register_string(self.name),
            ty: registry.register_type(&self.ty),
        }
    }
}

impl MessageParamSpec {
    /// Creates a new parameter specification for the given name and type.
    pub fn new<T>(name: &'static str) -> Self
    where
        T: Metadata,
    {
        Self {
            name,
            ty: T::meta_type(),
        }
    }

    /// Creates a new parameter specification for the given name and type.
    pub fn of<T>(name: &'static str, _ty: &T) -> Self
    where
        T: Metadata,
    {
        Self::new::<T>(name)
    }
}
