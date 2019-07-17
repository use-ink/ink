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

impl ContractSpec {
    /// Creates a new contract specification.
    pub fn new(name: &'static str, deploy: DeploySpec) -> Self {
        ContractSpec {
            name,
            deploy,
            messages: vec![],
            events: vec![],
            docs: vec![],
        }
    }

    /// Pushes a message to the contract specification.
    pub fn push_message(&mut self, msg: MessageSpec) {
        self.messages.push(msg);
    }

    /// Pushes a set of messages to the contract specification.
    pub fn push_messages<M>(&mut self, msgs: M)
    where
        M: IntoIterator<Item = MessageSpec>,
    {
        self.messages.extend(msgs.into_iter());
    }

    /// Pushes an event to the contract specification.
    pub fn push_event(&mut self, event: EventSpec) {
        self.events.push(event);
    }

    /// Pushes a set of events to the contract specification.
    pub fn push_events<E>(&mut self, events: E)
    where
        E: IntoIterator<Item = EventSpec>,
    {
        self.events.extend(events.into_iter());
    }

    /// Pushes a line of documentation.
    pub fn push_doc(&mut self, line: &'static str) {
        self.docs.push(line)
    }

    /// Pushes a set of events to the contract specification.
    pub fn push_docs<L>(&mut self, lines: L)
    where
        L: IntoIterator<Item = &'static str>,
    {
        self.docs.extend(lines.into_iter());
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
    /// Creates a new deploy specification.
    pub fn new<A, D>(args: A, docs: D) -> Self
    where
        A: IntoIterator<Item = MessageParamSpec>,
        D: IntoIterator<Item = &'static str>,
    {
        Self {
            args: args.into_iter().collect::<Vec<_>>(),
            docs: docs.into_iter().collect::<Vec<_>>(),
        }
    }
}

/// Describes a contract message.
#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(bound = "F::TypeId: Serialize")]
pub struct MessageSpec<F: Form = MetaForm> {
    /// The name of the message.
    name: F::String,
    /// The selector hash of the message.
    selector: u64,
    /// If the message is allowed to mutate the contract state.
    mutates: bool,
    /// The parameters of the message.
    args: Vec<MessageParamSpec<F>>,
    /// The return type of the message.
    return_type: ReturnTypeSpec<F>,
    /// The message documentation.
    docs: Vec<&'static str>,
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
    #[serde(rename = "documentation")]
    docs: Vec<&'static str>,
}

impl EventSpec {
    /// Creates a new event specification.
    pub fn new<A, D>(name: &'static str, args: A, docs: D) -> Self
    where
        A: IntoIterator<Item = EventParamSpec>,
        D: IntoIterator<Item = &'static str>,
    {
        Self {
            name,
            args: args.into_iter().collect::<Vec<_>>(),
            docs: docs.into_iter().collect::<Vec<_>>(),
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
}

/// Describes the return type of a contract message.
#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(transparent)]
#[serde(bound = "F::TypeId: Serialize")]
pub struct ReturnTypeSpec<F: Form = MetaForm> {
    #[serde(rename = "type")]
    opt_type: Option<F::TypeId>,
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
}
