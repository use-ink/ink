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

/// The global call builder type for an ink! trait definition.
pub trait TraitCallBuilder {
    /// The call builder type.
    type Builder;

    /// Returns a shared reference to the global call builder type.
    ///
    /// This allows to call `&self` ink! trait messages.
    fn call(&self) -> &Self::Builder;

    /// Returns an exclusive reference to the global call builder type.
    ///
    /// This allows to call any ink! trait message.
    fn call_mut(&mut self) -> &mut Self::Builder;
}

/// Implemented by the global trait info provider.
///
/// It is used to query the global trait call forwarder.
/// There is one global trait call forwarder that implements
/// the call forwarding (short- and long-form) for all calls
/// to this trait in `ink-as-dependency` configuration.
pub trait TraitCallForwarder {
    /// The call forwarder type.
    type Forwarder: TraitCallBuilder;
}
