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

use ink_env::Environment;

/// Implemented by contracts that are compiled as dependencies.
///
/// Allows them to return their underlying account identifier.
pub trait ToAccountId<T>
where
    T: Environment,
{
    /// Returns the underlying account identifier of the instantiated contract.
    fn to_account_id(&self) -> <T as Environment>::AccountId;
}

/// Used to refer to the generated contract reference from the given contract storage struct.
pub trait ContractReference {
    /// The generated contract reference type.
    type Type;
}

/// Used to refer to the generated contract call builder.
///
/// The generated contract call builder implements the long-hand calling API
/// for all inherent or trait ink! messages.
pub trait ContractCallBuilder {
    /// The generated contract call builder type.
    type Type;
}

/// Trait to inform about the name of an ink! smart contract.
pub trait ContractName {
    /// The name of the ink! smart contract.
    const NAME: &'static str;
}
