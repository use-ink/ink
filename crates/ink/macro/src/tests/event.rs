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
        }
        expands to {
            const _: () = {
                impl ::ink::env::Event<::ink::abi::Ink> for UnitStruct {
                    type RemainingTopics = [::ink::env::event::state::HasRemainingTopics; 1usize];

                    const SIGNATURE_TOPIC: ::core::option::Option<[::core::primitive::u8; 32]> =
                        ::core::option::Option::Some( ::ink::blake2x256!("UnitStruct()") );

                    fn topics<B>(
                        &self,
                        builder: ::ink::env::event::TopicsBuilder<::ink::env::event::state::Uninit, B, ::ink::abi::Ink>,
                    ) -> <B as ::ink::env::event::TopicsBuilderBackend<::ink::abi::Ink>>::Output
                    where
                        B: ::ink::env::event::TopicsBuilderBackend<::ink::abi::Ink>,
                    {
                        match self {
                            UnitStruct => {
                                builder
                                    .build::<Self>()
                                    .push_topic(<Self as ::ink::env::Event<::ink::abi::Ink>>::SIGNATURE_TOPIC.as_ref())
                                    .finish()
                            }
                        }
                    }

                    fn encode_data(&self) -> ::ink::prelude::vec::Vec<::core::primitive::u8> {
                        ::ink::abi::AbiEncodeWith::<::ink::abi::Ink>::encode_with(self)
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
                impl ::ink::env::Event<::ink::abi::Ink> for UnitStruct {
                    type RemainingTopics = ::ink::env::event::state::NoRemainingTopics;

                    const SIGNATURE_TOPIC: ::core::option::Option<[::core::primitive::u8; 32]> =
                        ::core::option::Option::None;

                    fn topics<B>(
                        &self,
                        builder: ::ink::env::event::TopicsBuilder<::ink::env::event::state::Uninit, B, ::ink::abi::Ink>,
                    ) -> <B as ::ink::env::event::TopicsBuilderBackend<::ink::abi::Ink>>::Output
                    where
                        B: ::ink::env::event::TopicsBuilderBackend<::ink::abi::Ink>,
                    {
                        match self {
                            UnitStruct => {
                                builder
                                    .build::<Self>()
                                    .finish()
                            }
                        }
                    }

                    fn encode_data(&self) -> ::ink::prelude::vec::Vec<::core::primitive::u8> {
                        ::ink::abi::AbiEncodeWith::<::ink::abi::Ink>::encode_with(self)
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
        }
        expands to {
            const _: () = {
                impl ::ink::env::Event<::ink::abi::Ink> for Event {
                    type RemainingTopics = [::ink::env::event::state::HasRemainingTopics; 1usize];

                    const SIGNATURE_TOPIC: ::core::option::Option<[::core::primitive::u8; 32]> =
                        ::core::option::Option::Some( ::ink::blake2x256!("Event(u32,u64,u128)") );

                    fn topics<B>(
                        &self,
                        builder: ::ink::env::event::TopicsBuilder<::ink::env::event::state::Uninit, B, ::ink::abi::Ink>,
                    ) -> <B as ::ink::env::event::TopicsBuilderBackend<::ink::abi::Ink>>::Output
                    where
                        B: ::ink::env::event::TopicsBuilderBackend<::ink::abi::Ink>,
                    {
                        match self {
                            Event { field_1 : __binding_0 , field_2 : __binding_1 , field_3 : __binding_2 , } => {
                                builder
                                    .build::<Self>()
                                    .push_topic(<Self as ::ink::env::Event<::ink::abi::Ink>>::SIGNATURE_TOPIC.as_ref())
                                    .finish()
                            }
                        }
                    }

                    fn encode_data(&self) -> ::ink::prelude::vec::Vec<::core::primitive::u8> {
                        ::ink::abi::AbiEncodeWith::<::ink::abi::Ink>::encode_with(self)
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
                impl ::ink::env::Event<::ink::abi::Ink> for Event {
                    type RemainingTopics = [::ink::env::event::state::HasRemainingTopics; 3usize];

                    const SIGNATURE_TOPIC: ::core::option::Option<[::core::primitive::u8; 32]> =
                        ::core::option::Option::Some( ::ink::blake2x256!("Event(u32,u64,u128)") );

                    fn topics<B>(
                        &self,
                        builder: ::ink::env::event::TopicsBuilder<::ink::env::event::state::Uninit, B, ::ink::abi::Ink>,
                    ) -> <B as ::ink::env::event::TopicsBuilderBackend<::ink::abi::Ink>>::Output
                    where
                        B: ::ink::env::event::TopicsBuilderBackend<::ink::abi::Ink>,
                    {
                        match self {
                            Event { field_1 : __binding_0 , field_2 : __binding_1 , field_3 : __binding_2 , } => {
                                builder
                                    .build::<Self>()
                                    .push_topic(<Self as ::ink::env::Event<::ink::abi::Ink>>::SIGNATURE_TOPIC.as_ref())
                                    .push_topic(::ink::as_option!(__binding_1))
                                    .push_topic(::ink::as_option!(__binding_2))
                                    .finish()
                            }
                        }
                    }

                    fn encode_data(&self) -> ::ink::prelude::vec::Vec<::core::primitive::u8> {
                        ::ink::abi::AbiEncodeWith::<::ink::abi::Ink>::encode_with(self)
                    }
                }
            };
        } no_build
    }
}

#[test]
fn custom_signature_topic() {
    crate::test_derive! {
        event_derive {
            #[derive(scale::Encode)]
            #[ink(signature_topic = "1111111111111111111111111111111111111111111111111111111111111111")]
            struct UnitStruct;
        }
        expands to {
            const _: () = {
                impl ::ink::env::Event<::ink::abi::Ink> for UnitStruct {
                    type RemainingTopics = [::ink::env::event::state::HasRemainingTopics; 1usize];

                    const SIGNATURE_TOPIC: ::core::option::Option<[::core::primitive::u8; 32]> =
                        ::core::option::Option::Some( [17u8, 17u8, 17u8, 17u8, 17u8, 17u8, 17u8, 17u8, 17u8, 17u8, 17u8, 17u8, 17u8, 17u8, 17u8, 17u8, 17u8, 17u8, 17u8, 17u8, 17u8, 17u8, 17u8, 17u8, 17u8, 17u8, 17u8, 17u8, 17u8, 17u8, 17u8, 17u8] );

                    fn topics<B>(
                        &self,
                        builder: ::ink::env::event::TopicsBuilder<::ink::env::event::state::Uninit, B, ::ink::abi::Ink>,
                    ) -> <B as ::ink::env::event::TopicsBuilderBackend<::ink::abi::Ink>>::Output
                    where
                        B: ::ink::env::event::TopicsBuilderBackend<::ink::abi::Ink>,
                    {
                        match self {
                            UnitStruct => {
                                builder
                                    .build::<Self>()
                                    .push_topic(<Self as ::ink::env::Event<::ink::abi::Ink>>::SIGNATURE_TOPIC.as_ref())
                                    .finish()
                            }
                        }
                    }

                    fn encode_data(&self) -> ::ink::prelude::vec::Vec<::core::primitive::u8> {
                        ::ink::abi::AbiEncodeWith::<::ink::abi::Ink>::encode_with(self)
                    }
                }
            };
        } no_build
    }
}

#[test]
fn name_override_works() {
    crate::test_derive! {
        event_derive {
            #[derive(scale::Encode)]
            #[ink(name = "MyUnitStruct")]
            struct UnitStruct;
        }
        expands to {
            const _: () = {
                impl ::ink::env::Event<::ink::abi::Ink> for UnitStruct {
                    type RemainingTopics = [::ink::env::event::state::HasRemainingTopics; 1usize];

                    const SIGNATURE_TOPIC: ::core::option::Option<[::core::primitive::u8; 32]> =
                        ::core::option::Option::Some( ::ink::blake2x256!("MyUnitStruct()") );

                    fn topics<B>(
                        &self,
                        builder: ::ink::env::event::TopicsBuilder<::ink::env::event::state::Uninit, B, ::ink::abi::Ink>,
                    ) -> <B as ::ink::env::event::TopicsBuilderBackend<::ink::abi::Ink>>::Output
                    where
                        B: ::ink::env::event::TopicsBuilderBackend<::ink::abi::Ink>,
                    {
                        match self {
                            UnitStruct => {
                                builder
                                    .build::<Self>()
                                    .push_topic(<Self as ::ink::env::Event<::ink::abi::Ink>>::SIGNATURE_TOPIC.as_ref())
                                    .finish()
                            }
                        }
                    }

                    fn encode_data(&self) -> ::ink::prelude::vec::Vec<::core::primitive::u8> {
                        ::ink::abi::AbiEncodeWith::<::ink::abi::Ink>::encode_with(self)
                    }
                }
            };
        } no_build
    }
}

#[test]
fn custom_signature_topic_precedence() {
    crate::test_derive! {
        event_derive {
            #[derive(scale::Encode)]
            #[ink(signature_topic = "1111111111111111111111111111111111111111111111111111111111111111")]
            #[ink(name = "MyUnitStruct")]
            struct UnitStruct;
        }
        expands to {
            const _: () = {
                impl ::ink::env::Event<::ink::abi::Ink> for UnitStruct {
                    type RemainingTopics = [::ink::env::event::state::HasRemainingTopics; 1usize];

                    const SIGNATURE_TOPIC: ::core::option::Option<[::core::primitive::u8; 32]> =
                        ::core::option::Option::Some( [17u8, 17u8, 17u8, 17u8, 17u8, 17u8, 17u8, 17u8, 17u8, 17u8, 17u8, 17u8, 17u8, 17u8, 17u8, 17u8, 17u8, 17u8, 17u8, 17u8, 17u8, 17u8, 17u8, 17u8, 17u8, 17u8, 17u8, 17u8, 17u8, 17u8, 17u8, 17u8] );

                    fn topics<B>(
                        &self,
                        builder: ::ink::env::event::TopicsBuilder<::ink::env::event::state::Uninit, B, ::ink::abi::Ink>,
                    ) -> <B as ::ink::env::event::TopicsBuilderBackend<::ink::abi::Ink>>::Output
                    where
                        B: ::ink::env::event::TopicsBuilderBackend<::ink::abi::Ink>,
                    {
                        match self {
                            UnitStruct => {
                                builder
                                    .build::<Self>()
                                    .push_topic(<Self as ::ink::env::Event<::ink::abi::Ink>>::SIGNATURE_TOPIC.as_ref())
                                    .finish()
                            }
                        }
                    }

                    fn encode_data(&self) -> ::ink::prelude::vec::Vec<::core::primitive::u8> {
                        ::ink::abi::AbiEncodeWith::<::ink::abi::Ink>::encode_with(self)
                    }
                }
            };
        } no_build
    }
}
