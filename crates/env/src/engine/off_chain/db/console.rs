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

/// A debug console used to print console contents and store them.
pub struct Console {
    /// The buffer to store the already pasted contents.
    past_prints: Vec<String>,
}

impl Console {
    /// Creates a new empty console.
    pub fn new() -> Self {
        Self {
            past_prints: Vec::new(),
        }
    }

    /// Resets the console to uninitialized state.
    pub fn reset(&mut self) {
        self.past_prints.clear();
    }

    /// Prints the contents to the actual console and stores them.
    pub fn println(&mut self, contents: &str) {
        self.past_prints.push(contents.to_string());
        println!("{}", contents);
    }

    /// Returns an iterator over the past console prints.
    pub fn past_prints(&self) -> PastPrints {
        PastPrints::new(self)
    }
}

/// Iterator over the past prints to the console.
pub struct PastPrints<'a> {
    /// Iterator over the past printlns.
    iter: core::slice::Iter<'a, String>,
}

impl<'a> PastPrints<'a> {
    /// Creates a new iterator over the past console prints.
    fn new(console: &'a Console) -> Self {
        Self {
            iter: console.past_prints.iter(),
        }
    }
}

impl<'a> Iterator for PastPrints<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(AsRef::as_ref)
    }
}

impl<'a> ExactSizeIterator for PastPrints<'a> {
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<'a> DoubleEndedIterator for PastPrints<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back().map(AsRef::as_ref)
    }
}
