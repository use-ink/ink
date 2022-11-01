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

/// todo: docs
pub trait EventInfo {
    const PATH: &'static str;
}

/// todo: docs
/// The ID is the index of the event variant in the enum
pub trait EventVariantInfo<const ID: usize> {
    const NAME: &'static str;
    /// todo: docs
    /// Will be hashed unique path of Event -> Variant, used for topic of Event variant
    /// Should be able to compute up front
    const SIGNATURE_TOPIC: [u8; 32];
}

/// todo: docs
pub trait EventDefinition<const ID: u128> {
    type Type: EventInfo;
}

/// todo: docs
pub enum EventDefinitionRegistry;


