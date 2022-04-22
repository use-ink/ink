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

pub use ink_env as env;
pub use ink_lang as lang;
#[cfg(feature = "std")]
pub use ink_metadata as metadata;
pub use ink_prelude as prelude;
pub use ink_primitives as primitives;
pub use ink_storage as storage;

pub use scale;
#[cfg(feature = "std")]
pub use scale_info;

// The top level macros:
//  - `#[ink::contract]`
//  - `#[ink::trait_definition]`
pub use self::lang::{
    contract,
    trait_definition,
};
