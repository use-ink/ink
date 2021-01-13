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

use super::super::OffHash;
use crate::{
    hash::{
        Blake2x256,
        CryptoHash,
        HashOutput,
    },
    topics::{
        Topics,
        TopicsBuilderBackend,
    },
    Clear,
    Environment,
};

#[derive(Default)]
pub struct TopicsBuilder {
    topics: Vec<OffHash>,
}

impl<E> TopicsBuilderBackend<E> for TopicsBuilder
where
    E: Environment,
{
    type Output = Vec<OffHash>;

    fn expect(&mut self, _expected_topics: usize) {}

    fn push_topic<T>(&mut self, topic_value: &T)
    where
        T: scale::Encode,
    {
        let encoded = topic_value.encode();
        let len_encoded = encoded.len();
        let mut result = <E as Environment>::Hash::clear();
        let len_result = result.as_ref().len();
        if len_encoded <= len_result {
            result.as_mut()[..len_encoded].copy_from_slice(&encoded[..]);
        } else {
            let mut hash_output = <Blake2x256 as HashOutput>::Type::default();
            <Blake2x256 as CryptoHash>::hash(&encoded[..], &mut hash_output);
            let copy_len = core::cmp::min(hash_output.len(), len_result);
            result.as_mut()[0..copy_len].copy_from_slice(&hash_output[0..copy_len]);
        }
        let off_hash = OffHash::new(&result);
        debug_assert!(
            !self.topics.contains(&off_hash),
            "duplicate topic hash discovered!"
        );
        self.topics.push(off_hash);
    }

    fn output(self) -> Self::Output {
        self.topics
    }
}

/// Record for an emitted event.
#[derive(Debug, Clone)]
pub struct EmittedEvent {
    /// Recorded topics of the emitted event.
    pub topics: Vec<OffHash>,
    /// Recorded encoding of the emitted event.
    pub data: Vec<u8>,
}

impl EmittedEvent {
    /// Creates a new emitted event.
    pub fn new<T, E>(emitted_event: E) -> Self
    where
        T: Environment,
        E: Topics + scale::Encode,
    {
        let topics = emitted_event.topics::<T, _>(TopicsBuilder::default().into());
        Self {
            topics,
            data: emitted_event.encode(),
        }
    }
}

/// Records all emitted events for later inspection.
pub struct EmittedEventsRecorder {
    emitted_events: Vec<EmittedEvent>,
}

impl EmittedEventsRecorder {
    /// Creates a new empty emitted event recorder.
    pub fn new() -> Self {
        Self {
            emitted_events: Vec::new(),
        }
    }

    /// Resets the emitted events to none.
    pub fn reset(&mut self) {
        self.emitted_events.clear();
    }

    /// Records a new emitted event.
    pub fn record<T, E>(&mut self, new_event: E)
    where
        T: Environment,
        E: Topics + scale::Encode,
    {
        self.emitted_events
            .push(EmittedEvent::new::<T, E>(new_event));
    }

    /// Returns an iterator over the emitted events in their emission order.
    pub fn emitted_events(&self) -> core::slice::Iter<EmittedEvent> {
        self.emitted_events.iter()
    }
}
