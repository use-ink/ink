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

#![cfg_attr(not(feature = "std"), no_std)]
use ink_env::{
    DefaultEnvironment,
    Environment,
};

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

/// The address of an Ethereum account.
#[derive(Debug, Copy, Clone)]
pub struct EthereumAddress(pub [u8; 20]);

impl Default for EthereumAddress {
    fn default() -> Self {
        Self {
            0: [0; 20]
        }
    }
}

impl core::ops::Deref for EthereumAddress {
    type Target = [u8; 20];

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl core::ops::DerefMut for EthereumAddress {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl ECDSAPublicKey {
    pub fn to_eth_address(&self) -> EthereumAddress {
        use ink_env::hash;
        use secp256k1::PublicKey;

        // Transform compressed public key into uncompressed.
        let pub_key = PublicKey::parse_compressed(&self.0)
            .expect("Unable to parse the compressed ECDSA public key");
        let uncompressed = pub_key.serialize();

        // Hash the uncompressed public key by Keccak256 algorithm.
        let mut hash = <hash::Keccak256 as hash::HashOutput>::Type::default();
        // The first byte indicates that the public key is uncompressed.
        // Let's skip it for hashing the public key directly.
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
