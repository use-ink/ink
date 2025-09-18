// Copyright (C) Use Ink (UK) Ltd.
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

use core::marker::PhantomData;

use ink_env::{
    Environment,
    Result,
    call::{
        Call,
        CallParams,
        ConstructorReturnType,
        CreateParams,
        DelegateCall,
        FromAddr,
        LimitParamsV2,
        utils::DecodeMessageResult,
    },
    hash::{
        CryptoHash,
        HashOutput,
    },
};
use ink_primitives::{
    Address,
    H256,
    U256,
    abi::{
        AbiEncodeWith,
        Ink,
        Sol,
    },
};
use pallet_revive_uapi::ReturnErrorCode;

/// The API behind the `self.env()` and `Self::env()` syntax in ink!.
///
/// This allows ink! messages to make use of the environment efficiently
/// and user-friendly while also maintaining access invariants.
#[derive(Copy, Clone)]
pub struct EnvAccess<'a, E> {
    /// Tricks the Rust compiler into thinking that we use `E`.
    marker: PhantomData<fn() -> &'a E>,
}

impl<E> Default for EnvAccess<'_, E> {
    #[inline]
    fn default() -> Self {
        Self {
            marker: Default::default(),
        }
    }
}

impl<E> core::fmt::Debug for EnvAccess<'_, E> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.debug_struct("EnvAccess").finish()
    }
}

impl<E> EnvAccess<'_, E>
where
    E: Environment,
{
    /// Returns the address of the caller of the executed contract.
    ///
    /// # Example
    ///
    /// ```
    /// # #[ink::contract]
    /// # pub mod my_contract {
    /// #     #[ink(storage)]
    /// #     pub struct MyContract { }
    /// #
    /// #     impl MyContract {
    /// #         #[ink(constructor)]
    /// #         pub fn new() -> Self {
    /// #             Self {}
    /// #         }
    /// #
    /// #[ink(message)]
    /// pub fn call_me(&self) {
    ///     let caller = self.env().caller();
    /// }
    /// #
    /// #     }
    /// # }
    /// ```
    ///
    /// # Note
    ///
    /// For more details visit: [`ink_env::caller`]
    pub fn caller(self) -> Address {
        ink_env::caller()
    }

    /// Returns the transferred value for the contract execution.
    ///
    /// # Example
    ///
    /// ```
    /// # #[ink::contract]
    /// # pub mod my_contract {
    /// #     #[ink(storage)]
    /// #     pub struct MyContract { }
    /// #
    /// #     impl MyContract {
    /// #         #[ink(constructor)]
    /// #         pub fn new() -> Self {
    /// #             Self {}
    /// #         }
    /// #
    /// /// Allows funding the contract. Prints a debug message with the transferred value.
    /// #[ink(message, payable)]
    /// pub fn fund(&mut self) {
    ///     let caller = self.env().caller();
    ///     let value = self.env().transferred_value();
    /// }
    /// #
    /// #     }
    /// # }
    /// ```
    ///
    /// # Note
    ///
    /// For more details visit: [`ink_env::transferred_value`]
    pub fn transferred_value(self) -> U256 {
        ink_env::transferred_value()
    }

    /// Returns the price for the specified amount of gas.
    ///
    /// # Example
    ///
    /// ```
    /// # #[ink::contract]
    /// # pub mod my_contract {
    /// #     #[ink(storage)]
    /// #     pub struct MyContract { }
    /// #
    /// #     impl MyContract {
    /// #         #[ink(constructor)]
    /// #         pub fn new() -> Self {
    /// #             Self {}
    /// #         }
    /// #
    /// #[ink(message)]
    /// pub fn foo(&self) {}
    ///
    /// // todo
    /// // /// Returns a tuple of
    /// // ///   - the result of adding the `rhs` to the `lhs`
    /// // ///   - the gas costs of this addition operation
    /// // ///   - the price for the gas
    /// // #[ink(message)]
    /// // pub fn addition_gas_cost(&self, rhs: i32, lhs: i32) -> (i32, u64, Balance) {
    /// //    let before = self.env().gas_left();
    /// //    let result = rhs + lhs;
    /// //    let after = self.env().gas_left();
    /// //    let gas_used = after - before;
    /// //    let gas_cost = self.env().weight_to_fee(gas_used);
    /// //    (result, gas_used, gas_cost)
    /// //}
    /// #
    /// #     }
    /// # }
    /// ```
    ///
    /// # Note
    ///
    /// For more details visit: [`ink_env::weight_to_fee`]
    /// todo: there is now also `gas_price`
    pub fn weight_to_fee(self, gas: u64) -> E::Balance {
        ink_env::weight_to_fee::<E>(gas)
    }

    /// Returns the timestamp of the current block.
    ///
    /// # Example
    ///
    /// ```
    /// #[ink::contract]
    /// pub mod my_contract {
    ///     #[ink(storage)]
    ///     pub struct MyContract {
    ///         last_invocation: Timestamp,
    ///     }
    ///
    ///     impl MyContract {
    ///         #[ink(constructor)]
    ///         pub fn new() -> Self {
    ///             Self {
    ///                 last_invocation: Self::env().block_timestamp(),
    ///             }
    ///         }
    ///
    ///         /// Records the last time the message was invoked.
    ///         #[ink(message)]
    ///         pub fn execute_me(&mut self) {
    ///             self.last_invocation = self.env().block_timestamp();
    ///         }
    ///     }
    /// }
    /// ```
    ///
    /// # Note
    ///
    /// The Substrate default for the timestamp type is the milliseconds since the
    /// Unix epoch. However, this is not guaranteed: the specific timestamp is
    /// defined by the chain environment on which this contract runs.
    ///
    /// For more details visit: [`ink_env::block_timestamp`]
    pub fn block_timestamp(self) -> E::Timestamp {
        ink_env::block_timestamp::<E>()
    }

    /// Retrieves the account id for a specified address.
    ///
    /// # Example
    ///
    /// ```
    /// #[ink::contract]
    /// pub mod only_owner {
    ///     #[ink(storage)]
    ///     pub struct OnlyOwner {
    ///         owner: AccountId,
    ///         value: u32,
    ///     }
    ///
    ///     impl OnlyOwner {
    ///         #[ink(constructor)]
    ///         pub fn new(owner: AccountId) -> Self {
    ///             Self { owner, value: 0 }
    ///         }
    ///
    ///         /// Allows incrementing the contract's `value` only
    ///         /// for the owner.
    ///         ///
    ///         /// The contract panics if the caller is not the owner.
    ///         #[ink(message)]
    ///         pub fn increment(&mut self) {
    ///             let caller = self.env().address();
    ///             let caller_acc = self.env().to_account_id(caller);
    ///             assert!(self.owner == caller_acc);
    ///             self.value = self.value + 1;
    ///         }
    ///     }
    /// }
    /// ```
    ///
    /// # Note
    ///
    /// For more details visit: [`ink_env::to_account_id`]
    #[cfg(feature = "unstable-hostfn")]
    pub fn to_account_id(self, addr: Address) -> E::AccountId {
        ink_env::to_account_id::<E>(addr)
    }

    /// Returns the account ID of the executed contract.
    ///
    /// # Example
    ///
    /// ```
    /// #[ink::contract]
    /// pub mod only_owner {
    ///     #[ink(storage)]
    ///     pub struct OnlyOwner {
    ///         owner: AccountId,
    ///         value: u32,
    ///     }
    ///
    ///     impl OnlyOwner {
    ///         #[ink(constructor)]
    ///         pub fn new() -> Self {
    ///             Self {
    ///                 owner: Self::env().account_id(),
    ///                 value: 0,
    ///             }
    ///         }
    ///
    ///         /// Allows incrementing the contract's `value` only
    ///         /// for the owner.
    ///         ///
    ///         /// The contract panics if the caller is not the owner.
    ///         #[ink(message)]
    ///         pub fn increment(&mut self) {
    ///             let caller = self.env().account_id();
    ///             assert!(self.owner == caller);
    ///             self.value = self.value + 1;
    ///         }
    ///     }
    /// }
    /// ```
    ///
    /// # Note
    ///
    /// For more details visit: [`ink_env::account_id`]
    pub fn account_id(self) -> E::AccountId {
        ink_env::account_id::<E>()
    }

    /// Returns the address of the executed contract.
    ///
    /// For more details visit: [`ink_env::address`]
    pub fn address(self) -> Address {
        ink_env::address()
    }

    /// Returns the balance of the executed contract.
    ///
    /// # Example
    ///
    /// ```
    /// # #[ink::contract]
    /// # pub mod my_contract {
    /// #     #[ink(storage)]
    /// #     pub struct MyContract { }
    /// #
    /// #     impl MyContract {
    /// #         #[ink(constructor)]
    /// #         pub fn new() -> Self {
    /// #             Self {}
    /// #         }
    /// #
    /// /// Returns the contract's balance.
    /// #[ink(message)]
    /// pub fn my_balance(&self) -> ink::U256 {
    ///     self.env().balance()
    /// }
    /// #
    /// #     }
    /// # }
    /// ```
    ///
    /// # Note
    ///
    /// For more details visit: [`ink_env::balance`]
    pub fn balance(self) -> U256 {
        ink_env::balance()
    }

    /// Returns the current block number.
    ///
    /// # Example
    ///
    /// ```
    /// #[ink::contract]
    /// pub mod my_contract {
    ///     #[ink(storage)]
    ///     pub struct MyContract {
    ///         last_invocation: BlockNumber,
    ///     }
    ///
    ///     impl MyContract {
    ///         #[ink(constructor)]
    ///         pub fn new() -> Self {
    ///             Self {
    ///                 last_invocation: Self::env().block_number(),
    ///             }
    ///         }
    ///
    ///         /// The function can be executed at most once every 100 blocks.
    ///         #[ink(message)]
    ///         pub fn execute_me(&mut self) {
    ///             let now = self.env().block_number();
    ///             assert!(now - self.last_invocation > 100);
    ///             self.last_invocation = now;
    ///         }
    ///     }
    /// }
    /// ```
    ///
    /// # Note
    ///
    /// For more details visit: [`ink_env::block_number`]
    pub fn block_number(self) -> E::BlockNumber {
        ink_env::block_number::<E>()
    }

    /// Returns the minimum balance that is required for creating an account
    /// (i.e. the chain's existential deposit).
    ///
    /// # Example
    ///
    /// ```
    /// # #[ink::contract]
    /// # pub mod my_contract {
    /// #     #[ink(storage)]
    /// #     pub struct MyContract { }
    /// #
    /// #     impl MyContract {
    /// #         #[ink(constructor)]
    /// #         pub fn new() -> Self {
    /// #             Self {}
    /// #         }
    /// #
    /// #[ink(message)]
    /// pub fn minimum_balance(&self) -> ink::U256 {
    ///     self.env().minimum_balance()
    /// }
    /// #
    /// #     }
    /// # }
    /// ```
    ///
    /// # Note
    ///
    /// For more details visit: [`ink_env::minimum_balance`]
    pub fn minimum_balance(self) -> U256 {
        ink_env::minimum_balance()
    }

    /// Emits an event.
    ///
    /// # Note
    ///
    /// In "all" ABI mode, both an ink! and Solidity ABI event are emitted.
    #[cfg(not(ink_abi = "all"))]
    pub fn emit_event<Evt>(self, event: Evt)
    where
        Evt: ink_env::Event<crate::env::DefaultAbi>,
    {
        ink_env::emit_event::<Evt>(event)
    }

    /// Emits an event.
    ///
    /// # Note
    ///
    /// In "all" ABI mode, both an ink! and Solidity ABI event are emitted.
    #[cfg(ink_abi = "all")]
    pub fn emit_event<Evt>(self, event: Evt)
    where
        Evt: ink_env::Event<crate::abi::Ink> + ink_env::Event<crate::abi::Sol>,
    {
        ink_env::emit_event::<Evt>(event)
    }

    /// Emits an event using the ink! ABI encoding (i.e. with SCALE codec for event data
    /// encode/decode).
    pub fn emit_event_ink<Evt>(self, event: Evt)
    where
        Evt: ink_env::Event<Ink>,
    {
        ink_env::emit_event_ink::<Evt>(event)
    }

    /// Emits an event using the Solidity ABI encoding.
    pub fn emit_event_sol<Evt>(self, event: Evt)
    where
        Evt: ink_env::Event<Sol>,
    {
        ink_env::emit_event_sol::<Evt>(event)
    }

    /// Instantiates another contract using the supplied code hash.
    ///
    /// Invokes the `instantiate_v2` host function which allows passing all weight and
    /// storage limit parameters.
    ///
    /// # Example
    ///
    /// ```
    /// # #[ink::contract]
    /// # pub mod my_contract {
    /// # // In order for this to actually work with another contract we'd need a way
    /// # // to turn the `ink-as-dependency` crate feature on in doctests, which we
    /// # // can't do.
    /// # //
    /// # // Instead we use our own contract's `Ref`, which is fine for this example
    /// # // (just need something that implements the `ContractRef` trait).
    /// # pub mod other_contract {
    /// #     pub use super::MyContractRef as OtherContractRef;
    /// # }
    /// use ink::env::{
    ///     call::{
    ///         build_create,
    ///         ExecutionInput,
    ///         Selector,
    ///     },
    ///     DefaultEnvironment,
    /// };
    /// use other_contract::OtherContractRef;
    /// #
    /// #     #[ink(storage)]
    /// #     pub struct MyContract { }
    /// #
    /// #     impl MyContract {
    /// #         #[ink(constructor)]
    /// #         pub fn new() -> Self {
    /// #             Self {}
    /// #         }
    /// #
    ///
    /// /// Instantiates another contract.
    /// #[ink(message)]
    /// pub fn instantiate_contract(&self) -> MyContractRef {
    ///     let create_params = build_create::<OtherContractRef>()
    ///         .code_hash(ink::H256::from([0x42; 32]))
    ///         .ref_time_limit(500_000_000)
    ///         .proof_size_limit(100_000)
    ///         .storage_deposit_limit(ink::U256::from(500_000_000_000u64))
    ///         .endowment(25.into())
    ///         .exec_input(
    ///             ExecutionInput::new(Selector::new(ink::selector_bytes!("new")))
    ///                 .push_arg(42)
    ///                 .push_arg(true)
    ///                 .push_arg(&[0x10u8; 32]),
    ///         )
    ///         .salt_bytes(Some([0x13; 32]))
    ///         .returns::<OtherContractRef>()
    ///         .params();
    ///     self.env()
    ///         .instantiate_contract(&create_params)
    ///         .unwrap_or_else(|error| {
    ///             panic!(
    ///                 "Received an error from `pallet-revive` while instantiating: {error:?}"
    ///             )
    ///         })
    ///         .unwrap_or_else(|error| {
    ///             panic!("Received a `LangError` while instantiating: {error:?}")
    ///         })
    /// }
    /// #
    /// #     }
    /// # }
    /// ```
    ///
    /// See [our `delegator` example](https://github.com/use-ink/ink-examples/tree/main/upgradeable-contracts#delegator)
    /// for a complete contract example.
    ///
    /// # Note
    ///
    /// For more details visit: [`ink_env::instantiate_contract`]
    pub fn instantiate_contract<ContractRef, Args, R, Abi>(
        self,
        params: &CreateParams<E, ContractRef, LimitParamsV2, Args, R, Abi>,
    ) -> Result<
        ink_primitives::ConstructorResult<
            <R as ConstructorReturnType<ContractRef, Abi>>::Output,
        >,
    >
    where
        ContractRef: FromAddr + ink_env::ContractReverseReference,
        <ContractRef as ink_env::ContractReverseReference>::Type:
            ink_env::reflect::ContractConstructorDecoder,
        Args: AbiEncodeWith<Abi>,
        R: ConstructorReturnType<ContractRef, Abi>,
    {
        ink_env::instantiate_contract::<E, ContractRef, Args, R, Abi>(params)
    }

    /// Invokes a contract message and returns its result.
    ///
    /// # Example
    ///
    /// ```
    /// # #[ink::contract]
    /// # pub mod my_contract {
    /// use ink::env::{
    ///     DefaultEnvironment,
    ///     call::{
    ///         ExecutionInput,
    ///         Selector,
    ///         build_call,
    ///     },
    /// };
    ///
    /// #
    /// #     #[ink(storage)]
    /// #     pub struct MyContract { }
    /// #
    /// #     impl MyContract {
    /// #         #[ink(constructor)]
    /// #         pub fn new() -> Self {
    /// #             Self {}
    /// #         }
    /// #
    /// /// Invokes a contract message and fetches the result.
    /// #[ink(message)]
    /// pub fn invoke_contract(&self) -> i32 {
    ///     let call_params = build_call::<DefaultEnvironment>()
    ///         .call(ink::Address::from([0x42; 20]))
    ///         .ref_time_limit(500_000_000)
    ///         .proof_size_limit(100_000)
    ///         .storage_deposit_limit(1_000_000_000.into())
    ///         .exec_input(
    ///             ExecutionInput::new(Selector::new([0xCA, 0xFE, 0xBA, 0xBE]))
    ///                 .push_arg(42u8)
    ///                 .push_arg(true)
    ///                 .push_arg(&[0x10u8; 32]),
    ///         )
    ///         .returns::<i32>()
    ///         .params();
    ///
    ///     self.env()
    ///         .invoke_contract(&call_params)
    ///         .unwrap_or_else(|env_err| {
    ///             panic!("Received an error from the Environment: {:?}", env_err)
    ///         })
    ///         .unwrap_or_else(|lang_err| panic!("Received a `LangError`: {:?}", lang_err))
    /// }
    /// #
    /// #     }
    /// # }
    /// ```
    ///
    /// # Note
    ///
    /// For more details visit: [`ink_env::invoke_contract`]
    pub fn invoke_contract<Args, R, Abi>(
        self,
        params: &CallParams<E, Call, Args, R, Abi>,
    ) -> Result<ink_primitives::MessageResult<R>>
    where
        Args: AbiEncodeWith<Abi>,
        R: DecodeMessageResult<Abi>,
    {
        ink_env::invoke_contract::<E, Args, R, Abi>(params)
    }

    /// Invokes in delegate manner a code message and returns its result.
    ///
    /// # Example
    ///
    /// ```
    /// # #[ink::contract]
    /// # pub mod my_contract {
    /// use ink::env::{
    ///     DefaultEnvironment,
    ///     call::{
    ///         DelegateCall,
    ///         ExecutionInput,
    ///         Selector,
    ///         build_call,
    ///         utils::ReturnType,
    ///     },
    /// };
    /// use ink_primitives::Clear;
    ///
    /// #
    /// #     #[ink(storage)]
    /// #     pub struct MyContract { }
    /// #
    /// #     impl MyContract {
    /// #         #[ink(constructor)]
    /// #         pub fn new() -> Self {
    /// #             Self {}
    /// #         }
    /// #
    /// /// Invokes in delegate manner a contract message and fetches the result.
    /// #[ink(message)]
    /// pub fn invoke_contract_delegate(&self) -> i32 {
    ///     let call_params = build_call::<DefaultEnvironment>()
    ///         .call_type(DelegateCall::new(ink::Address::zero()))
    ///         .exec_input(
    ///             ExecutionInput::new(Selector::new([0xCA, 0xFE, 0xBA, 0xBE]))
    ///                 .push_arg(42u8)
    ///                 .push_arg(true)
    ///                 .push_arg(&[0x10u8; 32]),
    ///         )
    ///         .returns::<i32>()
    ///         .params();
    ///     self.env()
    ///         .invoke_contract_delegate(&call_params)
    ///         .unwrap_or_else(|env_err| {
    ///             panic!("Received an error from the Environment: {:?}", env_err)
    ///         })
    ///         .unwrap_or_else(|lang_err| panic!("Received a `LangError`: {:?}", lang_err))
    /// }
    /// #
    /// #     }
    /// # }
    /// ```
    ///
    /// # Note
    ///
    /// For more details visit: [`ink_env::invoke_contract_delegate`]
    pub fn invoke_contract_delegate<Args, R, Abi>(
        self,
        params: &CallParams<E, DelegateCall, Args, R, Abi>,
    ) -> Result<ink_primitives::MessageResult<R>>
    where
        Args: AbiEncodeWith<Abi>,
        R: DecodeMessageResult<Abi>,
    {
        ink_env::invoke_contract_delegate::<E, Args, R, Abi>(params)
    }

    /// Terminates the existence of a contract.
    ///
    /// # Example
    ///
    /// ```
    /// # #[ink::contract]
    /// # pub mod my_contract {
    /// #     #[ink(storage)]
    /// #     pub struct MyContract { }
    /// #
    /// #     impl MyContract {
    /// #         #[ink(constructor)]
    /// #         pub fn new() -> Self {
    /// #             Self {}
    /// #         }
    /// #
    /// /// Terminates with the caller as beneficiary.
    /// #[ink(message)]
    /// pub fn terminate_me(&mut self) {
    ///     // todo check this example. if caller returns origin it's no longer possible.
    ///     self.env().terminate_contract(self.env().caller());
    /// }
    /// #
    /// #     }
    /// # }
    /// ```
    ///
    /// # Note
    ///
    /// For more details visit: [`ink_env::terminate_contract`]
    #[cfg(feature = "unstable-hostfn")]
    pub fn terminate_contract(self, beneficiary: Address) -> ! {
        ink_env::terminate_contract(beneficiary)
    }

    // Transfers value from the current contract to the destination contract.
    //
    // # Example
    //
    // ```
    // # #[ink::contract]
    // # pub mod my_contract {
    // #     #[ink(storage)]
    // #     pub struct MyContract { }
    // #
    // #     impl MyContract {
    // #         #[ink(constructor)]
    // #         pub fn new() -> Self {
    // #             Self {}
    // #         }
    // #
    // /// Transfers the token amount ten to the caller.
    // #[ink(message)]
    // pub fn give_me_ten(&mut self) {
    //     let value: Balance = 10;
    //     self.env()
    //         .transfer(self.env().caller(), value)
    //         .unwrap_or_else(|err| panic!("transfer failed: {:?}", err));
    // }
    // #
    // #     }
    // # }
    // ```
    //
    // # Note
    //
    // For more details visit: [`ink_env::transfer`]
    pub fn transfer(self, destination: Address, value: U256) -> Result<()> {
        // todo remove E
        ink_env::transfer::<E>(destination, value)
    }

    /// Computes the hash of the given bytes using the cryptographic hash `H`.
    ///
    /// # Example
    ///
    /// ```
    /// use ink_env::hash::{
    ///     HashOutput,
    ///     Sha2x256,
    /// };
    ///
    /// let input: &[u8] = &[13, 14, 15];
    /// let mut output = <Sha2x256 as HashOutput>::Type::default(); // 256-bit buffer
    /// let hash = ink_env::hash_bytes::<Sha2x256>(input, &mut output);
    /// ```
    ///
    /// # Note
    ///
    /// For more details visit: [`ink_env::hash_bytes`]
    pub fn hash_bytes<H>(self, input: &[u8]) -> <H as HashOutput>::Type
    where
        H: CryptoHash,
    {
        let mut output = <H as HashOutput>::Type::default();
        ink_env::hash_bytes::<H>(input, &mut output);
        output
    }

    /// Computes the hash of the given SCALE encoded value using the cryptographic hash
    /// `H`.
    ///
    /// # Example
    ///
    /// ```
    /// use ink_env::hash::{
    ///     HashOutput,
    ///     Sha2x256,
    /// };
    ///
    /// let encodable = (42, "foo", true); // Implements `scale::Encode`
    /// let mut output = <Sha2x256 as HashOutput>::Type::default(); // 256-bit buffer
    /// ink_env::hash_encoded::<Sha2x256, _>(&encodable, &mut output);
    ///
    /// const EXPECTED: [u8; 32] = [
    ///     243, 242, 58, 110, 205, 68, 100, 244, 187, 55, 188, 248, 29, 136, 145, 115, 186,
    ///     134, 14, 175, 178, 99, 183, 21, 4, 94, 92, 69, 199, 207, 241, 179,
    /// ];
    /// assert_eq!(output, EXPECTED);
    /// ```
    ///
    /// # Note
    ///
    /// For more details visit: [`ink_env::hash_encoded`]
    pub fn hash_encoded<H, V>(self, value: &V) -> <H as HashOutput>::Type
    where
        H: CryptoHash,
        V: scale::Encode,
    {
        let mut output = <H as HashOutput>::Type::default();
        ink_env::hash_encoded::<H, V>(value, &mut output);
        output
    }

    /// Recovers the compressed ECDSA public key for given `signature` and `message_hash`,
    /// and stores the result in `output`.
    ///
    /// # Example
    ///
    /// ```
    /// # #[ink::contract]
    /// # pub mod my_contract {
    /// #     #[ink(storage)]
    /// #     pub struct MyContract { }
    /// #
    /// #     impl MyContract {
    /// #         #[ink(constructor)]
    /// #         pub fn new() -> Self {
    /// #             Self {}
    /// #         }
    /// #
    /// /// Recovery from pre-defined signature and message hash
    /// #[ink(message)]
    /// pub fn ecdsa_recover(&self) {
    ///     const signature: [u8; 65] = [
    ///         195, 218, 227, 165, 226, 17, 25, 160, 37, 92, 142, 238, 4, 41, 244, 211, 18,
    ///         94, 131, 116, 231, 116, 255, 164, 252, 248, 85, 233, 173, 225, 26, 185, 119,
    ///         235, 137, 35, 204, 251, 134, 131, 186, 215, 76, 112, 17, 192, 114, 243, 102,
    ///         166, 176, 140, 180, 124, 213, 102, 117, 212, 89, 89, 92, 209, 116, 17, 28,
    ///     ];
    ///     const message_hash: [u8; 32] = [
    ///         167, 124, 116, 195, 220, 156, 244, 20, 243, 69, 1, 98, 189, 205, 79, 108,
    ///         213, 78, 65, 65, 230, 30, 17, 37, 184, 220, 237, 135, 1, 209, 101, 229,
    ///     ];
    ///     const EXPECTED_COMPRESSED_PUBLIC_KEY: [u8; 33] = [
    ///         3, 110, 192, 35, 209, 24, 189, 55, 218, 250, 100, 89, 40, 76, 222, 208, 202,
    ///         127, 31, 13, 58, 51, 242, 179, 13, 63, 19, 22, 252, 164, 226, 248, 98,
    ///     ];
    ///     let result = self.env().ecdsa_recover(&signature, &message_hash);
    ///     assert!(result.is_ok());
    ///     assert_eq!(
    ///         result.unwrap().as_ref(),
    ///         EXPECTED_COMPRESSED_PUBLIC_KEY.as_ref()
    ///     );
    ///
    ///     // Pass invalid zero message hash
    ///     let failed_result = self.env().ecdsa_recover(&signature, &[0; 32]);
    ///     assert!(failed_result.is_err());
    ///     if let Err(e) = failed_result {
    ///         assert_eq!(e, ink::env::ReturnErrorCode::EcdsaRecoveryFailed.into());
    ///     }
    /// }
    /// #
    /// #     }
    /// # }
    /// ```
    pub fn ecdsa_recover(
        self,
        signature: &[u8; 65],
        message_hash: &[u8; 32],
    ) -> Result<[u8; 33]> {
        let mut output = [0; 33];
        ink_env::ecdsa_recover(signature, message_hash, &mut output)
            .map(|_| output)
            .map_err(|_| ReturnErrorCode::EcdsaRecoveryFailed.into())
    }

    /// Returns an Ethereum address from the ECDSA compressed public key.
    ///
    /// # Example
    ///
    /// ```
    /// # #[ink::contract]
    /// # pub mod my_contract {
    /// #     #[ink(storage)]
    /// #     pub struct MyContract { }
    /// #
    /// #     impl MyContract {
    /// #         #[ink(constructor)]
    /// #         pub fn new() -> Self {
    /// #             Self {}
    /// #         }
    /// #
    /// #[ink(message)]
    /// pub fn ecdsa_to_eth_address(&self) {
    ///     let pub_key = [
    ///         3, 110, 192, 35, 209, 24, 189, 55, 218, 250, 100, 89, 40, 76, 222, 208, 202,
    ///         127, 31, 13, 58, 51, 242, 179, 13, 63, 19, 22, 252, 164, 226, 248, 98,
    ///     ];
    ///     let EXPECTED_ETH_ADDRESS = [
    ///         253, 240, 181, 194, 143, 66, 163, 109, 18, 211, 78, 49, 177, 94, 159, 79,
    ///         207, 37, 21, 191,
    ///     ];
    ///     let output = self
    ///         .env()
    ///         .ecdsa_to_eth_address(&pub_key)
    ///         .unwrap_or_else(|err| {
    ///             panic!(
    ///                 "must return an Ethereum address for the compressed public key: {:?}",
    ///                 err
    ///             )
    ///         });
    ///     assert_eq!(output, EXPECTED_ETH_ADDRESS);
    /// }
    /// #
    /// #     }
    /// # }
    /// ```
    ///
    /// # Note
    ///
    /// For more details visit: [`ink_env::ecdsa_to_eth_address`]
    #[cfg(feature = "unstable-hostfn")]
    pub fn ecdsa_to_eth_address(self, pubkey: &[u8; 33]) -> Result<[u8; 20]> {
        let mut output = [0; 20];
        ink_env::ecdsa_to_eth_address(pubkey, &mut output)
            .map(|_| output)
            .map_err(|_| ReturnErrorCode::EcdsaRecoveryFailed.into())
    }

    /// Verifies a SR25519 signature against a message and a public key.
    ///
    /// # Example
    ///
    /// ```
    /// # #[ink::contract]
    /// # pub mod my_contract {
    /// #     #[ink(storage)]
    /// #     pub struct MyContract { }
    /// #
    /// #     impl MyContract {
    /// #         #[ink(constructor)]
    /// #         pub fn new() -> Self {
    /// #             Self {}
    /// #         }
    /// #
    /// #[ink(message)]
    /// pub fn sr25519_verify(&self) {
    ///     let mut signature: [u8; 64] = [
    ///         10, 125, 162, 182, 49, 112, 76, 220, 254, 147, 199, 64, 228, 18, 23, 185,
    ///         172, 102, 122, 12, 135, 85, 216, 218, 26, 130, 50, 219, 82, 127, 72, 124,
    ///         135, 231, 128, 210, 237, 193, 137, 106, 235, 107, 27, 239, 11, 199, 195, 141,
    ///         157, 242, 19, 91, 99, 62, 171, 139, 251, 23, 119, 232, 47, 173, 58, 143,
    ///     ];
    ///     let mut message: [u8; 49] = [
    ///         60, 66, 121, 116, 101, 115, 62, 48, 120, 52, 54, 102, 98, 55, 52, 48, 56,
    ///         100, 52, 102, 50, 56, 53, 50, 50, 56, 102, 52, 97, 102, 53, 49, 54, 101, 97,
    ///         50, 53, 56, 53, 49, 98, 60, 47, 66, 121, 116, 101, 115, 62,
    ///     ];
    ///     let mut public_key: [u8; 32] = [
    ///         212, 53, 147, 199, 21, 253, 211, 28, 97, 20, 26, 189, 4, 169, 159, 214, 130,
    ///         44, 133, 88, 133, 76, 205, 227, 154, 86, 132, 231, 165, 109, 162, 125,
    ///     ];
    ///     let result = ink::env::sr25519_verify(&signature, &message, &public_key);
    ///     assert_eq!(result, Ok(()));
    /// }
    /// #
    /// #     }
    /// # }
    /// ```
    ///
    /// # Note
    ///
    /// The context for sr25519 signing is hard-coded to "substrate" to match sr25519
    /// signing in substrate.
    ///
    /// For more details visit: [`ink_env::sr25519_verify`]
    ///
    /// todo
    /// **WARNING**: this function is from the [unstable interface](https://github.com/paritytech/substrate/tree/master/frame/contracts#unstable-interfaces),
    /// which is unsafe and normally is not available on production chains.
    #[cfg(feature = "unstable-hostfn")]
    pub fn sr25519_verify(
        self,
        signature: &[u8; 64],
        message: &[u8],
        pub_key: &[u8; 32],
    ) -> Result<()> {
        ink_env::sr25519_verify(signature, message, pub_key)
            .map_err(|_| ReturnErrorCode::Sr25519VerifyFailed.into())
    }

    /// Checks whether a contract lives under `addr`.
    /// todo update comment
    ///
    /// # Example
    ///
    /// ```
    /// # #[ink::contract]
    /// # pub mod my_contract {
    /// #     #[ink(storage)]
    /// #     pub struct MyContract { }
    /// #
    /// #     impl MyContract {
    /// #         #[ink(constructor)]
    /// #         pub fn new() -> Self {
    /// #             Self {}
    /// #         }
    /// #
    /// #[ink(message)]
    /// pub fn is_contract(&mut self, addr: ink::Address) -> bool {
    ///     self.env().is_contract(&addr)
    /// }
    /// #    }
    /// # }
    /// ```
    ///
    /// # Note
    ///
    /// For more details visit: [`ink_env::is_contract`]
    #[cfg(feature = "unstable-hostfn")]
    pub fn is_contract(self, addr: &Address) -> bool {
        ink_env::is_contract(addr)
    }

    /// Checks whether the caller of the current contract is the origin of the whole call
    /// stack.
    ///
    /// # Example
    ///
    /// ```
    /// # #[ink::contract]
    /// # pub mod my_contract {
    /// #     #[ink(storage)]
    /// #     pub struct MyContract { }
    /// #
    /// #     impl MyContract {
    /// #         #[ink(constructor)]
    /// #         pub fn new() -> Self {
    /// #             Self {}
    /// #         }
    /// #
    /// #[ink(message)]
    /// pub fn caller_is_origin(&mut self) -> bool {
    ///     self.env().caller_is_origin()
    /// }
    /// #    }
    /// # }
    /// ```
    ///
    /// # Note
    ///
    /// For more details visit: [`ink_env::caller_is_origin`]
    pub fn caller_is_origin(self) -> bool {
        ink_env::caller_is_origin::<E>()
    }

    /// Checks whether the caller of the current contract is root.
    ///
    /// # Example
    ///
    /// ```
    /// # #[ink::contract]
    /// # pub mod my_contract {
    /// #     #[ink(storage)]
    /// #     pub struct MyContract { }
    /// #
    /// #     impl MyContract {
    /// #         #[ink(constructor)]
    /// #         pub fn new() -> Self {
    /// #             Self {}
    /// #         }
    /// #
    /// #[ink(message)]
    /// pub fn caller_is_root(&mut self) -> bool {
    ///     self.env().caller_is_root()
    /// }
    /// #    }
    /// # }
    /// ```
    ///
    /// # Note
    ///
    /// For more details visit: [`ink_env::caller_is_root`]
    pub fn caller_is_root(self) -> bool {
        ink_env::caller_is_root::<E>()
    }

    /// Returns the code hash of the contract at the given `account` id.
    ///
    /// # Example
    ///
    /// ```
    /// # #[ink::contract]
    /// # pub mod my_contract {
    /// #     #[ink(storage)]
    /// #     pub struct MyContract { }
    /// #
    /// #     impl MyContract {
    /// #         #[ink(constructor)]
    /// #         pub fn new() -> Self {
    /// #             Self {}
    /// #         }
    /// #
    /// #[ink(message)]
    /// // todo
    /// pub fn code_hash(&mut self, addr: ink::Address) -> Option<ink::H256> {
    ///     self.env().code_hash(&addr).ok()
    /// }
    /// #    }
    /// # }
    /// ```
    ///
    /// # Note
    ///
    /// For more details visit: [`ink_env::code_hash`]
    pub fn code_hash(self, addr: &Address) -> Result<H256> {
        ink_env::code_hash(addr)
    }

    /// Returns the code hash of the contract at the given `account` id.
    ///
    /// # Example
    ///
    /// ```
    /// # #[ink::contract]
    /// # pub mod my_contract {
    /// #     #[ink(storage)]
    /// #     pub struct MyContract { }
    /// #
    /// #     impl MyContract {
    /// #         #[ink(constructor)]
    /// #         pub fn new() -> Self {
    /// #             Self {}
    /// #         }
    /// #
    /// #[ink(message)]
    /// pub fn own_code_hash(&mut self) -> ink::H256 {
    ///     self.env()
    ///         .own_code_hash()
    ///         .unwrap_or_else(|err| panic!("contract should have a code hash: {:?}", err))
    /// }
    /// #    }
    /// # }
    /// ```
    ///
    /// # Note
    ///
    /// For more details visit: [`ink_env::own_code_hash`]
    pub fn own_code_hash(self) -> Result<H256> {
        ink_env::own_code_hash()
    }

    /// Replace the contract code at the specified address with new code.
    ///
    /// # Example
    ///
    /// ```
    /// # #[ink::contract]
    /// # pub mod my_contract {
    /// #     #[ink(storage)]
    /// #     pub struct MyContract { }
    /// #
    /// #     impl MyContract {
    /// #         #[ink(constructor)]
    /// #         pub fn new() -> Self {
    /// #             Self {}
    /// #         }
    /// #
    /// #[ink(message)]
    /// pub fn set_code_hash(&mut self, code_hash: ink::H256) {
    ///     self.env()
    ///         .set_code_hash(&code_hash)
    ///         .unwrap_or_else(|err| panic!("failed to set code hash: {:?}", err))
    /// }
    /// #    }
    /// # }
    /// ```
    ///
    /// # Note
    ///
    /// For more details visit: [`ink_env::set_code_hash`]
    #[cfg(feature = "unstable-hostfn")]
    pub fn set_code_hash(self, code_hash: &H256) -> Result<()> {
        ink_env::set_code_hash::<E>(code_hash)
    }

    #[cfg(all(feature = "xcm", feature = "unstable-hostfn"))]
    pub fn xcm_execute<Call: scale::Encode>(
        self,
        msg: &xcm::VersionedXcm<Call>,
    ) -> Result<()> {
        ink_env::xcm_execute::<E, _>(msg)
    }

    #[cfg(all(feature = "xcm", feature = "unstable-hostfn"))]
    pub fn xcm_send<Call: scale::Encode>(
        self,
        dest: &xcm::VersionedLocation,
        msg: &xcm::VersionedXcm<Call>,
    ) -> Result<xcm::v4::XcmHash> {
        ink_env::xcm_send::<E, _>(dest, msg)
    }
}
