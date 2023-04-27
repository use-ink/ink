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

// These tests are partly testing if code is expanded correctly.
// Hence the syntax contains a number of verbose statements which
// are not properly cleaned up.
#![allow(clippy::absurd_extreme_comparisons)]
#![allow(clippy::identity_op)]
#![allow(clippy::eq_op)]
#![allow(clippy::match_single_binding)]

use crate::event::event_derive;

#[test]
fn unit_struct_works() {
    crate::test_derive! {
        event_derive {
            struct UnitStruct;
        }
        expands to {
            const _: () = {
                impl ::ink::env::Topics for UnitStruct {
                    type RemainingTopics = [::ink::env::topics::state::HasRemainingTopics; 1usize];

                    fn topics<E, B>(
                        &self,
                        builder: ::ink::env::topics::TopicsBuilder<::ink::env::topics::state::Uninit, E, B>,
                    ) -> <B as ::ink::env::topics::TopicsBuilderBackend<E>>::Output
                    where
                        E: ::ink::env::Environment,
                        B: ::ink::env::topics::TopicsBuilderBackend<E>,
                    {
                        match self {
                            UnitStruct => {
                                builder
                                    .build::<Self>()
                                    .push_topic(&Self::SIGNATURE_TOPIC.expect("non-anonymous events must have a signature topic"))
                                    .finish()
                            }
                        }
                    }
                }
            };
        }
    }
}
