// Copyright (C) Parity Technologies (UK) Ltd.
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
            #[derive(scale::Encode)]
            struct UnitStruct;

            impl ::ink::env::GetSignatureTopic for UnitStruct {
                fn signature_topic() -> Option<[u8; 32]> {
                    Some([0; 32])
                }
            }
        }
        expands to {
            const _: () = {
                impl ::ink::env::Event for UnitStruct {
                    type RemainingTopics = [::ink::env::event::state::HasRemainingTopics; 1usize];

                    fn topics<E, B>(
                        &self,
                        builder: ::ink::env::event::TopicsBuilder<::ink::env::event::state::Uninit, E, B>,
                    ) -> <B as ::ink::env::event::TopicsBuilderBackend<E>>::Output
                    where
                        E: ::ink::env::Environment,
                        B: ::ink::env::event::TopicsBuilderBackend<E>,
                    {
                        match self {
                            UnitStruct => {
                                builder
                                    .build::<Self>()
                                    .push_topic(<Self as ::ink::env::GetSignatureTopic>::signature_topic().as_ref())
                                    .finish()
                            }
                        }
                    }
                }
            };
        }
    }
}

#[test]
fn unit_struct_anonymous_has_no_topics() {
    crate::test_derive! {
        event_derive {
            #[derive(scale::Encode)]
            #[ink(anonymous)]
            struct UnitStruct;
        }
        expands to {
            const _: () = {
                impl ::ink::env::Event for UnitStruct {
                    type RemainingTopics = ::ink::env::event::state::NoRemainingTopics;

                    fn topics<E, B>(
                        &self,
                        builder: ::ink::env::event::TopicsBuilder<::ink::env::event::state::Uninit, E, B>,
                    ) -> <B as ::ink::env::event::TopicsBuilderBackend<E>>::Output
                    where
                        E: ::ink::env::Environment,
                        B: ::ink::env::event::TopicsBuilderBackend<E>,
                    {
                        match self {
                            UnitStruct => {
                                builder
                                    .build::<Self>()
                                    .finish()
                            }
                        }
                    }
                }
            };
        } no_build
    }
}

#[test]
fn struct_with_fields_no_topics() {
    crate::test_derive! {
        event_derive {
            #[derive(scale::Encode)]
            struct Event {
                field_1: u32,
                field_2: u64,
                field_3: u128,
            }

            impl ::ink::env::GetSignatureTopic for Event {
                fn signature_topic() -> Option<[u8; 32]> {
                    Some([0; 32])
                }
            }
        }
        expands to {
            const _: () = {
                impl ::ink::env::Event for Event {
                    type RemainingTopics = [::ink::env::event::state::HasRemainingTopics; 1usize];

                    fn topics<E, B>(
                        &self,
                        builder: ::ink::env::event::TopicsBuilder<::ink::env::event::state::Uninit, E, B>,
                    ) -> <B as ::ink::env::event::TopicsBuilderBackend<E>>::Output
                    where
                        E: ::ink::env::Environment,
                        B: ::ink::env::event::TopicsBuilderBackend<E>,
                    {
                        match self {
                            Event { .. } => {
                                builder
                                    .build::<Self>()
                                    .push_topic(<Self as ::ink::env::GetSignatureTopic>::signature_topic().as_ref())
                                    .finish()
                            }
                        }
                    }
                }
            };
        }
    }
}

#[test]
fn struct_with_fields_and_some_topics() {
    crate::test_derive! {
        event_derive {
            #[derive(scale::Encode)]
            struct Event {
                field_1: u32,
                #[ink(topic)]
                field_2: u64,
                #[ink(topic)]
                field_3: u128,
            }
        }
        expands to {
            const _: () = {
                impl ::ink::env::Event for Event {
                    type RemainingTopics = [::ink::env::event::state::HasRemainingTopics; 3usize];


                    fn topics<E, B>(
                        &self,
                        builder: ::ink::env::event::TopicsBuilder<::ink::env::event::state::Uninit, E, B>,
                    ) -> <B as ::ink::env::event::TopicsBuilderBackend<E>>::Output
                    where
                        E: ::ink::env::Environment,
                        B: ::ink::env::event::TopicsBuilderBackend<E>,
                    {
                        match self {
                            Event { field_2 : __binding_1 , field_3 : __binding_2 , .. } => {
                                builder
                                    .build::<Self>()
                                    .push_topic(<Self as GetSignatureTopic>::signature_topic().as_ref())
                                    .push_topic(::ink::as_option!(__binding_1))
                                    .push_topic(::ink::as_option!(__binding_2))
                                    .finish()
                            }
                        }
                    }
                }
            };
        } no_build
    }
}
