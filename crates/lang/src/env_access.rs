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
    RentParams,
    Result,
};
use ink_primitives::Key;

use crate::ChainExtensionInstance;

/// The environment of the compiled ink! smart contract.
pub trait ContractEnv {
    /// The environment type.
    type Env: ::ink_env::Environment;
}

/// Simplifies interaction with the host environment via `self`.
///
/// # Note
///
/// This is generally implemented for storage structs that include
/// their environment in order to allow the different dispatch functions
/// to use it for returning the contract's output.
pub trait Env {
    /// The access wrapper.
    type EnvAccess;

    /// Accesses the environment with predefined environmental types.
    fn env(self) -> Self::EnvAccess;
}

/// Simplifies interaction with the host environment via `Self`.
///
/// # Note
///
/// This is generally implemented for storage structs that include
/// their environment in order to allow the different dispatch functions
/// to use it for returning the contract's output.
pub trait StaticEnv {
    /// The access wrapper.
    type EnvAccess;

    /// Accesses the environment with predefined environmental types.
    fn env() -> Self::EnvAccess;
}

/// A typed accessor to the environment.
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
    /// use ink_prelude;
    ///
    /// #[ink(message, payable)]
    /// pub fn call_me(&self) {
    ///     let caller = self.env().caller();
    ///     let message = ink_prelude::format!("got a call from {:?}", caller);
    ///     ink_env::debug_println(&message);
    /// }
    /// ```
    ///
    /// # Note
    ///
    /// For more details visit: [`ink_env::caller`]
    pub fn caller(self) -> T::AccountId {
        ink_env::caller::<T>().expect("couldn't decode caller")
    }

    /// Returns the transferred balance for the contract execution.
    ///
    /// # Example
    ///
    /// ```
    /// use ink_prelude;
    ///
    /// /// Allows funding the contract. Prints a debug message with the transferred balance.
    /// #[ink(message, payable)]
    /// pub fn fund(&self) {
    ///     let caller = self.env().caller();
    ///     let value = self.env().transferred_balance();
    ///     let message = ink_prelude::format!("thanks for the funding of {:?} from {:?}", value, caller);
    ///     ink_env::debug_println(&message);
    /// }
    /// ```
    ///
    /// # Note
    ///
    /// For more details visit: [`ink_env::transferred_balance`]
    pub fn transferred_balance(self) -> T::Balance {
        ink_env::transferred_balance::<T>().expect("couldn't decode transferred balance")
    }

    /// Returns the price for the specified amount of gas.
    ///
    /// # Example
    ///
    /// ```
    /// let price = self.env().weight_to_fee(13);
    /// ```
    ///
    /// # Note
    ///
    /// For more details visit: [`ink_env::weight_to_fee`]
    pub fn weight_to_fee(self, gas: u64) -> T::Balance {
        ink_env::weight_to_fee::<T>(gas).expect("couldn't decode weight fee")
    }

    /// Returns the amount of gas left for the contract execution.
    ///
    /// # Example
    ///
    /// ```
    /// let gas_left = self.env().gas_left();
    /// ```
    ///
    /// # Note
    ///
    /// For more details visit: [`ink_env::gas_left`]
    pub fn gas_left(self) -> T::Balance {
        ink_env::gas_left::<T>().expect("couldn't decode gas left")
    }

    /// Returns the timestamp of the current block.
    ///
    /// # Example
    ///
    /// ```
    /// let now = self.env().block_timestamp();
    /// ```
    ///
    /// # Note
    ///
    /// For more details visit: [`ink_env::block_timestamp`]
    pub fn block_timestamp(self) -> T::Timestamp {
        ink_env::block_timestamp::<T>().expect("couldn't decode block time stamp")
    }

    /// Returns the account ID of the executed contract.
    ///
    /// # Example
    ///
    /// ```
    /// use ink_prelude;
    ///
    /// /// Prints a debug message with the called contract's account id.
    /// #[ink(message, payable)]
    /// pub fn call_me(&self) {
    ///     let account_id = self.env().account_id();
    ///     let message = ink_prelude::format!("contract's account id is {:?}", account_id);
    ///     ink_env::debug_println(&message);
    /// }
    /// ```
    ///
    /// # Note
    ///
    /// For more details visit: [`ink_env::account_id`]
    pub fn account_id(self) -> T::AccountId {
        ink_env::account_id::<T>().expect("couldn't decode contract account ID")
    }

    /// Returns the balance of the executed contract.
    ///
    /// # Example
    ///
    /// ```
    /// let contracts_balance = self.env().balance();
    /// ```
    ///
    /// # Note
    ///
    /// For more details visit: [`ink_env::balance`]
    pub fn balance(self) -> T::Balance {
        ink_env::balance::<T>().expect("couldn't decode contract balance")
    }

    /// Returns the current rent allowance for the executed contract.
    ///
    /// # Example
    ///
    /// ```
    /// let allowance = self.env().rent_allowance();
    /// ```
    ///
    /// # Note
    ///
    /// For more details visit: [`ink_env::rent_allowance`]
    pub fn rent_allowance(self) -> T::Balance {
        ink_env::rent_allowance::<T>().expect("couldn't decode contract rent allowance")
    }

    /// Sets the rent allowance of the executed contract to the new value.
    ///
    /// # Example
    ///
    /// ```
    /// self.env().set_rent_allowance(self.env().balance / 2);
    /// ```
    ///
    /// # Note
    ///
    /// For more details visit: [`ink_env::set_rent_allowance`]
    pub fn set_rent_allowance(self, new_value: T::Balance) {
        ink_env::set_rent_allowance::<T>(new_value)
    }

    /// Returns information needed for rent calculations.
    ///
    /// # Note
    ///
    /// For more details visit: [`ink_env::RentParams`]
    pub fn rent_params(self) -> RentParams<T> {
        ink_env::rent_params::<T>().expect("couldn't decode contract rent params")
    }

    /// Returns the current block number.
    ///
    /// # Example
    ///
    /// ```
    /// let block_number = self.env().block_number();
    /// ```
    ///
    /// # Note
    ///
    /// For more details visit: [`ink_env::block_number`]
    pub fn block_number(self) -> T::BlockNumber {
        ink_env::block_number::<T>().expect("couldn't decode block number")
    }

    /// Returns the minimum balance that is required for creating an account.
    ///
    /// # Example
    ///
    /// ```
    /// let minimum_balance = self.env().minimum_balance();
    /// ```
    ///
    /// # Note
    ///
    /// For more details visit: [`ink_env::minimum_balance`]
    pub fn minimum_balance(self) -> T::Balance {
        ink_env::minimum_balance::<T>().expect("couldn't decode minimum account balance")
    }

    /// Returns the tombstone deposit for the contracts chain.
    ///
    /// # Example
    ///
    /// ```
    /// let tombstone_deposit = self.env().tombstone_deposit();
    /// ```
    ///
    /// # Note
    ///
    /// For more details visit: [`ink_env::tombstone_deposit`]
    pub fn tombstone_deposit(self) -> T::Balance {
        ink_env::tombstone_deposit::<T>().expect("couldn't decode tombstone deposits")
    }

    /// Instantiates another contract.
    ///
    /// # Example
    ///
    /// ```
    /// # use ::ink_env::{
    /// #     Environment,
    /// #     DefaultEnvironment,
    /// #     call::{build_create, Selector, ExecutionInput, FromAccountId}
    /// # };
    /// # type Hash = <DefaultEnvironment as Environment>::Hash;
    /// # type AccountId = <DefaultEnvironment as Environment>::AccountId;
    /// # type Salt = &'static [u8];
    /// # struct MyContract;
    /// # impl FromAccountId<DefaultEnvironment> for MyContract {
    /// #     fn from_account_id(account_id: AccountId) -> Self { Self }
    /// # }
    /// /// Instantiates another contract.
    /// #[ink(message)]
    /// pub fn instantiate_contract(&self) -> AccountId {
    ///     let create_params = build_create::<DefaultEnvironment, MyContract>()
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
    /// # use ::ink_env::{
    /// #     Environment,
    /// #     DefaultEnvironment,
    /// #     call::{build_call, Selector, ExecutionInput}
    /// # };
    /// # type AccountId = <DefaultEnvironment as Environment>::AccountId;
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
    /// # use ::ink_env::{
    /// #     Environment,
    /// #     DefaultEnvironment,
    /// #     call::{build_call, Selector, ExecutionInput}
    /// # };
    /// # type AccountId = <DefaultEnvironment as Environment>::AccountId;
    /// /// Evaluates a contract message and fetches the result.
    /// #[ink(message)]
    /// pub fn invoke_contract(&self) -> i32 {
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
    ///         .returns::<i32>()
    ///         .params();
    ///     self.env().eval_contract(&call_params).expect("call invocation must succeed")
    /// }
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

    /// Restores a smart contract from its tombstone state.
    ///
    /// # Example
    ///
    /// ```
    /// # type Hash = <DefaultEnvironment as Environment>::Hash;
    /// # use ::ink_env::{
    /// #     Environment,
    /// #     DefaultEnvironment,
    /// # };
    /// # type AccountId = <DefaultEnvironment as Environment>::AccountId;
    /// /// Simple resurrection of a contract.
    /// #[ink(message)]
    /// pub fn resurrect(&self, contract: AccountId) {
    ///     self.env().restore_contract(contract,
    ///         Hash::from([0x42; 32]),
    ///         1000,
    ///         &[]
    ///     )
    /// }
    /// ```
    ///
    /// # Note
    ///
    /// For more details visit: [`ink_env::restore_contract`]
    pub fn restore_contract(
        self,
        account_id: T::AccountId,
        code_hash: T::Hash,
        rent_allowance: T::Balance,
        filtered_keys: &[Key],
    ) {
        ink_env::restore_contract::<T>(
            account_id,
            code_hash,
            rent_allowance,
            filtered_keys,
        )
    }

    /// Terminates the existence of a contract without creating a tombstone.
    ///
    /// # Example
    ///
    /// ```
    /// /// Terminates with the caller as beneficiary.
    /// #[ink(message)]
    /// pub fn terminate_me(&mut self) {
    ///     self.env().terminate_contract(self.env().caller());
    /// }
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
    /// /// Transfers the token amount ten to the caller.
    /// #[ink(message)]
    /// pub fn give_me_ten(&mut self) {
    ///     let value: Balance = 10;
    ///     self.env().transfer(self.env().caller(), value).expect("transfer failed");
    /// }
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
    /// #[ink(message)]
    /// pub fn random_bool(&self) -> bool {
    ///     let additional_randomness = &[];
    ///     let (hash, _block_number) = self.env().random(additional_randomness);
    ///     hash.as_ref()[0] != 0
    /// }
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
}
