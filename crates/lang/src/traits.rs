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
/// This communicates the `u32` number that uniquely identifies
/// the ink! trait definition.
pub trait TraitUniqueId {
    /// The unique trait `u32` identifier.
    const ID: u32;
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

/// Implemented by call builders of smart contracts.
///
/// These might be implementing multiple different ink! traits.
/// The codegen makes them implement this trait once for every
/// ink! trait they have to implement.
///
/// While the trait is not necessary it encapsulates a lot of
/// utility and auxiliary code required for the actual ink! trait
/// implementations.
pub trait TraitCallForwarderFor<const ID: u32> {
    type Forwarder: TraitCallBuilder;

    /// Forwards the `&self` call.
    ///
    /// # Note
    ///
    /// This is used for the short-hand calling syntax.
    fn forward(&self) -> &Self::Forwarder;

    /// Forwards the `&mut self` call.
    ///
    /// # Note
    ///
    /// This is used for the short-hand calling syntax.
    fn forward_mut(&mut self) -> &mut Self::Forwarder;

    /// Builds up the `&self` call.
    ///
    /// # Note
    ///
    /// This is used for the long-hand calling syntax.
    fn build(&self) -> &<Self::Forwarder as TraitCallBuilder>::Builder;

    /// Builds up the `&mut self` call.
    ///
    /// # Note
    ///
    /// This is used for the long-hand calling syntax.
    fn build_mut(&mut self) -> &mut <Self::Forwarder as TraitCallBuilder>::Builder;
}
