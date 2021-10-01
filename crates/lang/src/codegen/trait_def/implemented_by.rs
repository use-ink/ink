// Copyright 2018-2021 Parity Technologies (UK) Ltd.
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

/// Marker trait required by info objects of ink! trait definitions.
///
/// - Automatically implemented for the trait info object of an ink! trait
///   definition by the `#[ink::trait_definition]` procedural macro.
///
/// # Note
///
/// The `GLOBAL_TRAIT_ID` is a `u32` value uniquely identifying the
/// ink! trait definition. Every implementer of an ink! trait definition
/// is required to implement this trait given the correct `GLOBAL_TRAIT_ID`.
///
/// # Safety
///
/// The trait is marked as `unsafe` to signal to ink! smart contract authors
/// that manually implementing an ink! trait definition is unsafe to do and
/// may potentially lead to unwanted behavior.
pub unsafe trait TraitImplementedById<const GLOBAL_TRAIT_ID: u32> {}
