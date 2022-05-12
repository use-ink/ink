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

mod atomic_guard;
mod storage_key_holder;
mod storage_layout;
mod storage_type;

#[macro_export]
macro_rules! test_derive {
    ($name:path { $($i:tt)* } expands to { $($o:tt)* }) => {
        {
            #[allow(dead_code)]
            fn ensure_compiles() {
                $($i)*
                $($o)*
            }

            $crate::test_derive!($name { $($i)* } expands to { $($o)* } no_build);
        }
    };

    ($name:path { $($i:tt)* } expands to { $($o:tt)* } no_build) => {
        {
            let i = stringify!( $($i)* );
            let parsed = ::syn::parse_str::<::syn::DeriveInput>(i)
                .expect(concat!(
                    "Failed to parse input to `#[derive(",
                    stringify!($name),
                    ")]`",
                ));

            let res = $name(::synstructure::Structure::new(&parsed));

            let expected = quote::quote!( $($o)* );
            assert_eq!(expected.to_string(), res.to_string());
        }
    };
}
