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
                    const MODULE_PATH: &'static str = ::core::module_path!();

                    fn event_spec() -> ::ink::metadata::EventSpec {
                        #[::ink::linkme::distributed_slice(::ink::CONTRACT_EVENTS)]
                        #[linkme(crate = ::ink::linkme)]
                        static EVENT_METADATA: fn() -> ::ink::metadata::EventSpec =
                            <UnitStruct as ::ink::metadata::EventMetadata>::event_spec;

                        ::ink::metadata::EventSpec::new("UnitStruct")
                            .module_path(::core::module_path!())
                            .signature_topic(<Self as ::ink::env::Event<::ink::abi::Ink>>::SIGNATURE_TOPIC)
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
                    const MODULE_PATH: &'static str = ::core::module_path!();

                    fn event_spec() -> ::ink::metadata::EventSpec {
                        #[::ink::linkme::distributed_slice(::ink::CONTRACT_EVENTS)]
                        #[linkme(crate = ::ink::linkme)]
                        static EVENT_METADATA: fn() -> ::ink::metadata::EventSpec =
                            <Event as ::ink::metadata::EventMetadata>::event_spec;

                        ::ink::metadata::EventSpec::new("Event")
                            .module_path(::core::module_path!())
                            .signature_topic(<Self as ::ink::env::Event<::ink::abi::Ink>>::SIGNATURE_TOPIC)
                            .args([
                                ::ink::metadata::EventParamSpec::new(::core::stringify!(field_1))
                                    .of_type(::ink::metadata::TypeSpec::with_name_segs::<u32, _>(
                                        ::core::iter::Iterator::map(
                                            ::core::iter::IntoIterator::into_iter([::core::stringify!(u32)]),
                                            ::core::convert::AsRef::as_ref
                                        )
                                    ))
                                    .indexed(false)
                                    .docs([])
                                    .done(),
                                ::ink::metadata::EventParamSpec::new(::core::stringify!(field_2))
                                    .of_type(::ink::metadata::TypeSpec::with_name_segs::<u64, _>(
                                        ::core::iter::Iterator::map(
                                            ::core::iter::IntoIterator::into_iter([::core::stringify!(u64)]),
                                            ::core::convert::AsRef::as_ref
                                        )
                                    ))
                                    .indexed(false)
                                    .docs([])
                                    .done(),
                                ::ink::metadata::EventParamSpec::new(::core::stringify!(field_3))
                                    .of_type(::ink::metadata::TypeSpec::with_name_segs::<u128, _>(
                                        ::core::iter::Iterator::map(
                                            ::core::iter::IntoIterator::into_iter([::core::stringify!(u128)]),
                                            ::core::convert::AsRef::as_ref
                                        )
                                    ))
                                    .indexed(false)
                                    .docs([])
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
                    const MODULE_PATH: &'static str = ::core::module_path!();

                    fn event_spec() -> ::ink::metadata::EventSpec {
                        #[::ink::linkme::distributed_slice(::ink::CONTRACT_EVENTS)]
                        #[linkme(crate = ::ink::linkme)]
                        static EVENT_METADATA: fn() -> ::ink::metadata::EventSpec =
                            <Event as ::ink::metadata::EventMetadata>::event_spec;

                        ::ink::metadata::EventSpec::new("Event")
                            .module_path(::core::module_path!())
                            .signature_topic(<Self as ::ink::env::Event<::ink::abi::Ink>>::SIGNATURE_TOPIC)
                            .args([
                                ::ink::metadata::EventParamSpec::new(::core::stringify!(field_1))
                                    .of_type(::ink::metadata::TypeSpec::with_name_segs::<u32, _>(
                                        ::core::iter::Iterator::map(
                                            ::core::iter::IntoIterator::into_iter([::core::stringify!(u32)]),
                                            ::core::convert::AsRef::as_ref
                                        )
                                    ))
                                    .indexed(false)
                                    .docs([])
                                    .done(),
                                ::ink::metadata::EventParamSpec::new(::core::stringify!(field_2))
                                    .of_type(::ink::metadata::TypeSpec::with_name_segs::<u64, _>(
                                        ::core::iter::Iterator::map(
                                            ::core::iter::IntoIterator::into_iter([::core::stringify!(u64)]),
                                            ::core::convert::AsRef::as_ref
                                        )
                                    ))
                                    .indexed(true)
                                    .docs([])
                                    .done(),
                                ::ink::metadata::EventParamSpec::new(::core::stringify!(field_3))
                                    .of_type(::ink::metadata::TypeSpec::with_name_segs::<u128, _>(
                                        ::core::iter::Iterator::map(
                                            ::core::iter::IntoIterator::into_iter([::core::stringify!(u128)]),
                                            ::core::convert::AsRef::as_ref
                                        )
                                    ))
                                    .indexed(true)
                                    .docs([])
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
fn name_override_works() {
    crate::test_derive! {
        event_metadata_derive {
            #[derive(ink::Event, scale::Encode)]
            #[ink(name = "MyUnitStruct")]
            struct UnitStruct;
        }
        expands to {
            const _: () = {
                impl ::ink::metadata::EventMetadata for UnitStruct {
                    const MODULE_PATH: &'static str = ::core::module_path!();

                    fn event_spec() -> ::ink::metadata::EventSpec {
                        #[::ink::linkme::distributed_slice(::ink::CONTRACT_EVENTS)]
                        #[linkme(crate = ::ink::linkme)]
                        static EVENT_METADATA: fn() -> ::ink::metadata::EventSpec =
                            <UnitStruct as ::ink::metadata::EventMetadata>::event_spec;

                        ::ink::metadata::EventSpec::new("MyUnitStruct")
                            .module_path(::core::module_path!())
                            .signature_topic(<Self as ::ink::env::Event<::ink::abi::Ink>>::SIGNATURE_TOPIC)
                            .args([])
                            .docs([])
                            .done()
                    }
                }
            };
        } no_build
    }
}
