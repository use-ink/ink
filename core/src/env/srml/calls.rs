// Copyright 2019 Parity Technologies (UK) Ltd.
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

use crate::env::EnvTypes;
use parity_codec::{
    Decode,
    Encode,
};

#[derive(Encode, Decode)]
#[cfg_attr(feature = "test-env", derive(Debug, Clone, PartialEq, Eq))]
pub enum Balances<T: EnvTypes> {
    #[allow(non_camel_case_types)]
    transfer(T::AccountIndex, #[codec(compact)] T::Balance),
    #[allow(non_camel_case_types)]
    set_balance(
        T::AccountIndex,
        #[codec(compact)] T::Balance,
        #[codec(compact)] T::Balance,
    ),
}

#[cfg(test)]
mod tests {
    use super::*;

    use contracts::{
        ComputeDispatchFee, ContractAddressFor,
        TrieId, TrieIdGenerator,
    };
    use parity_codec::{Decode, Encode};
    use runtime_primitives::testing::{Header, H256};
    use runtime_primitives::traits::{BlakeTwo256, IdentityLookup};
    use srml_support::{
        impl_outer_dispatch, impl_outer_event, impl_outer_origin, parameter_types,
        StorageValue, traits::Get,
    };
    use std::cell::RefCell;
    use system;

    type AccountId = u64;
    type AccountIndex = AccountId;
    type Balance = u64;
    type Hash = u64;

    impl_outer_event! {
        pub enum MetaEvent for Test {
            balances<T>, contracts<T>,
        }
    }
    impl_outer_origin! {
        pub enum Origin for Test { }
    }
    impl_outer_dispatch! {
        pub enum Call for Test where origin: Origin {
            balances::Balances,
            contract::Contract,
        }
    }

    thread_local! {
	static EXISTENTIAL_DEPOSIT: RefCell<u64> = RefCell::new(0);
	static TRANSFER_FEE: RefCell<u64> = RefCell::new(0);
	static CREATION_FEE: RefCell<u64> = RefCell::new(0);
	static BLOCK_GAS_LIMIT: RefCell<u64> = RefCell::new(0);
}

    pub struct ExistentialDeposit;
    impl Get<u64> for ExistentialDeposit {
        fn get() -> u64 { EXISTENTIAL_DEPOSIT.with(|v| *v.borrow()) }
    }

    pub struct TransferFee;
    impl Get<u64> for TransferFee {
        fn get() -> u64 { TRANSFER_FEE.with(|v| *v.borrow()) }
    }

    pub struct CreationFee;
    impl Get<u64> for CreationFee {
        fn get() -> u64 { CREATION_FEE.with(|v| *v.borrow()) }
    }

    pub struct BlockGasLimit;
    impl Get<u64> for BlockGasLimit {
        fn get() -> u64 { BLOCK_GAS_LIMIT.with(|v| *v.borrow()) }
    }

    #[derive(Clone, Eq, PartialEq, Debug)]
    pub struct Test;
    parameter_types! {
	    pub const BlockHashCount: u64 = 250;
    }
    impl system::Trait for Test {
        type Origin = Origin;
        type Index = u64;
        type BlockNumber = u64;
        type Hash = H256;
        type Hashing = BlakeTwo256;
        type AccountId = u64;
        type Lookup = IdentityLookup<Self::AccountId>;
        type Header = Header;
        type Event = MetaEvent;
        type BlockHashCount = BlockHashCount;
    }
    parameter_types! {
        pub const BalancesTransactionBaseFee: u64 = 0;
        pub const BalancesTransactionByteFee: u64 = 0;
    }
    impl balances::Trait for Test {
        type Balance = Balance;
        type OnFreeBalanceZero = Contract;
        type OnNewAccount = ();
        type Event = MetaEvent;
        type TransactionPayment = ();
        type DustRemoval = ();
        type TransferPayment = ();
        type ExistentialDeposit = ExistentialDeposit;
        type TransferFee = TransferFee;
        type CreationFee = CreationFee;
        type TransactionBaseFee = BalancesTransactionBaseFee;
        type TransactionByteFee = BalancesTransactionByteFee;
    }
    parameter_types! {
        pub const MinimumPeriod: u64 = 1;
    }
    impl timestamp::Trait for Test {
        type Moment = u64;
        type OnTimestampSet = ();
        type MinimumPeriod = MinimumPeriod;
    }
    parameter_types! {
        pub const SignedClaimHandicap: u64 = 2;
        pub const TombstoneDeposit: u64 = 16;
        pub const StorageSizeOffset: u32 = 8;
        pub const RentByteFee: u64 = 4;
        pub const RentDepositOffset: u64 = 10_000;
        pub const SurchargeReward: u64 = 150;
        pub const TransactionBaseFee: u64 = 2;
        pub const TransactionByteFee: u64 = 6;
        pub const ContractFee: u64 = 21;
        pub const CallBaseFee: u64 = 135;
        pub const CreateBaseFee: u64 = 175;
        pub const MaxDepth: u32 = 100;
    }
    impl contracts::Trait for Test {
        type Currency = Balances;
        type Call = Call;
        type DetermineContractAddress = DummyContractAddressFor;
        type Event = MetaEvent;
        type ComputeDispatchFee = DummyComputeDispatchFee;
        type TrieIdGenerator = DummyTrieIdGenerator;
        type GasPayment = ();
        type SignedClaimHandicap = SignedClaimHandicap;
        type TombstoneDeposit = TombstoneDeposit;
        type StorageSizeOffset = StorageSizeOffset;
        type RentByteFee = RentByteFee;
        type RentDepositOffset = RentDepositOffset;
        type SurchargeReward = SurchargeReward;
        type TransferFee = TransferFee;
        type CreationFee = CreationFee;
        type TransactionBaseFee = TransactionBaseFee;
        type TransactionByteFee = TransactionByteFee;
        type ContractFee = ContractFee;
        type CallBaseFee = CallBaseFee;
        type CreateBaseFee = CreateBaseFee;
        type MaxDepth = MaxDepth;
        type BlockGasLimit = BlockGasLimit;
    }

    type Balances = balances::Module<Test>;
    type Contract = contracts::Module<Test>;
//    type System = system::Module<Test>;

    pub struct DummyContractAddressFor;
    impl ContractAddressFor<H256, u64> for DummyContractAddressFor {
        fn contract_address_for(_code_hash: &H256, _data: &[u8], origin: &u64) -> u64 {
            *origin + 1
        }
    }

    pub struct DummyTrieIdGenerator;
    impl TrieIdGenerator<u64> for DummyTrieIdGenerator {
        fn trie_id(account_id: &u64) -> TrieId {
            use substrate_primitives::storage::well_known_keys;

            let new_seed = contracts::AccountCounter::mutate(|v| {
                *v = v.wrapping_add(1);
                *v
            });

            // TODO: see https://github.com/paritytech/substrate/issues/2325
            let mut res = vec![];
            res.extend_from_slice(well_known_keys::CHILD_STORAGE_KEY_PREFIX);
            res.extend_from_slice(b"default:");
            res.extend_from_slice(&new_seed.to_le_bytes());
            res.extend_from_slice(&account_id.to_le_bytes());
            res
        }
    }

    pub struct DummyComputeDispatchFee;
    impl ComputeDispatchFee<Call, u64> for DummyComputeDispatchFee {
        fn compute_dispatch_fee(_call: &Call) -> u64 {
            69
        }
    }

    /// ink! env types
    #[derive(Debug, Clone, Eq, PartialEq)]
    enum TestEnvTypes {}
    impl EnvTypes for TestEnvTypes {
        type AccountId = AccountId;
        type AccountIndex = AccountIndex;
        type Balance = Balance;
        type Hash = Hash;
        type Moment = u64;
        type Call = EnvCall;
        type BlockNumber = u64;
    }

    #[derive(Encode, Decode, Debug, Clone, Eq, PartialEq)]
    enum EnvCall {
        Balances(super::Balances<TestEnvTypes>),
    }

    #[test]
    fn call_balance_transfer() {
        let account = 0;
        let balance = 10_000;
        let contract_call = super::Balances::<TestEnvTypes>::transfer(account, balance);
        let srml_call = balances::Call::<Test>::transfer(account, balance);
        let contract_call_encoded = contract_call.encode();
        let srml_call_encoded = srml_call.encode();
        assert_eq!(srml_call_encoded, contract_call_encoded);

        let srml_call_decoded: balances::Call<Test> = Decode::decode(&mut contract_call_encoded.as_slice())
            .expect("Balances transfer call decodes to srml type");
        let srml_call_encoded = srml_call_decoded.encode();
        let contract_call_decoded: super::Balances<TestEnvTypes> = Decode::decode(&mut srml_call_encoded.as_slice())
            .expect("Balances transfer call decodes back to contract type");
        assert_eq!(contract_call, contract_call_decoded);
    }
}
