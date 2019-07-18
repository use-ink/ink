// Copyright 2018-2019 Parity Technologies (UK) Ltd.
// This file is part of ink!.
//
// ink! is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// ink! is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with ink!.  If not, see <http://www.gnu.org/licenses/>.

use core::{
    array::TryFromSliceError,
    convert::TryFrom,
};

use crate::env::EnvTypes;
use parity_codec::{
    Decode,
    Encode,
};

/// The SRML fundamental types.
#[allow(unused)]
#[cfg_attr(feature = "test-env", derive(Debug, Clone, PartialEq, Eq))]
pub enum DefaultSrmlTypes {}

impl EnvTypes for DefaultSrmlTypes {
    type AccountId = AccountId;
    type AccountIndex = AccountIndex;
    type Balance = Balance;
    type Hash = Hash;
    type Moment = Moment;
    type BlockNumber = BlockNumber;
    type Call = Call;
}

/// The default SRML address index type.
pub type AccountIndex = u32;

/// The default SRML address type.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Encode, Decode)]
pub struct AccountId([u8; 32]);

impl From<[u8; 32]> for AccountId {
    fn from(address: [u8; 32]) -> AccountId {
        AccountId(address)
    }
}

impl<'a> TryFrom<&'a [u8]> for AccountId {
    type Error = TryFromSliceError;

    fn try_from(bytes: &'a [u8]) -> Result<AccountId, TryFromSliceError> {
        let address = <[u8; 32]>::try_from(bytes)?;
        Ok(AccountId(address))
    }
}

/// The default SRML balance type.
pub type Balance = u64;

/// The default SRML hash type.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Encode, Decode)]
pub struct Hash([u8; 32]);

impl From<[u8; 32]> for Hash {
    fn from(hash: [u8; 32]) -> Hash {
        Hash(hash)
    }
}

impl<'a> TryFrom<&'a [u8]> for Hash {
    type Error = TryFromSliceError;

    fn try_from(bytes: &'a [u8]) -> Result<Hash, TryFromSliceError> {
        let hash = <[u8; 32]>::try_from(bytes)?;
        Ok(Hash(hash))
    }
}

/// The default SRML moment type.
pub type Moment = u64;

/// The default SRML blocknumber type.
pub type BlockNumber = u64;

/// The default SRML call type.
#[derive(Encode, Decode)]
#[cfg_attr(feature = "test-env", derive(Debug, Clone, PartialEq, Eq))]
pub enum Call {
    #[codec(index = "3")]
    Balances(super::calls::Balances<DefaultSrmlTypes>),
}

impl From<super::calls::Balances<DefaultSrmlTypes>> for Call {
    fn from(balances_call: super::calls::Balances<DefaultSrmlTypes>) -> Call {
        Call::Balances(balances_call)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::env::calls;

    use node_runtime::{self, Runtime};
    use parity_codec::{Decode, Encode};

    type AccountId = u64;
    type AccountIndex = u32;
    type Balance = u64;
    type Hash = u64;

    #[test]
    fn call_balance_transfer() {
        let account = 0;
        let balance = 10_000;
        let transfer = calls::Balances::<DefaultSrmlTypes>::transfer(account, balance);
        let contract_call = super::Call::Balances(transfer);
        let srml_call = node_runtime::BalancesCall::<Runtime>::transfer(account, balance);
        let contract_call_encoded = contract_call.encode();
        let srml_call_encoded = srml_call.encode();
        assert_eq!(srml_call_encoded, contract_call_encoded);

        let srml_call_decoded: Call = Decode::decode(&mut contract_call_encoded.as_slice())
            .expect("Balances transfer call decodes to srml type");
        let srml_call_encoded = srml_call_decoded.encode();
        let contract_call_decoded: super::Call = Decode::decode(&mut srml_call_encoded.as_slice())
            .expect("Balances transfer call decodes back to contract type");
        assert_eq!(contract_call, contract_call_decoded);
    }
}


