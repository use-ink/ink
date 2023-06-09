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

use crate::event::event_metadata_derive;

#[test]
fn unit_struct_works() {
    crate::test_derive! {
        event_metadata_derive {
            #[derive(ink::Event, scale::Encode)]
            struct UnitStruct;
        }
        expands to {
            const _: () = {
                impl ::ink::metadata::EventMetadata for UnitStruct {
                    fn event_spec() -> ::ink::metadata::EventSpec {
                        #[::ink::metadata::linkme::distributed_slice(::ink::metadata::EVENTS)]
                        #[linkme(crate = ::ink::metadata::linkme)]
                        static EVENT_METADATA: fn() -> ::ink::metadata::EventSpec =
                            <UnitStruct as ::ink::metadata::EventMetadata>::event_spec;

                        ::ink::metadata::EventSpec::new(::core::stringify!(UnitStruct))
                            .signature_topic(<Self as ::ink::env::Event>::SIGNATURE_TOPIC)
                            .args([])
                            .docs([])
                            .done()
                    }
                }
            };
        }
    }
}

#[test]
fn struct_with_fields_no_topics() {
    crate::test_derive! {
        event_metadata_derive {
            #[derive(ink::Event, scale::Encode)]
            struct Event {
                field_1: u32,
                field_2: u64,
                field_3: u128,
            }
        }
        expands to {
            const _: () = {
                impl ::ink::metadata::EventMetadata for Event {
                    fn event_spec() -> ::ink::metadata::EventSpec {
                        #[::ink::metadata::linkme::distributed_slice(::ink::metadata::EVENTS)]
                        #[linkme(crate = ::ink::metadata::linkme)]
                        static EVENT_METADATA: fn() -> ::ink::metadata::EventSpec =
                            <Event as ::ink::metadata::EventMetadata>::event_spec;

                        ::ink::metadata::EventSpec::new(::core::stringify!(Event))
                            .signature_topic(<Self as ::ink::env::Event>::SIGNATURE_TOPIC)
                            .args([
                                ::ink::metadata::EventParamSpec::new(::core::stringify!(field_1))
                                    .of_type(::ink::metadata::TypeSpec::with_name_str::<u32>(::core::stringify!(u32)))
                                    .indexed(false)
                                    .done(),
                                ::ink::metadata::EventParamSpec::new(::core::stringify!(field_2))
                                    .of_type(::ink::metadata::TypeSpec::with_name_str::<u64>(::core::stringify!(u64)))
                                    .indexed(false)
                                    .done(),
                                ::ink::metadata::EventParamSpec::new(::core::stringify!(field_3))
                                    .of_type(::ink::metadata::TypeSpec::with_name_str::<u128>(::core::stringify!(u128)))
                                    .indexed(false)
                                    .done()
                            ])
                            .docs([])
                            .done()
                    }
                }
            };
        }
    }
}

#[test]
fn struct_with_fields_and_some_topics() {
    crate::test_derive! {
        event_metadata_derive {
            #[derive(ink::Event, scale::Encode)]
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
                impl ::ink::metadata::EventMetadata for Event {
                    fn event_spec() -> ::ink::metadata::EventSpec {
                        #[::ink::metadata::linkme::distributed_slice(::ink::metadata::EVENTS)]
                        #[linkme(crate = ::ink::metadata::linkme)]
                        static EVENT_METADATA: fn() -> ::ink::metadata::EventSpec =
                            <Event as ::ink::metadata::EventMetadata>::event_spec;

                        ::ink::metadata::EventSpec::new(::core::stringify!(Event))
                            .signature_topic(<Self as ::ink::env::Event>::SIGNATURE_TOPIC)
                            .args([
                                ::ink::metadata::EventParamSpec::new(::core::stringify!(field_1))
                                    .of_type(::ink::metadata::TypeSpec::with_name_str::<u32>(::core::stringify!(u32)))
                                    .indexed(false)
                                    .done(),
                                ::ink::metadata::EventParamSpec::new(::core::stringify!(field_2))
                                    .of_type(::ink::metadata::TypeSpec::with_name_str::<u64>(::core::stringify!(u64)))
                                    .indexed(true)
                                    .done(),
                                ::ink::metadata::EventParamSpec::new(::core::stringify!(field_3))
                                    .of_type(::ink::metadata::TypeSpec::with_name_str::<u128>(::core::stringify!(u128)))
                                    .indexed(true)
                                    .done()
                            ])
                            .docs([])
                            .done()
                    }
                }
            };
        }
    }
}
