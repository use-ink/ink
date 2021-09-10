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

use ink_env::{
    call::{
        utils::{
            ReturnType,
            Set,
        },
        CallBuilder,
        ExecutionInput,
        Selector,
    },
    DefaultEnvironment,
    Environment,
};
use ink_storage::traits::SpreadLayout;

/// Trait used to indicate that an ink! trait definition has been checked
/// by the `#[ink::trait_definition]` procedural macro.
#[doc(hidden)]
pub unsafe trait CheckedInkTrait<T> {}

/// Trait used by `#[ink::trait_definition]` to ensure that the associated
/// return type for each trait message is correct.
#[doc(hidden)]
pub trait ImpliesReturn<T> {}

impl<T> ImpliesReturn<T> for T {}
impl<T, E, Callee, GasCost, TransferredValue, Args> ImpliesReturn<T>
    for CallBuilder<
        E,
        Callee,
        GasCost,
        TransferredValue,
        Set<ExecutionInput<Args>>,
        Set<ReturnType<T>>,
    >
where
    E: Environment,
{
}

impl<E, Callee, GasCost, TransferredValue, Args> ImpliesReturn<()>
    for CallBuilder<
        E,
        Callee,
        GasCost,
        TransferredValue,
        Set<ExecutionInput<Args>>,
        Set<()>,
    >
where
    E: Environment,
{
}

/// Dispatchable functions that have inputs.
#[doc(hidden)]
pub trait FnInput {
    /// The tuple-type of all inputs.
    type Input: scale::Decode + 'static;
}

/// Dispatchable functions that have an output.
#[doc(hidden)]
pub trait FnOutput {
    /// The output type.
    type Output: scale::Encode + 'static;
}

/// The selector of dispatchable functions.
#[doc(hidden)]
pub trait FnSelector {
    /// The selector.
    const SELECTOR: Selector;
}

/// The storage state that the dispatchable function acts on.
#[doc(hidden)]
pub trait FnState {
    /// The storage state.
    type State: SpreadLayout + Sized;
}

/// A dispatchable contract constructor message.
#[doc(hidden)]
pub trait Constructor: FnInput + FnSelector + FnState {
    const CALLABLE: fn(<Self as FnInput>::Input) -> <Self as FnState>::State;
}

/// A `&self` dispatchable contract message.
#[doc(hidden)]
pub trait MessageRef: FnInput + FnOutput + FnSelector + FnState {
    const CALLABLE: fn(
        &<Self as FnState>::State,
        <Self as FnInput>::Input,
    ) -> <Self as FnOutput>::Output;
}

/// A `&mut self` dispatchable contract message.
#[doc(hidden)]
pub trait MessageMut: FnInput + FnOutput + FnSelector + FnState {
    const CALLABLE: fn(
        &mut <Self as FnState>::State,
        <Self as FnInput>::Input,
    ) -> <Self as FnOutput>::Output;
}

/// Indicates that some compile time expression is expected to be `true`.
#[doc(hidden)]
pub trait True {}

/// The ECDSA compressed public key.
#[derive(Debug, Copy, Clone)]
pub struct ECDSAPublicKey(pub [u8; 33]);

impl Default for ECDSAPublicKey {
    fn default() -> Self {
        Self { 0: [0; 33] }
    }
}

impl core::ops::Deref for ECDSAPublicKey {
    type Target = [u8; 33];

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl core::ops::DerefMut for ECDSAPublicKey {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// Address of ethereum account
pub type EthereumAddress = [u8; 20];

impl ECDSAPublicKey {
    pub fn to_eth_address(&self) -> EthereumAddress {
        use ink_env::hash;
        use secp256k1::PublicKey;

        // Transform compressed public key into uncompressed.
        let pub_key = PublicKey::parse_compressed(&self.0)
            .expect("Unable to parse the compressed ecdsa public key");
        let uncompressed = pub_key.serialize();

        // Hash the uncompressed public key without first byte by Keccak256 algorithm.
        let mut hash = <hash::Keccak256 as hash::HashOutput>::Type::default();
        ink_env::hash_bytes::<hash::Keccak256>(&uncompressed[1..], &mut hash);

        // Take the last 20 bytes as an Address
        let mut result = EthereumAddress::default();
        result.copy_from_slice(&hash[12..]);

        result
    }

    pub fn to_account_id(&self) -> <DefaultEnvironment as Environment>::AccountId {
        use ink_env::hash;

        let mut output = <hash::Blake2x256 as hash::HashOutput>::Type::default();
        ink_env::hash_bytes::<hash::Blake2x256>(&self.0[..], &mut output);

        output.into()
    }
}
