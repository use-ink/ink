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
        utils::ReturnType,
        CallParams,
        CreateParams,
    },
    hash::{
        CryptoHash,
        HashOutput,
    },
    Environment,
    Error,
    Result,
};
use ink_eth_compatibility::ECDSAPublicKey;

/// The API behind the `self.env()` and `Self::env()` syntax in ink!.
///
/// This allows ink! messages to make use of the environment efficiently
/// and user friendly while also maintaining access invariants.
#[derive(Copy, Clone)]
pub struct EnvAccess<'a, T> {
    /// Tricks the Rust compiler into thinking that we use `T`.
    marker: PhantomData<fn() -> &'a T>,
}

impl<'a, T> Default for EnvAccess<'a, T> {
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

impl<'a, T> EnvAccess<'a, T>
where
    T: Environment,
    <T as Environment>::ChainExtension: ChainExtensionInstance,
{
    /// Allows to call one of the available defined chain extension methods.
    pub fn extension(
        self,
    ) -> <<T as Environment>::ChainExtension as ChainExtensionInstance>::Instance {
        <<T as Environment>::ChainExtension as ChainExtensionInstance>::instantiate()
    }
}

impl<'a, T> EnvAccess<'a, T>
where
    T: Environment,
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
    pub fn caller(self) -> T::AccountId {
        ink_env::caller::<T>()
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
    pub fn transferred_value(self) -> T::Balance {
        ink_env::transferred_value::<T>()
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
    pub fn weight_to_fee(self, gas: u64) -> T::Balance {
        ink_env::weight_to_fee::<T>(gas)
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
        ink_env::gas_left::<T>()
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
    pub fn block_timestamp(self) -> T::Timestamp {
        ink_env::block_timestamp::<T>()
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
    pub fn account_id(self) -> T::AccountId {
        ink_env::account_id::<T>()
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
    pub fn balance(self) -> T::Balance {
        ink_env::balance::<T>()
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
    pub fn block_number(self) -> T::BlockNumber {
        ink_env::block_number::<T>()
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
    pub fn minimum_balance(self) -> T::Balance {
        ink_env::minimum_balance::<T>()
    }

    /// Instantiates another contract.
    ///
    /// # Example
    ///
    /// ```
    /// # use ink_lang as ink;
    /// # #[ink::contract]
    /// # pub mod my_contract {
    /// #
    /// # // We do this since we can't have two `ink::contract` calls in the
    /// # // same module (conflicting method definitions in the generated code).
    /// # //
    /// # // As long as we have something that implements the `ContractRef` trait
    /// # // it doesn't really matter where it comes from (for this example).
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
        params: &CreateParams<T, Args, Salt, C>,
    ) -> Result<T::AccountId>
    where
        Args: scale::Encode,
        Salt: AsRef<[u8]>,
    {
        ink_env::instantiate_contract::<T, Args, Salt, C>(params)
    }

    /// Invokes a contract message without fetching its result.
    ///
    /// # Example
    ///
    /// ```
    /// # use ink_lang as ink;
    /// # #[ink::contract]
    /// # pub mod my_contract {
    /// use ink_env::{
    ///     DefaultEnvironment,
    ///     call::{build_call, Selector, ExecutionInput}
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
    /// /// Invokes another contract message without fetching the result.
    /// #[ink(message)]
    /// pub fn invoke_contract(&self) {
    ///     let call_params = build_call::<DefaultEnvironment>()
    ///         .callee(AccountId::from([0x42; 32]))
    ///         .gas_limit(5000)
    ///         .transferred_value(10)
    ///         .exec_input(
    ///             ExecutionInput::new(Selector::new([0xCA, 0xFE, 0xBA, 0xBE]))
    ///                 .push_arg(42)
    ///                 .push_arg(true)
    ///                 .push_arg(&[0x10u8; 32])
    ///         )
    ///         .returns::<()>()
    ///         .params();
    ///     self.env().invoke_contract(&call_params).expect("call invocation must succeed");
    /// }
    /// #
    /// #     }
    /// # }
    /// ```
    ///
    /// # Note
    ///
    /// For more details visit: [`ink_env::invoke_contract`]
    pub fn invoke_contract<Args>(self, params: &CallParams<T, Args, ()>) -> Result<()>
    where
        Args: scale::Encode,
    {
        ink_env::invoke_contract::<T, Args>(params)
    }

    /// Evaluates a contract message and returns its result.
    ///
    /// # Example
    ///
    /// ```
    /// # use ink_lang as ink;
    /// # #[ink::contract]
    /// # pub mod my_contract {
    /// use ink_env::{
    ///     DefaultEnvironment,
    ///     call::{build_call, Selector, ExecutionInput, utils::ReturnType}
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
    /// /// Evaluates a contract message and fetches the result.
    /// #[ink(message)]
    /// pub fn evaluate_contract(&self) -> i32 {
    ///     let call_params = build_call::<DefaultEnvironment>()
    ///         .callee(AccountId::from([0x42; 32]))
    ///         .gas_limit(5000)
    ///         .transferred_value(10)
    ///         .exec_input(
    ///             ExecutionInput::new(Selector::new([0xCA, 0xFE, 0xBA, 0xBE]))
    ///                 .push_arg(42)
    ///                 .push_arg(true)
    ///                 .push_arg(&[0x10u8; 32])
    ///         )
    ///         .returns::<ReturnType<i32>>()
    ///         .params();
    ///     self.env().eval_contract(&call_params).expect("call invocation must succeed")
    /// }
    /// #
    /// #     }
    /// # }
    /// ```
    ///
    /// # Note
    ///
    /// For more details visit: [`ink_env::eval_contract`]
    pub fn eval_contract<Args, R>(
        self,
        params: &CallParams<T, Args, ReturnType<R>>,
    ) -> Result<R>
    where
        Args: scale::Encode,
        R: scale::Decode,
    {
        ink_env::eval_contract::<T, Args, R>(params)
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
    pub fn terminate_contract(self, beneficiary: T::AccountId) -> ! {
        ink_env::terminate_contract::<T>(beneficiary)
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
    pub fn transfer(self, destination: T::AccountId, value: T::Balance) -> Result<()> {
        ink_env::transfer::<T>(destination, value)
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
    pub fn random(self, subject: &[u8]) -> (T::Hash, T::BlockNumber) {
        ink_env::random::<T>(subject).expect("couldn't decode randomized hash")
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
    ///         161, 234, 203,  74, 147, 96,  51, 212,   5, 174, 231,   9, 142,  48, 137, 201,
    ///         162, 118, 192,  67, 239, 16,  71, 216, 125,  86, 167, 139,  70,   7,  86, 241,
    ///          33,  87, 154, 251,  81, 29, 160,   4, 176, 239,  88, 211, 244, 232, 232,  52,
    ///         211, 234, 100, 115, 230, 47,  80,  44, 152, 166,  62,  50,   8,  13,  86, 175,
    ///          28,
    ///     ];
    ///     const message_hash: [u8; 32] = [
    ///         162, 28, 244, 179, 96, 76, 244, 178, 188,  83, 230, 248, 143, 106,  77, 117,
    ///         239, 95, 244, 171, 65, 95,  62, 153, 174, 166, 182,  28, 130,  73, 196, 208
    ///     ];
    ///     let EXPECTED_COMPRESSED_PUBLIC_KEY: [u8; 33] = [
    ///           2, 121, 190, 102, 126, 249, 220, 187, 172, 85, 160,  98, 149, 206, 135, 11,
    ///           7,   2, 155, 252, 219,  45, 206,  40, 217, 89, 242, 129,  91,  22, 248, 23,
    ///         152,
    ///     ].into();
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
    ) -> Result<ECDSAPublicKey> {
        let mut output = [0; 33];
        ink_env::ecdsa_recover(signature, message_hash, &mut output)
            .map(|_| output.into())
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
    pub fn is_contract(self, account_id: &T::AccountId) -> bool {
        ink_env::is_contract::<T>(account_id)
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
        ink_env::caller_is_origin::<T>()
    }
}
