// Copyright (C) Use Ink (UK) Ltd.
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

use ink_primitives::abi::{
    AbiEncodeWith,
    Ink,
    Sol,
};

/// The concrete implementation that is guided by the topics builder.
///
/// To be implemented by the on-chain and off-chain environments respectively.
#[doc(hidden)]
pub trait TopicsBuilderBackend<Abi = crate::DefaultAbi> {
    /// The type of the serialized event topics.
    type Output;

    /// Pushes another topic for serialization to the backend.
    fn push_topic<T>(&mut self, topic_value: &T)
    where
        T: AbiEncodeWith<Abi>;

    /// Extracts the serialized topics.
    fn output(self) -> Self::Output;
}

/// Specifies the topic (i.e. indexed event parameter) encoding implementation for
/// the given ABI.
pub trait TopicEncoder: private::Sealed + Sized {
    /// True if the topic hashing implementation requires a buffer.
    ///
    /// (e.g. when hashing requires calling a pre-compile).
    const REQUIRES_BUFFER: bool;

    /// Encodes the value as a topic (i.e. an indexed event parameter).
    fn encode_topic<T>(value: &T) -> [u8; 32]
    where
        T: AbiEncodeWith<Self>;

    /// Encodes the value as a topic (i.e. an indexed event parameter), utilizing the
    /// given buffer for hashing (if necessary).
    fn encode_topic_with_hash_buffer<T>(
        value: &T,
        output: &mut [u8; 32],
        buffer: &mut [u8],
    ) where
        T: AbiEncodeWith<Self>;
}

/// Builder for event topic serialization.
///
/// Abstraction to build up event topic serialization with zero-overhead,
/// no heap-memory allocations and no dynamic dispatch.
#[doc(hidden)]
pub struct TopicsBuilder<S, B, Abi = crate::DefaultAbi> {
    backend: B,
    #[allow(clippy::type_complexity)]
    state: core::marker::PhantomData<fn() -> (S, Abi)>,
}

impl<B, Abi> From<B> for TopicsBuilder<state::Uninit, B, Abi>
where
    B: TopicsBuilderBackend<Abi>,
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

impl<B, Abi> TopicsBuilder<state::Uninit, B, Abi>
where
    B: TopicsBuilderBackend<Abi>,
{
    /// Initializes the topics builder.
    ///
    /// The number of expected topics is given implicitly by the `E` type parameter.
    pub fn build<Evt: Event<Abi>>(
        self,
    ) -> TopicsBuilder<<Evt as Event<Abi>>::RemainingTopics, B, Abi> {
        TopicsBuilder {
            backend: self.backend,
            state: Default::default(),
        }
    }
}

impl<S, B> TopicsBuilder<S, B, Ink>
where
    S: SomeRemainingTopics,
    B: TopicsBuilderBackend<Ink>,
{
    /// Pushes another event topic to be serialized through the topics builder.
    ///
    /// Returns a topics builder that expects one less event topic for serialization
    /// than before the call.
    pub fn push_topic<T>(
        mut self,
        value: Option<&T>,
    ) -> TopicsBuilder<<S as SomeRemainingTopics>::Next, B, Ink>
    where
        T: AbiEncodeWith<Ink>,
    {
        // Only publish the topic if it is not an `Option::None`.
        if let Some(topic) = value {
            self.backend.push_topic::<T>(topic);
        } else {
            self.backend.push_topic::<u8>(&0u8);
        }
        TopicsBuilder {
            backend: self.backend,
            state: Default::default(),
        }
    }
}

impl<S, B> TopicsBuilder<S, B, Sol>
where
    S: SomeRemainingTopics,
    B: TopicsBuilderBackend<Sol>,
{
    /// Pushes another event topic to be serialized through the topics builder.
    ///
    /// Returns a topics builder that expects one less event topic for serialization
    /// than before the call.
    pub fn push_topic<T>(
        mut self,
        value: &T,
    ) -> TopicsBuilder<<S as SomeRemainingTopics>::Next, B, Sol>
    where
        T: AbiEncodeWith<Sol>,
    {
        self.backend.push_topic::<T>(value);
        TopicsBuilder {
            backend: self.backend,
            state: Default::default(),
        }
    }
}

impl<B, Abi> TopicsBuilder<state::NoRemainingTopics, B, Abi>
where
    B: TopicsBuilderBackend<Abi>,
{
    /// Finalizes the topics builder.
    ///
    /// No more event topics can be serialized afterwards, but the environment will be
    /// able to extract the information collected by the topics builder in order to
    /// emit the serialized event.
    pub fn finish(self) -> <B as TopicsBuilderBackend<Abi>>::Output
    where
        B: TopicsBuilderBackend<Abi>,
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

impl_some_remaining_for!(2, 3, 4);

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
/// Normally this trait should be implemented automatically via `#[derive(ink::Event)`.
pub trait Event<Abi = crate::DefaultAbi>: AbiEncodeWith<Abi> {
    /// Type state indicating how many event topics are to be expected by the topics
    /// builder.
    type RemainingTopics: EventTopicsAmount;

    /// The unique signature topic of the event. `None` for anonymous events.
    ///
    /// It can be automatically calculated or manually specified.
    ///
    /// Usually this is calculated using the `#[derive(ink::Event)]` derive, which by
    /// default calculates this as `blake2b("Event(field1_type,field2_type)"`
    const SIGNATURE_TOPIC: core::option::Option<[u8; 32]>;

    /// Guides event topic serialization using the given topics builder.
    fn topics<B>(
        &self,
        builder: TopicsBuilder<state::Uninit, B, Abi>,
    ) -> <B as TopicsBuilderBackend<Abi>>::Output
    where
        B: TopicsBuilderBackend<Abi>;

    /// ABI encode the dynamic data of this event.
    fn encode_data(&self) -> ink_prelude::vec::Vec<u8>;
}

mod private {
    /// Seals the implementation of `TopicEncoder`.
    pub trait Sealed {}

    impl Sealed for ink_primitives::abi::Ink {}

    impl Sealed for ink_primitives::abi::Sol {}
}
