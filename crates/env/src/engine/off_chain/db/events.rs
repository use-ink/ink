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

use super::super::OffHash;
use crate::env::{
    EnvTypes,
    Topics,
};

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
        T: EnvTypes,
        E: Topics<T> + scale::Encode,
    {
        Self {
            topics: emitted_event
                .topics()
                .iter()
                .map(|hash| OffHash::new(hash))
                .collect::<Vec<_>>(),
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
        T: EnvTypes,
        E: Topics<T> + scale::Encode,
    {
        self.emitted_events.push(EmittedEvent::new(new_event));
    }

    /// Returns an iterator over the emitted events in their emission order.
    pub fn emitted_events(&self) -> core::slice::Iter<EmittedEvent> {
        self.emitted_events.iter()
    }
}
