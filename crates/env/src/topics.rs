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

//! Docs

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
    /// Initializes the topics builder and informs it about how many topics it must expect to serialize.
    ///
    /// The number of expected topics is given implicitely by the `E` type parameter.
    pub fn build<Event: Topics>(
        mut self,
    ) -> TopicsBuilder<<Event as Topics>::RemainingTopics, E, B> {
        self.backend
            .expect(<<Event as Topics>::RemainingTopics as EventTopicsAmount>::AMOUNT);
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
    /// No more event topics can be serialized afterwards but the environment will be
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

/// Implemented by event types to guide the event topic serialization using the topics builder.
///
/// Normally this trait should be implemented automatically via the ink! codegen.
pub trait Topics {
    /// Type state indicating how many event topics are to be expected by the topics builder.
    type RemainingTopics: EventTopicsAmount;

    /// Guides event topic serialization using the given topics builder.
    fn topics<E, B>(
        &self,
        builder: TopicsBuilder<state::Uninit, E, B>,
    ) -> <B as TopicsBuilderBackend<E>>::Output
    where
        E: Environment,
        B: TopicsBuilderBackend<E>;
}

/// For each topic a hash is generated. This hash must be unique
/// for a field and its value. The `prefix` is concatenated
/// with the `value` and this result is then hashed.
/// The `prefix` is typically set to the path a field has in
/// an event struct + the identifier of the event struct.
///
/// For example, in the case of our Erc20 example contract the
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
