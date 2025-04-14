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

#![cfg_attr(not(feature = "std"), no_std, no_main)]
#![allow(unexpected_cfgs)]

#[ink::event]
/// EventExternal docs
pub struct EventExternal {
    f1: bool,
    /// f2 docs
    #[ink(topic)]
    f2: u32,
}

#[ink::contract]
mod contract {
    #[ink(storage)]
    pub struct Contract {}

    #[ink(event)]
    /// EventInline docs
    pub struct EventInline {
        #[ink(topic)]
        f3: bool,
        /// f4 docs
        f4: u32,
    }

    impl Contract {
        #[ink(constructor)]
        pub fn new(_x: u8) -> Self {
            Self {}
        }
    }

    impl Contract {
        #[ink(message)]
        pub fn get_value(&self) -> u32 {
            42
        }
    }
}

#[cfg(test)]
mod tests {
    fn generate_metadata() -> ink_metadata::InkProject {
        unsafe extern "Rust" {
            fn __ink_generate_metadata() -> ink_metadata::InkProject;
        }

        unsafe { __ink_generate_metadata() }
    }

    #[test]
    fn collects_all_events() {
        let metadata = generate_metadata();

        assert_eq!(metadata.spec().events().len(), 2);

        let event_external = metadata
            .spec()
            .events()
            .iter()
            .find(|e| e.label() == "EventExternal")
            .expect("EventExternal should be present");

        assert_eq!(event_external.docs(), &["EventExternal docs"]);
        assert_eq!(event_external.args().len(), 2);

        let arg_f2 = event_external
            .args()
            .iter()
            .find(|a| a.label() == "f2")
            .expect("f2 should be present");
        assert_eq!(arg_f2.docs(), &["f2 docs"]);
        assert!(arg_f2.indexed());

        let event_inline = metadata
            .spec()
            .events()
            .iter()
            .find(|e| e.label() == "EventInline")
            .expect("EventInline should be present");

        assert_eq!(event_inline.docs(), &["EventInline docs"]);
        assert_eq!(event_inline.args().len(), 2);

        let arg_f4 = event_inline
            .args()
            .iter()
            .find(|a| a.label() == "f4")
            .expect("f4 should be present");
        assert_eq!(arg_f4.docs(), &["f4 docs"]);
        assert!(!arg_f4.indexed());
    }
}
