// Copyright 2018-2019 Parity Technologies (UK) Ltd.
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

//! Code generation for documentation generation of Wasm smart contracts.
//!
//! We use the special `#[cfg(rustdoc)]` that is set when `rustdoc` is
//! compiling a crate in order to generate special code that is only used
//! for documentation purposes.

use proc_macro2::TokenStream as TokenStream2;

use crate::hir;

pub fn generate_code(_tokens: &mut TokenStream2, _contract: &hir::Contract) {}
