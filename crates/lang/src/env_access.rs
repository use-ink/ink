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

use crate::ChainExtensionInstance;
use core::marker::PhantomData;
use ink_env::{
    call::{
        Call,
        CallParams,
        CreateParams,
        DelegateCall,
    },
    hash::{
        CryptoHash,
        HashOutput,
    },
    Environment,
    Error,
    Result,
};

/// The API behind the `self.env()` and `Self::env()` syntax in ink!.
///
/// This allows ink! messages to make use of the environment efficiently
/// and user friendly while also maintaining access invariants.
#[derive(Copy, Clone)]
pub struct EnvAccess<'a, E> {
    /// Tricks the Rust compiler into thinking that we use `E`.
    marker: PhantomData<fn() -> &'a E>,
}

impl<'a, E> Default for EnvAccess<'a, E> {
    #[inline]
    fn default() -> Self {
        Self {
            marker: Default::default(),
        }
    }
}

impl<'a, E> core::fmt::Debug for EnvAccess<'a, E> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.debug_struct("EnvAccess").finish()
    }
}

impl<'a, E> EnvAccess<'a, E>
where
    E: Environment,
    <E as Environment>::ChainExtension: ChainExtensionInstance,
{
    /// Allows to call one of the available defined chain extension methods.
    pub fn extension(
        self,
    ) -> <<E as Environment>::ChainExtension as ChainExtensionInstance>::Instance {
        <<E as Environment>::ChainExtension as ChainExtensionInstance>::instantiate()
    }
}

impl<'a, E> EnvAccess<'a, E>
where
    E: Environment,
{
    /// Returns the address of the caller of the executed contract.
    ///
    /// # Example
    ///
    /// ```
    /// # use ink_lang as ink;
    /// #
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
    ///     ink_env::debug_println!("got a call from {:?}", &caller);
    /// }
    /// #
    /// #     }
    /// # }
    /// ```
    ///
    /// # Note
    ///
    /// For more details visit: [`ink_env::caller`]
    pub fn caller(self) -> E::AccountId {
        ink_env::caller::<E>()
    }

    /// Returns the transferred value for the contract execution.
    ///
    /// # Example
    ///
    /// ```
    /// # use ink_lang as ink;
    /// #
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
    /// pub fn fund(&self) {
    ///     let caller = self.env().caller();
    ///     let value = self.env().transferred_value();
    ///     ink_env::debug_println!("thanks for the funding of {:?} from {:?}", value, caller);
    /// }
    /// #
    /// #     }
    /// # }
    /// ```
    ///
    /// # Note
    ///
    /// For more details visit: [`ink_env::transferred_value`]
    pub fn transferred_value(self) -> E::Balance {
        ink_env::transferred_value::<E>()
    }

    /// Returns the price for the specified amount of gas.
    ///
    /// # Example
    ///
    /// ```
    /// # use ink_lang as ink;
    /// #
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
    /// /// Returns a tuple of
    /// ///   - the result of adding the `rhs` to the `lhs`
    /// ///   - the gas costs of this addition operation
    /// ///   - the price for the gas
    /// #[ink(message)]
    /// pub fn addition_gas_cost(&self, rhs: i32, lhs: i32) -> (i32, u64, Balance) {
    ///     let before = self.env().gas_left();
    ///     let result = rhs + lhs;
    ///     let after = self.env().gas_left();
    ///     let gas_used = after - before;
    ///     let gas_cost = self.env().weight_to_fee(gas_used);
    ///     (result, gas_used, gas_cost)
    /// }
    /// #
    /// #     }
    /// # }
    /// ```
    ///
    /// # Note
    ///
    /// For more details visit: [`ink_env::weight_to_fee`]
    pub fn weight_to_fee(self, gas: u64) -> E::Balance {
        ink_env::weight_to_fee::<E>(gas)
    }

    /// Returns the amount of gas left for the contract execution.
    ///
    /// # Example
    ///
    /// ```
    /// # use ink_lang as ink;
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
    /// /// Returns a tuple of
    /// ///   - the result of adding the `rhs` to the `lhs` and
    /// ///   - the gas used for this addition operation.
    /// #[ink(message)]
    /// pub fn addition_gas_cost(&self, rhs: i32, lhs: i32) -> (i32, u64) {
    ///     let before = self.env().gas_left();
    ///     let result = rhs + lhs;
    ///     let after = self.env().gas_left();
    ///     (result, after - before)
    /// }
    /// #
    /// #     }
    /// # }
    /// ```
    ///
    /// # Note
    ///
    /// For more details visit: [`ink_env::gas_left`]
    pub fn gas_left(self) -> u64 {
        ink_env::gas_left::<E>()
    }

    /// Returns the timestamp of the current block.
    ///
    /// # Example
    ///
    /// ```
    /// use ink_lang as ink;
    ///
    /// #[ink::contract]
    /// pub mod my_contract {
    ///     #[ink(storage)]
    ///     pub struct MyContract {
    ///         last_invocation: Timestamp
    ///     }
    ///
    ///     impl MyContract {
    ///         #[ink(constructor)]
    ///         pub fn new() -> Self {
    ///             Self {
    ///                 last_invocation: Self::env().block_timestamp()
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

    /// Returns the account ID of the executed contract.
    ///
    /// # Example
    ///
    /// ```
    /// use ink_lang as ink;
    ///
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
    ///                 owner: Self::env().caller(),
    ///                 value: 0,
    ///             }
    ///         }
    ///
    ///         /// Allows incrementing the contract's `value` only
    ///         /// for the owner (i.e. the account which instantiated
    ///         /// this contract.
    ///         ///
    ///         /// The contract panics if the caller is not the owner.
    ///         #[ink(message)]
    ///         pub fn increment(&mut self) {
    ///             let caller = self.env().caller();
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

    /// Returns the balance of the executed contract.
    ///
    /// # Example
    ///
    /// ```
    /// # use ink_lang as ink;
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
    /// pub fn my_balance(&self) -> Balance {
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
    pub fn balance(self) -> E::Balance {
        ink_env::balance::<E>()
    }

    /// Returns the current block number.
    ///
    /// # Example
    ///
    /// ```
    /// use ink_lang as ink;
    ///
    /// #[ink::contract]
    /// pub mod my_contract {
    ///     #[ink(storage)]
    ///     pub struct MyContract {
    ///         last_invocation: BlockNumber
    ///     }
    ///
    ///     impl MyContract {
    ///         #[ink(constructor)]
    ///         pub fn new() -> Self {
    ///             Self {
    ///                 last_invocation: Self::env().block_number()
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
    /// # use ink_lang as ink;
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
    /// pub fn minimum_balance(&self) -> Balance {
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
    pub fn minimum_balance(self) -> E::Balance {
        ink_env::minimum_balance::<E>()
    }

    /// todo: [AJ] docs
    pub fn emit_event<Event>(self, event: Event)
        where
            Event: ink_env::Topics + scale::Encode,
    {
        ink_env::emit_event::<E, Event>(event)
    }

    /// Instantiates another contract.
    ///
    /// # Example
    ///
    /// ```
    /// # use ink_lang as ink;
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
    /// use ink_env::{
    ///     DefaultEnvironment,
    ///     call::{build_create, Selector, ExecutionInput}
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
    /// pub fn instantiate_contract(&self) -> AccountId {
    ///     let create_params = build_create::<DefaultEnvironment, OtherContractRef>()
    ///         .code_hash(Hash::from([0x42; 32]))
    ///         .gas_limit(4000)
    ///         .endowment(25)
    ///         .exec_input(
    ///             ExecutionInput::new(Selector::new([0xCA, 0xFE, 0xBA, 0xBE]))
    ///                 .push_arg(42)
    ///                 .push_arg(true)
    ///                 .push_arg(&[0x10u8; 32])
    ///             )
    ///         .salt_bytes(&[0xCA, 0xFE, 0xBA, 0xBE])
    ///         .params();
    ///     self.env().instantiate_contract(&create_params).expect("instantiation must succeed")
    /// }
    /// #
    /// #     }
    /// # }
    /// ```
    ///
    /// See [our `delegator` example](https://github.com/paritytech/ink/tree/master/examples/delegator)
    /// for a complete contract example.
    ///
    /// # Note
    ///
    /// For more details visit: [`ink_env::instantiate_contract`]
    pub fn instantiate_contract<Args, Salt, C>(
        self,
        params: &CreateParams<E, Args, Salt, C>,
    ) -> Result<E::AccountId>
    where
        Args: scale::Encode,
        Salt: AsRef<[u8]>,
    {
        ink_env::instantiate_contract::<E, Args, Salt, C>(params)
    }

    /// Invokes a contract message and returns its result.
    ///
    /// # Example
    ///
    /// ```
    /// # use ink_lang as ink;
    /// # #[ink::contract]
    /// # pub mod my_contract {
    /// use ink_env::{
    ///     DefaultEnvironment,
    ///     call::{build_call, Call, Selector, ExecutionInput}
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
    ///             .call_type(
    ///                 Call::new()
    ///                     .callee(AccountId::from([0x42; 32]))
    ///                     .gas_limit(5000)
    ///                     .transferred_value(10))
    ///             .exec_input(
    ///                 ExecutionInput::new(Selector::new([0xCA, 0xFE, 0xBA, 0xBE]))
    ///                  .push_arg(42u8)
    ///                  .push_arg(true)
    ///                  .push_arg(&[0x10u8; 32])
    ///     )
    ///     .returns::<i32>()
    ///     .params();
    ///     self.env().invoke_contract(&call_params).expect("call invocation must succeed")
    /// }
    /// #
    /// #     }
    /// # }
    /// ```
    ///
    /// # Note
    ///
    /// For more details visit: [`ink_env::invoke_contract`]
    pub fn invoke_contract<Args, R>(
        self,
        params: &CallParams<E, Call<E>, Args, R>,
    ) -> Result<R>
    where
        Args: scale::Encode,
        R: scale::Decode,
    {
        ink_env::invoke_contract::<E, Args, R>(params)
    }

    /// Invokes in delegate manner a code message and returns its result.
    ///
    /// # Example
    ///
    /// ```
    /// # use ink_lang as ink;
    /// # #[ink::contract]
    /// # pub mod my_contract {
    /// use ink_env::{
    ///     DefaultEnvironment,
    ///     Clear,
    ///     call::{build_call, DelegateCall, Selector, ExecutionInput, utils::ReturnType}
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
    /// /// Invokes in delegate manner a contract message and fetches the result.
    /// #[ink(message)]
    /// pub fn invoke_contract_delegate(&self) -> i32 {
    ///     let call_params = build_call::<DefaultEnvironment>()
    ///             .call_type(
    ///                 DelegateCall::new()
    ///                  .code_hash(<DefaultEnvironment as ink_env::Environment>::Hash::clear()))
    ///             .exec_input(
    ///                 ExecutionInput::new(Selector::new([0xCA, 0xFE, 0xBA, 0xBE]))
    ///                  .push_arg(42u8)
    ///                  .push_arg(true)
    ///                  .push_arg(&[0x10u8; 32])
    ///         )
    ///         .returns::<i32>()
    ///         .params();
    ///     self.env().invoke_contract_delegate(&call_params).expect("call delegate invocation must succeed")
    /// }
    /// #
    /// #     }
    /// # }
    /// ```
    ///
    /// # Note
    ///
    /// For more details visit: [`ink_env::invoke_contract_delegate`]
    pub fn invoke_contract_delegate<Args, R>(
        self,
        params: &CallParams<E, DelegateCall<E>, Args, R>,
    ) -> Result<R>
    where
        Args: scale::Encode,
        R: scale::Decode,
    {
        ink_env::invoke_contract_delegate::<E, Args, R>(params)
    }

    /// Terminates the existence of a contract.
    ///
    /// # Example
    ///
    /// ```
    /// # use ink_lang as ink;
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
    pub fn terminate_contract(self, beneficiary: E::AccountId) -> ! {
        ink_env::terminate_contract::<E>(beneficiary)
    }

    /// Transfers value from the contract to the destination account ID.
    ///
    /// # Example
    ///
    /// ```
    /// # use ink_lang as ink;
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
    /// /// Transfers the token amount ten to the caller.
    /// #[ink(message)]
    /// pub fn give_me_ten(&mut self) {
    ///     let value: Balance = 10;
    ///     self.env().transfer(self.env().caller(), value).expect("transfer failed");
    /// }
    /// #
    /// #     }
    /// # }
    /// ```
    ///
    /// # Note
    ///
    /// For more details visit: [`ink_env::transfer`]
    pub fn transfer(self, destination: E::AccountId, value: E::Balance) -> Result<()> {
        ink_env::transfer::<E>(destination, value)
    }

    /// Returns a random hash seed.
    ///
    /// # Example
    ///
    /// ```
    /// # use ink_lang as ink;
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
    /// pub fn random_bool(&self) -> bool {
    ///     let additional_randomness = b"seed";
    ///     let (hash, _block_number) = self.env().random(additional_randomness);
    ///     hash.as_ref()[0] != 0
    /// }
    /// #
    /// #     }
    /// # }
    /// ```
    ///
    /// # Note
    ///
    /// For more details visit: [`ink_env::random`]
    pub fn random(self, subject: &[u8]) -> (E::Hash, E::BlockNumber) {
        ink_env::random::<E>(subject).expect("couldn't decode randomized hash")
    }

    /// Computes the hash of the given bytes using the cryptographic hash `H`.
    ///
    /// # Example
    ///
    /// ```
    /// use ink_env::hash::{Sha2x256, HashOutput};
    ///
    /// let input: &[u8] = &[13, 14, 15];
    /// let mut output = <Sha2x256 as HashOutput>::Type::default(); // 256-bit buffer
    /// let hash  = ink_env::hash_bytes::<Sha2x256>(input, &mut output);
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

    /// Computes the hash of the given SCALE encoded value using the cryptographic hash `H`.
    ///
    /// # Example
    ///
    /// ```
    /// use ink_env::hash::{Sha2x256, HashOutput};
    ///
    /// let encodable = (42, "foo", true); // Implements `scale::Encode`
    /// let mut output = <Sha2x256 as HashOutput>::Type::default(); // 256-bit buffer
    /// ink_env::hash_encoded::<Sha2x256, _>(&encodable, &mut output);
    ///
    /// const EXPECTED: [u8; 32] = [
    ///   243, 242, 58, 110, 205, 68, 100, 244, 187, 55, 188, 248,  29, 136, 145, 115,
    ///   186, 134, 14, 175, 178, 99, 183,  21,   4, 94,  92,  69, 199, 207, 241, 179,
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
    /// # use ink_lang as ink;
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
    ///         195, 218, 227, 165, 226, 17, 25, 160, 37, 92, 142, 238, 4, 41, 244, 211, 18, 94,
    ///         131, 116, 231, 116, 255, 164, 252, 248, 85, 233, 173, 225, 26, 185, 119, 235,
    ///         137, 35, 204, 251, 134, 131, 186, 215, 76, 112, 17, 192, 114, 243, 102, 166, 176,
    ///         140, 180, 124, 213, 102, 117, 212, 89, 89, 92, 209, 116, 17, 28,
    ///     ];
    ///     const message_hash: [u8; 32] = [
    ///         167, 124, 116, 195, 220, 156, 244, 20, 243, 69, 1, 98, 189, 205, 79, 108, 213,
    ///         78, 65, 65, 230, 30, 17, 37, 184, 220, 237, 135, 1, 209, 101, 229,
    ///     ];
    ///     const EXPECTED_COMPRESSED_PUBLIC_KEY: [u8; 33] = [
    ///         3, 110, 192, 35, 209, 24, 189, 55, 218, 250, 100, 89, 40, 76, 222, 208, 202, 127,
    ///         31, 13, 58, 51, 242, 179, 13, 63, 19, 22, 252, 164, 226, 248, 98,
    ///     ];
    ///     let result = self.env().ecdsa_recover(&signature, &message_hash);
    ///     assert!(result.is_ok());
    ///     assert_eq!(result.unwrap().as_ref(), EXPECTED_COMPRESSED_PUBLIC_KEY.as_ref());
    ///
    ///     // Pass invalid zero message hash
    ///     let failed_result = self.env().ecdsa_recover(&signature, &[0; 32]);
    ///     assert!(failed_result.is_err());
    ///     if let Err(e) = failed_result {
    ///         assert_eq!(e, ink_env::Error::EcdsaRecoveryFailed);
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
            .map_err(|_| Error::EcdsaRecoveryFailed)
    }

    /// Returns an Ethereum address from the ECDSA compressed public key.
    ///
    /// # Example
    ///
    /// ```
    /// # use ink_lang as ink;
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
    ///         3, 110, 192, 35, 209, 24, 189, 55, 218, 250, 100, 89, 40, 76, 222, 208, 202, 127,
    ///         31, 13, 58, 51, 242, 179, 13, 63, 19, 22, 252, 164, 226, 248, 98,
    ///     ];
    ///     let EXPECTED_ETH_ADDRESS = [
    ///         253, 240, 181, 194, 143, 66, 163, 109, 18, 211, 78, 49, 177, 94, 159, 79, 207,
    ///         37, 21, 191,
    ///     ];
    ///     let output = self
    ///         .env()
    ///         .ecdsa_to_eth_address(&pub_key)
    ///         .expect("must return an Ethereum address for the compressed public key");
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
    pub fn ecdsa_to_eth_address(self, pubkey: &[u8; 33]) -> Result<[u8; 20]> {
        let mut output = [0; 20];
        ink_env::ecdsa_to_eth_address(pubkey, &mut output)
            .map(|_| output)
            .map_err(|_| Error::EcdsaRecoveryFailed)
    }

    /// Checks whether a specified account belongs to a contract.
    ///
    /// # Example
    ///
    /// ```
    /// # use ink_lang as ink;
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
    /// pub fn is_contract(&mut self, account_id: AccountId) -> bool {
    ///     self.env().is_contract(&account_id)
    /// }
    /// #    }
    /// # }
    /// ```
    ///
    /// # Note
    ///
    /// For more details visit: [`ink_env::is_contract`]
    pub fn is_contract(self, account_id: &E::AccountId) -> bool {
        ink_env::is_contract::<E>(account_id)
    }

    /// Checks whether the caller of the current contract is the origin of the whole call stack.
    ///
    /// # Example
    ///
    /// ```
    /// # use ink_lang as ink;
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

    /// Returns the code hash of the contract at the given `account` id.
    ///
    /// # Example
    ///
    /// ```
    /// # use ink_lang as ink;
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
    /// pub fn code_hash(&mut self, account_id: AccountId) -> Option<Hash> {
    ///     self.env().code_hash(&account_id).ok()
    /// }
    /// #    }
    /// # }
    /// ```
    ///
    /// # Note
    ///
    /// For more details visit: [`ink_env::code_hash`]
    pub fn code_hash(self, account_id: &E::AccountId) -> Result<E::Hash> {
        ink_env::code_hash::<E>(account_id)
    }

    /// Returns the code hash of the contract at the given `account` id.
    ///
    /// # Example
    ///
    /// ```
    /// # use ink_lang as ink;
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
    /// pub fn own_code_hash(&mut self) -> Hash {
    ///     self.env().own_code_hash().expect("contract should have a code hash")
    /// }
    /// #    }
    /// # }
    /// ```
    ///
    /// # Note
    ///
    /// For more details visit: [`ink_env::own_code_hash`]
    pub fn own_code_hash(self) -> Result<E::Hash> {
        ink_env::own_code_hash::<E>()
    }
}
