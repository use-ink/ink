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

use ink::codegen::TraitCallBuilder;
use ink_env::{
    call::{
        utils::{
            ReturnType,
            Set,
            Unset,
        },
        Call,
        CallBuilder,
        CreateBuilder,
        ExecutionInput,
        FromAccountId,
    },
    Environment,
};
use scale::Encode;

/// The type returned from `ContractRef` constructors, partially initialized with the execution
/// input arguments.
pub type CreateBuilderPartial<E, ContractRef, Args, R> = CreateBuilder<
    E,
    ContractRef,
    Unset<<E as Environment>::Hash>,
    Unset<u64>,
    Unset<<E as Environment>::Balance>,
    Set<ExecutionInput<Args>>,
    Unset<ink_env::call::state::Salt>,
    Set<ReturnType<R>>,
>;

/// Get the encoded constructor arguments from the partially initialized `CreateBuilder`
pub fn constructor_exec_input<E, ContractRef, Args: Encode, R>(
    builder: CreateBuilderPartial<E, ContractRef, Args, R>,
) -> Vec<u8>
where
    E: Environment,
{
    // set all the other properties to default values, we only require the `exec_input`.
    builder
        .endowment(0u32.into())
        .code_hash(ink_primitives::Clear::CLEAR_HASH)
        .salt_bytes(Vec::new())
        .params()
        .exec_input()
        .encode()
}

/// Captures the encoded input for an `ink!` message call, together with the account id of the
/// contract being called.
#[derive(Debug, Clone)]
pub struct Message<E: Environment, RetType> {
    account_id: E::AccountId,
    exec_input: Vec<u8>,
    _return_type: std::marker::PhantomData<RetType>,
}

impl<E, RetType> Message<E, RetType>
where
    E: Environment,
{
    /// The account id of the contract being called to invoke the message.
    pub fn account_id(&self) -> &E::AccountId {
        &self.account_id
    }

    /// The encoded message data, comprised of the selector and the message arguments.
    pub fn exec_input(&self) -> &[u8] {
        &self.exec_input
    }
}

/// Convenience method for building messages for the default environment.
///
/// # Note
///
/// This is hardcoded to [`ink_env::DefaultEnvironment`] so the user does not have to specify this
/// generic parameter, which currently is hardcoded in the E2E testing suite.
pub fn build_message<Ref>(
    account_id: <ink_env::DefaultEnvironment as Environment>::AccountId,
) -> MessageBuilder<ink_env::DefaultEnvironment, Ref>
where
    Ref: TraitCallBuilder + FromAccountId<ink_env::DefaultEnvironment>,
{
    MessageBuilder::from_account_id(account_id)
}

/// Build messages using a contract ref.
pub struct MessageBuilder<E: Environment, Ref> {
    account_id: E::AccountId,
    contract_ref: Ref,
}

impl<E, Ref> MessageBuilder<E, Ref>
where
    E: Environment,
    Ref: TraitCallBuilder + FromAccountId<E>,
{
    /// Create a new [`MessageBuilder`] to invoke a message on the given contract.
    pub fn from_account_id(account_id: E::AccountId) -> Self {
        let contract_ref = <Ref as FromAccountId<E>>::from_account_id(account_id.clone());
        Self {
            account_id,
            contract_ref,
        }
    }

    /// Build an encoded call for a message from a [`CallBuilder`] instance returned from a
    /// contract ref method.
    ///
    /// This utilizes the generated message inherent methods on the contract ref implementation,
    /// which returns a [`CallBuilder`] initialized with the selector and message arguments.
    ///
    /// # Example
    /// ```
    /// # #[ink::contract]
    /// # pub mod my_contract {
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
    /// #         #[ink(message)]
    /// #         pub fn message(&self) {}
    /// #     }
    /// # }
    /// #
    /// # fn message_builder_doc_test() {
    /// #     use my_contract::MyContractRef;
    /// #     let contract_acc_id = ink_primitives::AccountId::from([0x00; 32]);
    /// ink_e2e::MessageBuilder::<ink::env::DefaultEnvironment, MyContractRef>::from_account_id(
    ///     contract_acc_id,
    /// )
    /// .call(|contract| contract.message());
    /// # }
    /// ```

    pub fn call<F, Args, RetType>(mut self, mut message: F) -> Message<E, RetType>
    where
        F: FnMut(
            &mut <Ref as TraitCallBuilder>::Builder,
        ) -> CallBuilder<
            E,
            Set<Call<E>>,
            Set<ExecutionInput<Args>>,
            Set<ReturnType<RetType>>,
        >,
        Args: scale::Encode,
        RetType: scale::Decode,
    {
        let call_builder = <Ref as TraitCallBuilder>::call_mut(&mut self.contract_ref);
        let builder = message(call_builder);
        let exec_input = builder.params().exec_input().encode();
        Message {
            account_id: self.account_id.clone(),
            exec_input,
            _return_type: Default::default(),
        }
    }
}
