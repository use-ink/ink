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

use ink_prelude::string::String;

/// A debug buffer used to store debug messages and print them to stdout.
pub struct DebugBuffer {
    /// The buffer to store the emitted debug messages.
    past_debug_messages: Vec<String>,
}

impl DebugBuffer {
    /// Creates a new empty console.
    pub fn new() -> Self {
        Self {
            past_debug_messages: Vec::new(),
        }
    }

    /// Resets the debug buffer to uninitialized state.
    pub fn reset(&mut self) {
        self.past_debug_messages.clear();
    }

    /// Prints the message to stdout and stores it.
    pub fn debug_message(&mut self, message: &str) {
        self.past_debug_messages.push(message.to_string());
        print!("{}", message);
    }

    /// Returns an iterator over the past debug messages.
    pub fn past_debug_messages(&self) -> DebugMessages {
        DebugMessages::new(self)
    }
}

/// Iterator over the past debug messages.
pub struct DebugMessages<'a> {
    /// Iterator over the past debug messages.
    iter: core::slice::Iter<'a, String>,
}

impl<'a> DebugMessages<'a> {
    /// Creates a new iterator over the past debug messages.
    fn new(console: &'a DebugBuffer) -> Self {
        Self {
            iter: console.past_debug_messages.iter(),
        }
    }
}

impl<'a> Iterator for DebugMessages<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(AsRef::as_ref)
    }
}

impl<'a> ExactSizeIterator for DebugMessages<'a> {
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<'a> DoubleEndedIterator for DebugMessages<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back().map(AsRef::as_ref)
    }
}
