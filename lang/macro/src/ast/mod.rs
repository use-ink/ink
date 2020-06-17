// Copyright 2018-2020 Parity Technologies (UK) Ltd.
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

//! Provides types and infrastructure for syntactical portions of ink! that
//! require custom parsing that is not Rust conformant.
//!
//! In general we try not to require any sort of custom non-standard Rust
//! syntax.
//!
//! At the time of this writing we currently only use this for the argument
//! parsing of ink! config header `#[ink(version = "0.1.0", etc...)]` in order
//! to be able to parse identifiers in `name = value` segments for the `value`
//! part.

mod config;

use self::config::{
    AttributeArgs,
    MetaNameValue,
    PathOrLit,
};
