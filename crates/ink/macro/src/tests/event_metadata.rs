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
                            .args([])
                            .docs([])
                            .done()
                    }
                }
            };
        }
    }
}
