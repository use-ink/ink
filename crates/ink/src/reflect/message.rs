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

use ink_env::call::Selector;

/// Description of the ink! message.
pub struct MessageDescription {
    /// Yields `true` if the ink! message mutates the ink! storage.
    mutates: bool,
    /// Yields `true` if the ink! message is payable.
    payable: bool,
    /// The selector of the ink! message.
    selector: Selector,
}

impl MessageDescription {
    /// Creates the description of the message.
    pub const fn new(mutates: bool, payable: bool, selector: [u8; 4]) -> Self {
        Self {
            mutates,
            payable,
            selector: Selector::new(selector),
        }
    }

    /// Returns `true` of the ink! message mutates the ink! storage.
    pub const fn mutates(&self) -> bool {
        self.mutates
    }

    /// Returns `true` of the ink! message is payable.
    pub const fn payable(&self) -> bool {
        self.payable
    }

    /// Returns the selector of the ink! message.
    pub const fn selector(&self) -> Selector {
        self.selector
    }
}
