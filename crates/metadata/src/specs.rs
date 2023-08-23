// Copyright (C) Parity Technologies (UK) Ltd.
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

use crate::{
    serde_hex,
    utils::{
        deserialize_from_byte_str,
        serialize_as_byte_str,
        trim_extra_whitespace,
    },
};
#[cfg(not(feature = "std"))]
use alloc::{
    collections::BTreeMap,
    format,
    string::String,
    vec,
    vec::Vec,
};
use core::{
    fmt::Display,
    marker::PhantomData,
};
use scale_info::{
    form::{
        Form,
        MetaForm,
        PortableForm,
    },
    meta_type,
    IntoPortable,
    Registry,
    TypeInfo,
};
use schemars::JsonSchema;
use serde::{
    de::DeserializeOwned,
    Deserialize,
    Serialize,
};
#[cfg(feature = "std")]
use std::collections::BTreeMap;

/// Describes a contract.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(bound(
    serialize = "F::Type: Serialize, F::String: Serialize",
    deserialize = "F::Type: DeserializeOwned, F::String: DeserializeOwned"
))]
pub struct ContractSpec<F: Form = MetaForm>
where
    TypeSpec<F>: Default,
{
    /// The set of constructors of the contract.
    constructors: Vec<ConstructorSpec<F>>,
    /// The external messages of the contract.
    messages: Vec<MessageSpec<F>>,
    /// The events of the contract.
    events: Vec<EventSpec<F>>,
    /// The contract documentation.
    docs: Vec<F::String>,
    /// The language specific error type.
    lang_error: TypeSpec<F>,
    /// The environment types of the contract specification.
    environment: EnvironmentSpec<F>,
}

impl IntoPortable for ContractSpec {
    type Output = ContractSpec<PortableForm>;

    fn into_portable(self, registry: &mut Registry) -> Self::Output {
        ContractSpec {
            constructors: self
                .constructors
                .into_iter()
                .map(|constructor| constructor.into_portable(registry))
                .collect::<Vec<_>>(),
            messages: self
                .messages
                .into_iter()
                .map(|msg| msg.into_portable(registry))
                .collect::<Vec<_>>(),
            events: self
                .events
                .into_iter()
                .map(|event| event.into_portable(registry))
                .collect::<Vec<_>>(),
            docs: registry.map_into_portable(self.docs),
            lang_error: self.lang_error.into_portable(registry),
            environment: self.environment.into_portable(registry),
        }
    }
}

impl<F> ContractSpec<F>
where
    F: Form,
    TypeSpec<F>: Default,
{
    /// Returns the set of constructors of the contract.
    pub fn constructors(&self) -> &[ConstructorSpec<F>] {
        &self.constructors
    }

    /// Returns the external messages of the contract.
    pub fn messages(&self) -> &[MessageSpec<F>] {
        &self.messages
    }

    /// Returns the events of the contract.
    pub fn events(&self) -> &[EventSpec<F>] {
        &self.events
    }

    /// Returns the contract documentation.
    pub fn docs(&self) -> &[F::String] {
        &self.docs
    }

    /// Returns the language error type.
    pub fn lang_error(&self) -> &TypeSpec<F> {
        &self.lang_error
    }
    // Returns the environment types of the contract specification.
    pub fn environment(&self) -> &EnvironmentSpec<F> {
        &self.environment
    }
}

/// The message builder is ready to finalize construction.
pub enum Valid {}
/// The message builder is not ready to finalize construction.
pub enum Invalid {}

#[must_use]
pub struct ContractSpecBuilder<F, S = Invalid>
where
    F: Form,
    TypeSpec<F>: Default,
{
    /// The to-be-constructed contract specification.
    spec: ContractSpec<F>,
    /// Marker for compile-time checking of valid contract specifications.
    marker: PhantomData<fn() -> S>,
}

impl<F> ContractSpecBuilder<F, Invalid>
where
    F: Form,
    TypeSpec<F>: Default,
{
    /// Sets the constructors of the contract specification.
    pub fn constructors<C>(self, constructors: C) -> ContractSpecBuilder<F, Valid>
    where
        C: IntoIterator<Item = ConstructorSpec<F>>,
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

impl<F, S> ContractSpecBuilder<F, S>
where
    F: Form,
    TypeSpec<F>: Default,
{
    /// Sets the messages of the contract specification.
    pub fn messages<M>(self, messages: M) -> Self
    where
        M: IntoIterator<Item = MessageSpec<F>>,
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
        E: IntoIterator<Item = EventSpec<F>>,
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
        D: IntoIterator<Item = <F as Form>::String>,
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

    /// Sets the language error of the contract specification.
    pub fn lang_error(self, lang_error: TypeSpec<F>) -> Self {
        Self {
            spec: ContractSpec {
                lang_error,
                ..self.spec
            },
            ..self
        }
    }

    /// Sets the environment types of the contract specification.
    pub fn environment(self, environment: EnvironmentSpec<F>) -> Self {
        Self {
            spec: ContractSpec {
                environment,
                ..self.spec
            },
            ..self
        }
    }
}

impl<S> ContractSpecBuilder<MetaForm, S> {
    /// Collect metadata for all events linked into the contract.
    pub fn collect_events(self) -> Self {
        self.events(crate::collect_events())
    }
}

impl<F> ContractSpecBuilder<F, Valid>
where
    F: Form,
    F::String: Display,
    TypeSpec<F>: Default,
{
    /// Finalizes construction of the contract specification.
    pub fn done(self) -> ContractSpec<F> {
        assert!(
            !self.spec.constructors.is_empty(),
            "must have at least one constructor"
        );
        assert!(
            !self.spec.messages.is_empty(),
            "must have at least one message"
        );
        assert!(
            self.spec.constructors.iter().filter(|c| c.default).count() < 2,
            "only one default constructor is allowed"
        );
        assert!(
            self.spec.messages.iter().filter(|m| m.default).count() < 2,
            "only one default message is allowed"
        );

        let max_topics = self.spec.environment.max_event_topics;
        let events_exceeding_max_topics_limit = self
            .spec
            .events
            .iter()
            .filter_map(|e| {
                let signature_topic = if e.signature_topic.is_some() { 1 } else { 0 };
                let topics_count =
                    signature_topic + e.args.iter().filter(|a| a.indexed).count();
                if topics_count > max_topics {
                    Some(format!(
                        "`{}::{}` ({} topics)",
                        e.module_path, e.label, topics_count
                    ))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        assert!(
            events_exceeding_max_topics_limit.is_empty(),
            "maximum of {max_topics} event topics exceeded: {}",
            events_exceeding_max_topics_limit.join(", ")
        );

        let mut signature_topics: BTreeMap<Vec<u8>, Vec<String>> = BTreeMap::new();
        for e in self.spec.events.iter() {
            if let Some(signature_topic) = &e.signature_topic {
                signature_topics
                    .entry(signature_topic.bytes.clone())
                    .or_default()
                    .push(format!("`{}::{}`", e.module_path, e.label));
            }
        }
        let signature_topic_collisions = signature_topics
            .iter()
            .filter_map(|(_, topics)| {
                if topics.len() > 1 {
                    Some(format!(
                        "event signature topic collision: {}",
                        topics.join(", ")
                    ))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        assert!(
            signature_topic_collisions.is_empty(),
            "{}",
            signature_topic_collisions.join("\n")
        );

        self.spec
    }
}

impl<F> ContractSpec<F>
where
    F: Form,
    TypeSpec<F>: Default,
    EnvironmentSpec<F>: Default,
{
    /// Creates a new contract specification.
    pub fn new() -> ContractSpecBuilder<F, Invalid> {
        ContractSpecBuilder {
            spec: Self {
                constructors: Vec::new(),
                messages: Vec::new(),
                events: Vec::new(),
                docs: Vec::new(),
                lang_error: Default::default(),
                environment: Default::default(),
            },
            marker: PhantomData,
        }
    }
}

/// Describes a constructor of a contract.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(bound(
    serialize = "F::Type: Serialize, F::String: Serialize",
    deserialize = "F::Type: DeserializeOwned, F::String: DeserializeOwned",
))]
#[serde(rename_all = "camelCase")]
pub struct ConstructorSpec<F: Form = MetaForm> {
    /// The label of the constructor.
    ///
    /// In case of a trait provided constructor the label is prefixed with the trait
    /// label.
    pub label: F::String,
    /// The selector hash of the message.
    pub selector: Selector,
    /// If the constructor accepts any `value` from the caller.
    pub payable: bool,
    /// If the constructor allows reentrancy.
    pub allow_reentrancy: bool,
    /// The parameters of the deployment handler.
    pub args: Vec<MessageParamSpec<F>>,
    /// The return type of the constructor..
    pub return_type: ReturnTypeSpec<F>,
    /// The deployment handler documentation.
    pub docs: Vec<F::String>,
    /// If the constructor is the default for off-chain consumers (e.g UIs).
    default: bool,
}

impl IntoPortable for ConstructorSpec {
    type Output = ConstructorSpec<PortableForm>;

    fn into_portable(self, registry: &mut Registry) -> Self::Output {
        ConstructorSpec {
            label: self.label.to_string(),
            selector: self.selector,
            payable: self.payable,
            allow_reentrancy: self.allow_reentrancy,
            args: self
                .args
                .into_iter()
                .map(|arg| arg.into_portable(registry))
                .collect::<Vec<_>>(),
            return_type: self.return_type.into_portable(registry),
            docs: self.docs.into_iter().map(|s| s.into()).collect(),
            default: self.default,
        }
    }
}

impl<F> ConstructorSpec<F>
where
    F: Form,
{
    /// Returns the label of the constructor.
    ///
    /// In case of a trait provided constructor the label is prefixed with the trait
    /// label.
    pub fn label(&self) -> &F::String {
        &self.label
    }

    /// Returns the selector hash of the constructor.
    pub fn selector(&self) -> &Selector {
        &self.selector
    }

    /// Returns if the constructor is payable by the caller.
    pub fn payable(&self) -> &bool {
        &self.payable
    }

    /// Returns if the constructor allows reentrancy.
    pub fn allow_reentrancy(&self) -> &bool {
        &self.allow_reentrancy
    }

    /// Returns the parameters of the deployment handler.
    pub fn args(&self) -> &[MessageParamSpec<F>] {
        &self.args
    }

    /// Returns the return type of the constructor.
    pub fn return_type(&self) -> &ReturnTypeSpec<F> {
        &self.return_type
    }

    /// Returns the deployment handler documentation.
    pub fn docs(&self) -> &[F::String] {
        &self.docs
    }

    pub fn default(&self) -> &bool {
        &self.default
    }
}

/// A builder for constructors.
///
/// # Developer Note
///
/// Some fields are guarded by a type-state pattern to fail at
/// compile-time instead of at run-time. This is useful to better
/// debug code-gen macros.
#[allow(clippy::type_complexity)]
#[must_use]
pub struct ConstructorSpecBuilder<F: Form, Selector, IsPayable, AllowReentrancy, Returns>
{
    spec: ConstructorSpec<F>,
    marker: PhantomData<fn() -> (Selector, IsPayable, AllowReentrancy, Returns)>,
}

impl<F> ConstructorSpec<F>
where
    F: Form,
{
    /// Creates a new constructor spec builder.
    pub fn from_label(
        label: <F as Form>::String,
    ) -> ConstructorSpecBuilder<
        F,
        Missing<state::Selector>,
        Missing<state::IsPayable>,
        Missing<state::AllowReentrancy>,
        Missing<state::Returns>,
    > {
        ConstructorSpecBuilder {
            spec: Self {
                label,
                selector: Selector::default(),
                payable: Default::default(),
                allow_reentrancy: Default::default(),
                args: Vec::new(),
                return_type: ReturnTypeSpec::new(None),
                docs: Vec::new(),
                default: false,
            },
            marker: PhantomData,
        }
    }
}

impl<F, P, A, R> ConstructorSpecBuilder<F, Missing<state::Selector>, P, A, R>
where
    F: Form,
{
    /// Sets the function selector of the message.
    pub fn selector(
        self,
        selector: [u8; 4],
    ) -> ConstructorSpecBuilder<F, state::Selector, P, A, R> {
        ConstructorSpecBuilder {
            spec: ConstructorSpec {
                selector: selector.into(),
                ..self.spec
            },
            marker: PhantomData,
        }
    }
}

impl<F, S, A, R> ConstructorSpecBuilder<F, S, Missing<state::IsPayable>, A, R>
where
    F: Form,
{
    /// Sets if the constructor is payable, thus accepting value for the caller.
    pub fn payable(
        self,
        is_payable: bool,
    ) -> ConstructorSpecBuilder<F, S, state::IsPayable, A, R> {
        ConstructorSpecBuilder {
            spec: ConstructorSpec {
                payable: is_payable,
                ..self.spec
            },
            marker: PhantomData,
        }
    }
}

impl<F, S, P, R> ConstructorSpecBuilder<F, S, P, Missing<state::AllowReentrancy>, R>
where
    F: Form,
{
    /// Sets if the constructor is reentrant.
    pub fn allow_reentrancy(
        self,
        allow_reentrancy: bool,
    ) -> ConstructorSpecBuilder<F, S, P, state::AllowReentrancy, R> {
        ConstructorSpecBuilder {
            spec: ConstructorSpec {
                allow_reentrancy,
                ..self.spec
            },
            marker: PhantomData,
        }
    }
}

impl<F, S, P, A> ConstructorSpecBuilder<F, S, P, A, Missing<state::Returns>>
where
    F: Form,
{
    /// Sets the return type of the constructor.
    pub fn returns(
        self,
        return_type: ReturnTypeSpec<F>,
    ) -> ConstructorSpecBuilder<F, S, P, A, state::Returns> {
        ConstructorSpecBuilder {
            spec: ConstructorSpec {
                return_type,
                ..self.spec
            },
            marker: PhantomData,
        }
    }
}

impl<F, S, P, A, R> ConstructorSpecBuilder<F, S, P, A, R>
where
    F: Form,
{
    /// Sets the input arguments of the constructor specification.
    pub fn args<T>(self, args: T) -> Self
    where
        T: IntoIterator<Item = MessageParamSpec<F>>,
    {
        let mut this = self;
        debug_assert!(this.spec.args.is_empty());
        this.spec.args = args.into_iter().collect::<Vec<_>>();
        this
    }

    /// Sets the documentation of the constructor specification.
    pub fn docs<'a, D>(self, docs: D) -> Self
    where
        D: IntoIterator<Item = &'a str>,
        F::String: From<&'a str>,
    {
        let mut this = self;
        debug_assert!(this.spec.docs.is_empty());
        this.spec.docs = docs
            .into_iter()
            .map(|s| trim_extra_whitespace(s).into())
            .collect::<Vec<_>>();
        this
    }

    /// Sets the default of the constructor specification.
    pub fn default(self, default: bool) -> Self {
        ConstructorSpecBuilder {
            spec: ConstructorSpec {
                default,
                ..self.spec
            },
            marker: PhantomData,
        }
    }
}

impl<F>
    ConstructorSpecBuilder<
        F,
        state::Selector,
        state::IsPayable,
        state::AllowReentrancy,
        state::Returns,
    >
where
    F: Form,
{
    /// Finishes construction of the constructor.
    pub fn done(self) -> ConstructorSpec<F> {
        self.spec
    }
}

/// Describes a contract message.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(bound(
    serialize = "F::Type: Serialize, F::String: Serialize",
    deserialize = "F::Type: DeserializeOwned, F::String: DeserializeOwned"
))]
#[serde(rename_all = "camelCase")]
pub struct MessageSpec<F: Form = MetaForm> {
    /// The label of the message.
    ///
    /// In case of trait provided messages and constructors the prefix
    /// by convention in ink! is the label of the trait.
    label: F::String,
    /// The selector hash of the message.
    selector: Selector,
    /// If the message is allowed to mutate the contract state.
    mutates: bool,allow_reentrancy
    /// If the message accepts any `value` from the caller.
    payable: bool,
    /// If the message is allowed to re-enter the contract.
    reentrancy_allowed: bool,
    /// The parameters of the message.
    args: Vec<MessageParamSpec<F>>,
    /// The return type of the message.
    return_type: ReturnTypeSpec<F>,
    /// The message documentation.
    docs: Vec<F::String>,
    /// If the message is the default for off-chain consumers (e.g UIs).
    default: bool,
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
    /// Type state for the telling if the message is allowed to be reentrant.
    pub struct AllowReentrancy;
    /// Type state for the message return type.
    pub struct Returns;
    /// Type state for the `AccountId` type of the environment.
    pub struct AccountId;
    /// Type state for the `Balance` type of the environment.
    pub struct Balance;
    /// Type state for the `Hash` type of the environment.
    pub struct Hash;
    /// Type state for the `Timestamp` type of the environment.
    pub struct Timestamp;
    /// Type state for the `BlockNumber` type of the environment.
    pub struct BlockNumber;
    /// Type state for the `ChainExtension` type of the environment.
    pub struct ChainExtension;
    /// Type state for the max number of topics specified in the environment.
    pub struct MaxEventTopics;
    /// Type state for the size of the static buffer configured via environment variable.`
    pub struct BufferSize;
}

impl<F> MessageSpec<F>
where
    F: Form,
{
    /// Creates a new message spec builder.
    pub fn from_label(
        label: <F as Form>::String,
    ) -> MessageSpecBuilder<
        F,
        Missing<state::Selector>,
        Missing<state::Mutates>,
        Missing<state::IsPayable>,
        Missing<state::AllowReentrancy>,
        Missing<state::Returns>,
    > {
        MessageSpecBuilder {
            spec: Self {
                label,
                selector: Selector::default(),
                mutates: false,
                payable: false,
                reentrancy_allowed: false,
                args: Vec::new(),
                return_type: ReturnTypeSpec::new(None),
                docs: Vec::new(),
                default: false,
            },
            marker: PhantomData,
        }
    }
}

impl<F> MessageSpec<F>
where
    F: Form,
{
    /// Returns the label of the message.
    ///
    /// In case of trait provided messages and constructors the prefix
    /// by convention in ink! is the label of the trait.
    pub fn label(&self) -> &F::String {
        &self.label
    }

    /// Returns the selector hash of the message.
    pub fn selector(&self) -> &Selector {
        &self.selector
    }

    /// Returns true if the message is allowed to mutate the contract state.
    pub fn mutates(&self) -> bool {
        self.mutates
    }

    /// Returns true if the message is payable by the caller.
    pub fn payable(&self) -> bool {
        self.payable
    }

    /// Returns the parameters of the message.
    pub fn args(&self) -> &[MessageParamSpec<F>] {
        &self.args
    }

    /// Returns the return type of the message.
    pub fn return_type(&self) -> &ReturnTypeSpec<F> {
        &self.return_type
    }

    /// Returns the message documentation.
    pub fn docs(&self) -> &[F::String] {
        &self.docs
    }

    pub fn default(&self) -> &bool {
        &self.default
    }
}

/// A builder for messages.
///
/// # Developer Note
///
/// Some fields are guarded by a type-state pattern to fail at
/// compile-time instead of at run-time. This is useful to better
/// debug code-gen macros.
#[allow(clippy::type_complexity)]
#[must_use]
pub struct MessageSpecBuilder<F, Selector, Mutates, IsPayable, AllowReentrancy, Returns>
where
    F: Form,
{
    spec: MessageSpec<F>,
    marker: PhantomData<fn() -> (Selector, Mutates, IsPayable, AllowReentrancy, Returns)>,
}

impl<F, M, P, A, R> MessageSpecBuilder<F, Missing<state::Selector>, M, P, A, R>
where
    F: Form,
{
    /// Sets the function selector of the message.
    pub fn selector(
        self,
        selector: [u8; 4],
    ) -> MessageSpecBuilder<F, state::Selector, M, P, A, R> {
        MessageSpecBuilder {
            spec: MessageSpec {
                selector: selector.into(),
                ..self.spec
            },
            marker: PhantomData,
        }
    }
}

impl<F, S, P, A, R> MessageSpecBuilder<F, S, Missing<state::Mutates>, P, A, R>
where
    F: Form,
{
    /// Sets if the message is mutable, thus taking `&mut self` or not thus taking
    /// `&self`.
    pub fn mutates(
        self,
        mutates: bool,
    ) -> MessageSpecBuilder<F, S, state::Mutates, P, A, R> {
        MessageSpecBuilder {
            spec: MessageSpec {
                mutates,
                ..self.spec
            },
            marker: PhantomData,
        }
    }
}

impl<F, S, M, A, R> MessageSpecBuilder<F, S, M, Missing<state::IsPayable>, A, R>
where
    F: Form,
{
    /// Sets if the message is payable, thus accepting value for the caller.
    pub fn payable(
        self,
        is_payable: bool,
    ) -> MessageSpecBuilder<F, S, M, state::IsPayable, A, R> {
        MessageSpecBuilder {
            spec: MessageSpec {
                payable: is_payable,
                ..self.spec
            },
            marker: PhantomData,
        }
    }
}

impl<F, S, M, P, R> MessageSpecBuilder<F, S, M, P, Missing<state::AllowReentrancy>, R>
where
    F: Form,
{
    /// Sets if the message is reentrant.
    pub fn allow_reentrancy(
        self,
        allow_reentrancy: bool,
    ) -> MessageSpecBuilder<F, S, M, P, state::AllowReentrancy, R> {
        MessageSpecBuilder {
            spec: MessageSpec {
                reentrancy_allowed: allow_reentrancy,
                ..self.spec
            },
            marker: PhantomData,
        }
    }
}

impl<F, M, S, P, A> MessageSpecBuilder<F, S, M, P, A, Missing<state::Returns>>
where
    F: Form,
{
    /// Sets the return type of the message.
    pub fn returns(
        self,
        return_type: ReturnTypeSpec<F>,
    ) -> MessageSpecBuilder<F, S, M, P, A, state::Returns> {
        MessageSpecBuilder {
            spec: MessageSpec {
                return_type,
                ..self.spec
            },
            marker: PhantomData,
        }
    }
}

impl<F, S, M, P, A, R> MessageSpecBuilder<F, S, M, P, A, R>
where
    F: Form,
{
    /// Sets the input arguments of the message specification.
    pub fn args<T>(self, args: T) -> Self
    where
        T: IntoIterator<Item = MessageParamSpec<F>>,
    {
        let mut this = self;
        debug_assert!(this.spec.args.is_empty());
        this.spec.args = args.into_iter().collect::<Vec<_>>();
        this
    }

    /// Sets the documentation of the message specification.
    pub fn docs<D>(self, docs: D) -> Self
    where
        D: IntoIterator<Item = <F as Form>::String>,
    {
        let mut this = self;
        debug_assert!(this.spec.docs.is_empty());
        this.spec.docs = docs.into_iter().collect::<Vec<_>>();
        this
    }

    /// Sets the default of the message specification.
    pub fn default(self, default: bool) -> Self {
        MessageSpecBuilder {
            spec: MessageSpec {
                default,
                ..self.spec
            },
            marker: PhantomData,
        }
    }
}

impl<F>
    MessageSpecBuilder<
        F,
        state::Selector,
        state::Mutates,
        state::IsPayable,
        state::AllowReentrancy,
        state::Returns,
    >
where
    F: Form,
{
    /// Finishes construction of the message.
    pub fn done(self) -> MessageSpec<F> {
        self.spec
    }
}

impl IntoPortable for MessageSpec {
    type Output = MessageSpec<PortableForm>;

    fn into_portable(self, registry: &mut Registry) -> Self::Output {
        MessageSpec {
            label: self.label.to_string(),
            selector: self.selector,
            mutates: self.mutates,
            payable: self.payable,
            reentrancy_allowed: self.reentrancy_allowed,
            default: self.default,
            args: self
                .args
                .into_iter()
                .map(|arg| arg.into_portable(registry))
                .collect::<Vec<_>>(),
            return_type: self.return_type.into_portable(registry),
            docs: self.docs.into_iter().map(|s| s.into()).collect(),
        }
    }
}

/// Describes an event definition.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(bound(
    serialize = "F::Type: Serialize, F::String: Serialize",
    deserialize = "F::Type: DeserializeOwned, F::String: DeserializeOwned"
))]
pub struct EventSpec<F: Form = MetaForm> {
    /// The label of the event.
    label: F::String,
    /// The module path to the event type definition.
    module_path: F::String,
    /// The signature topic of the event. `None` if the event is anonymous.
    signature_topic: Option<SignatureTopic>,
    /// The event arguments.
    args: Vec<EventParamSpec<F>>,
    /// The event documentation.
    docs: Vec<F::String>,
}

/// The value of the signature topic for a non anonymous event.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(transparent)]
pub struct SignatureTopic {
    #[serde(
        serialize_with = "serialize_as_byte_str",
        deserialize_with = "deserialize_from_byte_str"
    )]
    bytes: Vec<u8>,
}

impl<T> From<T> for SignatureTopic
where
    T: AsRef<[u8]>,
{
    fn from(bytes: T) -> Self {
        SignatureTopic {
            bytes: bytes.as_ref().to_vec(),
        }
    }
}

impl SignatureTopic {
    /// Returns the bytes of the signature topic.
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }
}

/// An event specification builder.
#[must_use]
pub struct EventSpecBuilder<F>
where
    F: Form,
{
    spec: EventSpec<F>,
}

impl<F> EventSpecBuilder<F>
where
    F: Form,
{
    /// Sets the module path to the event type definition.
    pub fn module_path<'a>(self, path: &'a str) -> Self
    where
        F::String: From<&'a str>,
    {
        let mut this = self;
        this.spec.module_path = path.into();
        this
    }

    /// Sets the input arguments of the event specification.
    pub fn args<A>(self, args: A) -> Self
    where
        A: IntoIterator<Item = EventParamSpec<F>>,
    {
        let mut this = self;
        debug_assert!(this.spec.args.is_empty());
        this.spec.args = args.into_iter().collect::<Vec<_>>();
        this
    }

    /// Sets the signature topic of the event specification.
    pub fn signature_topic<T>(self, topic: Option<T>) -> Self
    where
        T: AsRef<[u8]>,
    {
        let mut this = self;
        debug_assert!(this.spec.signature_topic.is_none());
        this.spec.signature_topic = topic.as_ref().map(SignatureTopic::from);
        this
    }

    /// Sets the input arguments of the event specification.
    pub fn docs<'a, D>(self, docs: D) -> Self
    where
        D: IntoIterator<Item = &'a str>,
        F::String: From<&'a str>,
    {
        let mut this = self;
        debug_assert!(this.spec.docs.is_empty());
        this.spec.docs = docs
            .into_iter()
            .map(|s| trim_extra_whitespace(s).into())
            .collect::<Vec<_>>();
        this
    }

    /// Finalizes building the event specification.
    pub fn done(self) -> EventSpec<F> {
        self.spec
    }
}

impl IntoPortable for EventSpec {
    type Output = EventSpec<PortableForm>;

    fn into_portable(self, registry: &mut Registry) -> Self::Output {
        EventSpec {
            label: self.label.to_string(),
            module_path: self.module_path.to_string(),
            signature_topic: self.signature_topic,
            args: self
                .args
                .into_iter()
                .map(|arg| arg.into_portable(registry))
                .collect::<Vec<_>>(),
            docs: self.docs.into_iter().map(|s| s.into()).collect(),
        }
    }
}

impl<F> EventSpec<F>
where
    F: Form,
    F::String: Default,
{
    /// Creates a new event specification builder.
    pub fn new(label: <F as Form>::String) -> EventSpecBuilder<F> {
        EventSpecBuilder {
            spec: Self {
                label,
                module_path: Default::default(),
                signature_topic: None,
                args: Vec::new(),
                docs: Vec::new(),
            },
        }
    }
}

impl<F> EventSpec<F>
where
    F: Form,
{
    /// Returns the label of the event.
    pub fn label(&self) -> &F::String {
        &self.label
    }

    /// The event arguments.
    pub fn args(&self) -> &[EventParamSpec<F>] {
        &self.args
    }

    /// The signature topic of the event. `None` if the event is anonymous.
    pub fn signature_topic(&self) -> Option<&SignatureTopic> {
        self.signature_topic.as_ref()
    }

    /// The event documentation.
    pub fn docs(&self) -> &[F::String] {
        &self.docs
    }
}

/// The 4 byte selector to identify constructors and messages
#[derive(Debug, Default, PartialEq, Eq, derive_more::From, JsonSchema)]
pub struct Selector(#[schemars(with = "String")] [u8; 4]);

impl serde::Serialize for Selector {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serde_hex::serialize(&self.0, serializer)
    }
}

impl<'de> serde::Deserialize<'de> for Selector {
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let mut arr = [0; 4];
        serde_hex::deserialize_check_len(d, serde_hex::ExpectedLen::Exact(&mut arr[..]))?;
        Ok(arr.into())
    }
}

impl Selector {
    /// Create a new custom selector.
    pub fn new<T>(bytes: T) -> Self
    where
        T: Into<[u8; 4]>,
    {
        Self(bytes.into())
    }

    /// Returns the underlying selector bytes.
    pub fn to_bytes(&self) -> &[u8] {
        &self.0
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
/// In ink! we currently experience this problem with environmental types
/// such as the `Balance` type that is just a type alias to `u128` in the
/// default setup. Even though it would be useful for third party tools
/// such as the Polkadot UI to know that we are handling with `Balance`
/// types, we currently cannot communicate this without display names.
pub type DisplayName<F> = scale_info::Path<F>;

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
/// `pred`s display name is `Predicate` and the display name of
/// the return type is simply `bool`. Note that `Predicate` could
/// simply be a type alias to `fn(i32, i32) -> Ordering`.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(bound(
    serialize = "F::Type: Serialize, F::String: Serialize",
    deserialize = "F::Type: DeserializeOwned, F::String: DeserializeOwned"
))]
#[serde(rename_all = "camelCase")]
pub struct TypeSpec<F: Form = MetaForm> {
    /// The actual type.
    #[serde(rename = "type")]
    ty: F::Type,
    /// The compile-time known displayed representation of the type.
    display_name: DisplayName<F>,
}

impl Default for TypeSpec<MetaForm> {
    fn default() -> Self {
        TypeSpec::of_type::<()>()
    }
}

impl Default for TypeSpec<PortableForm> {
    fn default() -> Self {
        Self {
            ty: u32::default().into(),
            display_name: Default::default(),
        }
    }
}

impl IntoPortable for TypeSpec {
    type Output = TypeSpec<PortableForm>;

    fn into_portable(self, registry: &mut Registry) -> Self::Output {
        TypeSpec {
            ty: registry.register_type(&self.ty),
            display_name: self.display_name.into_portable(registry),
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
            ty: meta_type::<T>(),
            display_name: DisplayName::from_segments(segments)
                .unwrap_or_else(|err| panic!("display name is invalid: {err:?}")),
        }
    }

    /// Creates a new type specification without a display name.
    ///
    /// Example:
    /// ```no_run
    /// # use ink_metadata::{TypeSpec, ReturnTypeSpec};
    /// ReturnTypeSpec::new(TypeSpec::of_type::<i32>()); // return type of `i32`
    /// ```
    pub fn of_type<T>() -> Self
    where
        T: TypeInfo + 'static,
    {
        Self {
            ty: meta_type::<T>(),
            display_name: DisplayName::default(),
        }
    }
}

impl<F> TypeSpec<F>
where
    F: Form,
{
    /// Returns the actual type.
    pub fn ty(&self) -> &F::Type {
        &self.ty
    }

    /// Returns the compile-time known displayed representation of the type.
    pub fn display_name(&self) -> &DisplayName<F> {
        &self.display_name
    }

    /// Creates a new type specification for a given type and display name.
    pub fn new(ty: <F as Form>::Type, display_name: DisplayName<F>) -> Self {
        Self { ty, display_name }
    }
}

/// Describes a pair of parameter label and type.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(bound(
    serialize = "F::Type: Serialize, F::String: Serialize",
    deserialize = "F::Type: DeserializeOwned, F::String: DeserializeOwned"
))]
pub struct EventParamSpec<F: Form = MetaForm> {
    /// The label of the parameter.
    label: F::String,
    /// If the event parameter is indexed as a topic.
    indexed: bool,
    /// The type of the parameter.
    #[serde(rename = "type")]
    ty: TypeSpec<F>,
    /// The documentation associated with the arguments.
    docs: Vec<F::String>,
}

impl IntoPortable for EventParamSpec {
    type Output = EventParamSpec<PortableForm>;

    fn into_portable(self, registry: &mut Registry) -> Self::Output {
        EventParamSpec {
            label: self.label.to_string(),
            indexed: self.indexed,
            ty: self.ty.into_portable(registry),
            docs: self.docs.into_iter().map(|s| s.into()).collect(),
        }
    }
}

impl<F> EventParamSpec<F>
where
    F: Form,
    TypeSpec<F>: Default,
{
    /// Creates a new event parameter specification builder.
    pub fn new(label: F::String) -> EventParamSpecBuilder<F> {
        EventParamSpecBuilder {
            spec: Self {
                label,
                // By default event parameters are not indexed as topics.
                indexed: false,
                // We initialize every parameter type as `()`.
                ty: Default::default(),
                // We start with empty docs.
                docs: vec![],
            },
        }
    }
    /// Returns the label of the parameter.
    pub fn label(&self) -> &F::String {
        &self.label
    }

    /// Returns true if the event parameter is indexed as a topic.
    pub fn indexed(&self) -> bool {
        self.indexed
    }

    /// Returns the type of the parameter.
    pub fn ty(&self) -> &TypeSpec<F> {
        &self.ty
    }

    /// Returns the documentation associated with the arguments.
    pub fn docs(&self) -> &[F::String] {
        &self.docs
    }
}

/// Used to construct an event parameter specification.
#[must_use]
pub struct EventParamSpecBuilder<F>
where
    F: Form,
{
    /// The built-up event parameter specification.
    spec: EventParamSpec<F>,
}

impl<F> EventParamSpecBuilder<F>
where
    F: Form,
{
    /// Sets the type of the event parameter.
    pub fn of_type(self, spec: TypeSpec<F>) -> Self {
        let mut this = self;
        this.spec.ty = spec;
        this
    }

    /// If the event parameter is indexed as a topic.
    pub fn indexed(self, is_indexed: bool) -> Self {
        let mut this = self;
        this.spec.indexed = is_indexed;
        this
    }

    /// Sets the documentation of the event parameter.
    pub fn docs<'a, D>(self, docs: D) -> Self
    where
        D: IntoIterator<Item = &'a str>,
        F::String: From<&'a str>,
    {
        debug_assert!(self.spec.docs.is_empty());
        Self {
            spec: EventParamSpec {
                docs: docs
                    .into_iter()
                    .map(|s| trim_extra_whitespace(s).into())
                    .collect::<Vec<_>>(),
                ..self.spec
            },
        }
    }

    /// Finishes constructing the event parameter spec.
    pub fn done(self) -> EventParamSpec<F> {
        self.spec
    }
}

/// Describes the contract message return type.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(transparent)]
#[serde(bound(
    serialize = "F::Type: Serialize, F::String: Serialize",
    deserialize = "F::Type: DeserializeOwned, F::String: DeserializeOwned"
))]
#[must_use]
pub struct ReturnTypeSpec<F: Form = MetaForm> {
    #[serde(rename = "type")]
    opt_type: Option<TypeSpec<F>>,
}

impl IntoPortable for ReturnTypeSpec {
    type Output = ReturnTypeSpec<PortableForm>;

    fn into_portable(self, registry: &mut Registry) -> Self::Output {
        ReturnTypeSpec {
            opt_type: self
                .opt_type
                .map(|opt_type| opt_type.into_portable(registry)),
        }
    }
}

impl<F> ReturnTypeSpec<F>
where
    F: Form,
{
    /// Creates a new return type specification from the given type or `None`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use ink_metadata::{TypeSpec, ReturnTypeSpec};
    /// <ReturnTypeSpec<scale_info::form::MetaForm>>::new(None); // no return type;
    /// ```
    pub fn new<T>(ty: T) -> Self
    where
        T: Into<Option<TypeSpec<F>>>,
    {
        Self {
            opt_type: ty.into(),
        }
    }

    /// Returns the optional return type
    pub fn opt_type(&self) -> Option<&TypeSpec<F>> {
        self.opt_type.as_ref()
    }
}

/// Describes a pair of parameter label and type.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(bound(
    serialize = "F::Type: Serialize, F::String: Serialize",
    deserialize = "F::Type: DeserializeOwned, F::String: DeserializeOwned"
))]
pub struct MessageParamSpec<F: Form = MetaForm> {
    /// The label of the parameter.
    label: F::String,
    /// The type of the parameter.
    #[serde(rename = "type")]
    ty: TypeSpec<F>,
}

impl IntoPortable for MessageParamSpec {
    type Output = MessageParamSpec<PortableForm>;

    fn into_portable(self, registry: &mut Registry) -> Self::Output {
        MessageParamSpec {
            label: self.label.to_string(),
            ty: self.ty.into_portable(registry),
        }
    }
}

impl<F> MessageParamSpec<F>
where
    F: Form,
    TypeSpec<F>: Default,
{
    /// Constructs a new message parameter specification via builder.
    pub fn new(label: F::String) -> MessageParamSpecBuilder<F> {
        MessageParamSpecBuilder {
            spec: Self {
                label,
                // Uses `()` type by default.
                ty: TypeSpec::default(),
            },
        }
    }

    /// Returns the label of the parameter.
    pub fn label(&self) -> &F::String {
        &self.label
    }

    /// Returns the type of the parameter.
    pub fn ty(&self) -> &TypeSpec<F> {
        &self.ty
    }
}

/// Used to construct a message parameter specification.
#[must_use]
pub struct MessageParamSpecBuilder<F: Form> {
    /// The to-be-constructed message parameter specification.
    spec: MessageParamSpec<F>,
}

impl<F> MessageParamSpecBuilder<F>
where
    F: Form,
{
    /// Sets the type of the message parameter.
    pub fn of_type(self, ty: TypeSpec<F>) -> Self {
        let mut this = self;
        this.spec.ty = ty;
        this
    }

    /// Finishes construction of the message parameter.
    pub fn done(self) -> MessageParamSpec<F> {
        self.spec
    }
}

/// Describes a contract environment.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(bound(
    serialize = "F::Type: Serialize, F::String: Serialize",
    deserialize = "F::Type: DeserializeOwned, F::String: DeserializeOwned"
))]
#[serde(rename_all = "camelCase")]
pub struct EnvironmentSpec<F: Form = MetaForm>
where
    TypeSpec<F>: Default,
{
    account_id: TypeSpec<F>,
    balance: TypeSpec<F>,
    hash: TypeSpec<F>,
    timestamp: TypeSpec<F>,
    block_number: TypeSpec<F>,
    chain_extension: TypeSpec<F>,
    max_event_topics: usize,
    static_buffer_size: usize,
}

impl<F> Default for EnvironmentSpec<F>
where
    F: Form,
    TypeSpec<F>: Default,
{
    fn default() -> Self {
        Self {
            account_id: Default::default(),
            balance: Default::default(),
            hash: Default::default(),
            timestamp: Default::default(),
            block_number: Default::default(),
            chain_extension: Default::default(),
            max_event_topics: Default::default(),
            static_buffer_size: Default::default(),
        }
    }
}

impl IntoPortable for EnvironmentSpec {
    type Output = EnvironmentSpec<PortableForm>;

    fn into_portable(self, registry: &mut Registry) -> Self::Output {
        EnvironmentSpec {
            account_id: self.account_id.into_portable(registry),
            balance: self.balance.into_portable(registry),
            hash: self.hash.into_portable(registry),
            timestamp: self.timestamp.into_portable(registry),
            block_number: self.block_number.into_portable(registry),
            chain_extension: self.chain_extension.into_portable(registry),
            max_event_topics: self.max_event_topics,
            static_buffer_size: self.static_buffer_size,
        }
    }
}

impl<F> EnvironmentSpec<F>
where
    F: Form,
    TypeSpec<F>: Default,
{
    /// Returns the `AccountId` type of the environment.
    pub fn account_id(&self) -> &TypeSpec<F> {
        &self.account_id
    }
    /// Returns the `Balance` type of the environment.
    pub fn balance(&self) -> &TypeSpec<F> {
        &self.balance
    }
    /// Returns the `Hash` type of the environment.
    pub fn hash(&self) -> &TypeSpec<F> {
        &self.hash
    }
    /// Returns the `Timestamp` type of the environment.
    pub fn timestamp(&self) -> &TypeSpec<F> {
        &self.timestamp
    }
    /// Returns the `BlockNumber` type of the environment.
    pub fn block_number(&self) -> &TypeSpec<F> {
        &self.block_number
    }
    /// Returns the `ChainExtension` type of the environment.
    pub fn chain_extension(&self) -> &TypeSpec<F> {
        &self.chain_extension
    }
    /// Returns the `MAX_EVENT_TOPICS` value of the environment.
    pub fn max_event_topics(&self) -> usize {
        self.max_event_topics
    }
}

#[allow(clippy::type_complexity)]
impl<F> EnvironmentSpec<F>
where
    F: Form,
    TypeSpec<F>: Default,
    EnvironmentSpec<F>: Default,
{
    pub fn new() -> EnvironmentSpecBuilder<
        F,
        Missing<state::AccountId>,
        Missing<state::Balance>,
        Missing<state::Hash>,
        Missing<state::Timestamp>,
        Missing<state::BlockNumber>,
        Missing<state::ChainExtension>,
        Missing<state::MaxEventTopics>,
        Missing<state::BufferSize>,
    > {
        EnvironmentSpecBuilder {
            spec: Default::default(),
            marker: PhantomData,
        }
    }
}

/// An environment specification builder.
#[allow(clippy::type_complexity)]
#[must_use]
pub struct EnvironmentSpecBuilder<F, A, B, H, T, BN, C, M, BS>
where
    F: Form,
    TypeSpec<F>: Default,
    EnvironmentSpec<F>: Default,
{
    spec: EnvironmentSpec<F>,
    marker: PhantomData<fn() -> (A, B, H, T, BN, C, M, BS)>,
}

impl<F, B, H, T, BN, C, M, BS>
    EnvironmentSpecBuilder<F, Missing<state::AccountId>, B, H, T, BN, C, M, BS>
where
    F: Form,
    TypeSpec<F>: Default,
    EnvironmentSpec<F>: Default,
{
    /// Sets the `AccountId` type of the environment.
    pub fn account_id(
        self,
        account_id: TypeSpec<F>,
    ) -> EnvironmentSpecBuilder<F, state::AccountId, B, H, T, BN, C, M, BS> {
        EnvironmentSpecBuilder {
            spec: EnvironmentSpec {
                account_id,
                ..self.spec
            },
            marker: PhantomData,
        }
    }
}

impl<F, A, H, T, BN, C, M, BS>
    EnvironmentSpecBuilder<F, A, Missing<state::Balance>, H, T, BN, C, M, BS>
where
    F: Form,
    TypeSpec<F>: Default,
    EnvironmentSpec<F>: Default,
{
    /// Sets the `Balance` type of the environment.
    pub fn balance(
        self,
        balance: TypeSpec<F>,
    ) -> EnvironmentSpecBuilder<F, A, state::Balance, H, T, BN, C, M, BS> {
        EnvironmentSpecBuilder {
            spec: EnvironmentSpec {
                balance,
                ..self.spec
            },
            marker: PhantomData,
        }
    }
}

impl<F, A, B, T, BN, C, M, BS>
    EnvironmentSpecBuilder<F, A, B, Missing<state::Hash>, T, BN, C, M, BS>
where
    F: Form,
    TypeSpec<F>: Default,
    EnvironmentSpec<F>: Default,
{
    /// Sets the `Hash` type of the environment.
    pub fn hash(
        self,
        hash: TypeSpec<F>,
    ) -> EnvironmentSpecBuilder<F, A, B, state::Hash, T, BN, C, M, BS> {
        EnvironmentSpecBuilder {
            spec: EnvironmentSpec { hash, ..self.spec },
            marker: PhantomData,
        }
    }
}

impl<F, A, B, H, BN, C, M, BS>
    EnvironmentSpecBuilder<F, A, B, H, Missing<state::Timestamp>, BN, C, M, BS>
where
    F: Form,
    TypeSpec<F>: Default,
    EnvironmentSpec<F>: Default,
{
    /// Sets the `Timestamp` type of the environment.
    pub fn timestamp(
        self,
        timestamp: TypeSpec<F>,
    ) -> EnvironmentSpecBuilder<F, A, B, H, state::Timestamp, BN, C, M, BS> {
        EnvironmentSpecBuilder {
            spec: EnvironmentSpec {
                timestamp,
                ..self.spec
            },
            marker: PhantomData,
        }
    }
}

impl<F, A, B, H, T, C, M, BS>
    EnvironmentSpecBuilder<F, A, B, H, T, Missing<state::BlockNumber>, C, M, BS>
where
    F: Form,
    TypeSpec<F>: Default,
    EnvironmentSpec<F>: Default,
{
    /// Sets the `BlockNumber` type of the environment.
    pub fn block_number(
        self,
        block_number: TypeSpec<F>,
    ) -> EnvironmentSpecBuilder<F, A, B, H, T, state::BlockNumber, C, M, BS> {
        EnvironmentSpecBuilder {
            spec: EnvironmentSpec {
                block_number,
                ..self.spec
            },
            marker: PhantomData,
        }
    }
}

impl<F, A, B, H, T, BN, M, BS>
    EnvironmentSpecBuilder<F, A, B, H, T, BN, Missing<state::ChainExtension>, M, BS>
where
    F: Form,
    TypeSpec<F>: Default,
    EnvironmentSpec<F>: Default,
{
    /// Sets the `ChainExtension` type of the environment.
    pub fn chain_extension(
        self,
        chain_extension: TypeSpec<F>,
    ) -> EnvironmentSpecBuilder<F, A, B, H, T, BN, state::ChainExtension, M, BS> {
        EnvironmentSpecBuilder {
            spec: EnvironmentSpec {
                chain_extension,
                ..self.spec
            },
            marker: PhantomData,
        }
    }
}

impl<F, A, B, H, T, BN, C, BS>
    EnvironmentSpecBuilder<F, A, B, H, T, BN, C, Missing<state::MaxEventTopics>, BS>
where
    F: Form,
    TypeSpec<F>: Default,
    EnvironmentSpec<F>: Default,
{
    /// Sets the `MAX_EVENT_TOPICS` value of the environment.
    pub fn max_event_topics(
        self,
        max_event_topics: usize,
    ) -> EnvironmentSpecBuilder<F, A, B, H, T, BN, C, state::MaxEventTopics, BS> {
        EnvironmentSpecBuilder {
            spec: EnvironmentSpec {
                max_event_topics,
                ..self.spec
            },
            marker: PhantomData,
        }
    }
}

impl<F, A, B, H, T, BN, C, M>
    EnvironmentSpecBuilder<F, A, B, H, T, BN, C, M, Missing<state::BufferSize>>
where
    F: Form,
    TypeSpec<F>: Default,
    EnvironmentSpec<F>: Default,
{
    /// Sets the size of the static buffer configured via environment variable.`
    pub fn static_buffer_size(
        self,
        static_buffer_size: usize,
    ) -> EnvironmentSpecBuilder<F, A, B, H, T, BN, C, M, state::BufferSize> {
        EnvironmentSpecBuilder {
            spec: EnvironmentSpec {
                static_buffer_size,
                ..self.spec
            },
            marker: PhantomData,
        }
    }
}

impl<F>
    EnvironmentSpecBuilder<
        F,
        state::AccountId,
        state::Balance,
        state::Hash,
        state::Timestamp,
        state::BlockNumber,
        state::ChainExtension,
        state::MaxEventTopics,
        state::BufferSize,
    >
where
    F: Form,
    TypeSpec<F>: Default,
    EnvironmentSpec<F>: Default,
{
    /// Finished constructing the `EnvironmentSpec` object.
    pub fn done(self) -> EnvironmentSpec<F> {
        self.spec
    }
}
