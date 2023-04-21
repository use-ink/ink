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

//! This module contains the implementation for the event topic logic.

use crate::Environment;

/// The concrete implementation that is guided by the topics builder.
///
/// To be implemented by the on-chain and off-chain environments respectively.
#[doc(hidden)]
pub trait TopicsBuilderBackend<E>
where
    E: Environment,
{
    /// The type of the serialized event topics.
    type Output;

    /// Initialized the backend with the expected number of event topics.
    fn expect(&mut self, expected_topics: usize);

    /// Pushes another topic for serialization to the backend.
    fn push_topic<T>(&mut self, topic_value: &T)
    where
        T: scale::Encode;

    /// Extracts the serialized topics.
    fn output(self) -> Self::Output;
}

/// Builder for event topic serialization.
///
/// Abstraction to build up event topic serialization with zero-overhead,
/// no heap-memory allocations and no dynamic dispatch.
#[doc(hidden)]
pub struct TopicsBuilder<S, E, B> {
    backend: B,
    state: core::marker::PhantomData<fn() -> (S, E)>,
}

impl<E, B> From<B> for TopicsBuilder<state::Uninit, E, B>
where
    E: Environment,
    B: TopicsBuilderBackend<E>,
{
    fn from(backend: B) -> Self {
        Self {
            backend,
            state: Default::default(),
        }
    }
}

#[doc(hidden)]
pub mod state {
    /// The topic builder is uninitialized and needs to be provided with the
    /// expected number of topics that need to be constructed.
    pub enum Uninit {}
    /// There are some remaining topics that need to be provided with some values.
    pub enum HasRemainingTopics {}
    /// There are no more remaining topics and the topic builder shall be finalized.
    pub enum NoRemainingTopics {}
}

impl<E, B> TopicsBuilder<state::Uninit, E, B>
where
    E: Environment,
    B: TopicsBuilderBackend<E>,
{
    /// Initializes the topics builder and informs it about how many topics it must expect
    /// to serialize.
    ///
    /// The number of expected topics is given by the `TopicsAmount` type parameter.
    pub fn build<TopicsAmount: EventTopicsAmount>(
        mut self,
    ) -> TopicsBuilder<TopicsAmount, E, B> {
        self.backend
            .expect(<TopicsAmount as EventTopicsAmount>::AMOUNT);
        TopicsBuilder {
            backend: self.backend,
            state: Default::default(),
        }
    }
}

impl<E, S, B> TopicsBuilder<S, E, B>
where
    E: Environment,
    S: SomeRemainingTopics,
    B: TopicsBuilderBackend<E>,
{
    /// Pushes another event topic to be serialized through the topics builder.
    ///
    /// Returns a topics builder that expects one less event topic for serialization
    /// than before the call.
    pub fn push_topic<T>(
        mut self,
        value: &T,
    ) -> TopicsBuilder<<S as SomeRemainingTopics>::Next, E, B>
    where
        T: scale::Encode,
    {
        self.backend.push_topic(value);
        TopicsBuilder {
            backend: self.backend,
            state: Default::default(),
        }
    }
}

impl<E, B> TopicsBuilder<state::NoRemainingTopics, E, B>
where
    E: Environment,
    B: TopicsBuilderBackend<E>,
{
    /// Finalizes the topics builder.
    ///
    /// No more event topics can be serialized afterwards, but the environment will be
    /// able to extract the information collected by the topics builder in order to
    /// emit the serialized event.
    pub fn finish(self) -> <B as TopicsBuilderBackend<E>>::Output
    where
        B: TopicsBuilderBackend<E>,
    {
        self.backend.output()
    }
}

/// Indicates that there are some remaining topics left for expected serialization.
#[doc(hidden)]
pub trait SomeRemainingTopics {
    /// The type state indicating the amount of the remaining topics afterwards.
    ///
    /// Basically trivial sequence of: `N => N - 1` unless `N <= 1`
    type Next;
}

/// Indicates the actual amount of expected event topics.
#[doc(hidden)]
pub trait EventTopicsAmount {
    /// The actual amount of remaining topics.
    const AMOUNT: usize;
}

macro_rules! impl_some_remaining_for {
    ( $( $n:literal ),* $(,)? ) => {
        $(
            impl SomeRemainingTopics for [state::HasRemainingTopics; $n] {
                type Next = [state::HasRemainingTopics; $n - 1];
            }

            impl EventTopicsAmount for [state::HasRemainingTopics; $n] {
                const AMOUNT: usize = $n;
            }
        )*
    };
}
#[rustfmt::skip]
impl_some_remaining_for!(
             2,  3,  4,  5,  6,  7,  8,  9,
    10, 11, 12, 13, 14, 15, 16, 17, 18, 19,
    20, 21, 22, 23, 24, 25, 26, 27, 28, 29,
    30, 31, 32,
);

impl SomeRemainingTopics for [state::HasRemainingTopics; 1] {
    type Next = state::NoRemainingTopics;
}

impl EventTopicsAmount for [state::HasRemainingTopics; 1] {
    const AMOUNT: usize = 1;
}

impl EventTopicsAmount for state::NoRemainingTopics {
    const AMOUNT: usize = 0;
}

/// Implemented by event types to guide the event topic serialization using the topics
/// builder.
///
/// Normally this trait should be implemented automatically via the ink! codegen.
pub trait Topics {
    /// The environment type.
    type Env: Environment;

    /// Guides event topic serialization using the given topics builder.
    fn topics<B>(
        &self,
        builder: TopicsBuilder<state::Uninit, Self::Env, B>,
    ) -> <B as TopicsBuilderBackend<Self::Env>>::Output
    where
        B: TopicsBuilderBackend<Self::Env>;
}

/// For each topic a hash is generated. This hash must be unique
/// for a field and its value. The `prefix` is concatenated
/// with the `value`. This result is then hashed.
/// The `prefix` is typically set to the path a field has in
/// an event struct plus the identifier of the event struct.
///
/// For example, in the case of our ERC-20 example contract the
/// prefix `Erc20::Transfer::from` is concatenated with the
/// field value of `from` and then hashed.
/// In this example `Erc20` would be the contract identified,
/// `Transfer` the event identifier, and `from` the field identifier.
#[doc(hidden)]
pub struct PrefixedValue<'a, 'b, T> {
    pub prefix: &'a [u8],
    pub value: &'b T,
}

impl<X> scale::Encode for PrefixedValue<'_, '_, X>
where
    X: scale::Encode,
{
    #[inline]
    fn size_hint(&self) -> usize {
        self.prefix.size_hint() + self.value.size_hint()
    }

    #[inline]
    fn encode_to<T: scale::Output + ?Sized>(&self, dest: &mut T) {
        self.prefix.encode_to(dest);
        self.value.encode_to(dest);
    }
}

use core::marker::PhantomData;

/// Guards that an ink! event definitions respects the topic limit.
///
/// # Usage
///
/// ```
/// // #[ink(event)]
/// pub struct ExampleEvent {}
///
/// /// The amount of the topics of the example event struct.
/// const LEN_TOPICS: usize = 3;
///
/// /// The limit for the amount of topics per ink! event definition.
/// const TOPICS_LIMIT: usize = 4;
///
/// impl ::ink::codegen::EventLenTopics for ExampleEvent {
///     type LenTopics = ::ink::codegen::EventTopics<LEN_TOPICS>;
/// }
///
/// // The below code only compiles successfully if the example ink! event
/// // definitions respects the topic limitation: it must have an amount of
/// // topics less than or equal to the topic limit.
/// const _: () = ::ink::codegen::utils::consume_type::<
///     ::ink::codegen::EventRespectsTopicLimit<ExampleEvent, TOPICS_LIMIT>,
/// >();
/// ```
pub struct EventRespectsTopicLimit<Event, const LEN_MAX_TOPICS: usize>
where
    Event: EventLenTopics,
    <Event as EventLenTopics>::LenTopics: RespectTopicLimit<LEN_MAX_TOPICS>,
{
    marker: PhantomData<fn() -> Event>,
}

/// Guards that an amount of event topics respects the event topic limit.
///
/// # Note
///
/// Implemented by `EventTopics<M>` if M is less or equal to N.
/// Automatically implemented for up to 12 event topics.
pub trait RespectTopicLimit<const N: usize> {}

/// Represents an the amount of topics for an ink! event definition.
pub struct EventTopics<const N: usize>;

macro_rules! impl_is_smaller_or_equals {
    ( $first:literal $( , $rest:literal )* $(,)? ) => {
        impl RespectTopicLimit<$first> for EventTopics<$first> {}
        $(
            impl RespectTopicLimit<$rest> for EventTopics<$first> {}
        )*

        impl_is_smaller_or_equals! { $( $rest ),* }
    };
    ( ) => {};
}
impl_is_smaller_or_equals! {
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12
}

/// Stores the number of event topics of the ink! event definition.
pub trait EventLenTopics {
    /// Type denoting the number of event topics.
    ///
    /// # Note
    ///
    /// We use an associated type instead of an associated constant here
    /// because Rust does not yet allow for generics in constant parameter
    /// position which would be required in the `EventRespectsTopicLimit`
    /// trait definition.
    /// As soon as this is possible in Rust we might change this to a constant.
    type LenTopics;
}
