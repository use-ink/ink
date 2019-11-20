// Copyright 2018-2019 Parity Technologies (UK) Ltd.
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

//! # For Developers
//!
//! For the implementation of the `Env` trait and all of its sub traits
//! we can ignore the `buffer` parameters that we use extensively in the
//! `SrmlEnv` implementation since here we already have our buffers in the
//! contract's memory.

use crate::{
    env2::{
        call::{
            CallParams,
            CreateParams,
            ReturnType,
        },
        error::CallError,
        property,
        test::{
            typed_encoded::AlreadyInitialized,
            Account,
            AccountKind,
            CallContractRecord,
            ContractAccount,
            CreateContractRecord,
            EmitEventRecord,
            InvokeRuntimeRecord,
            Record,
            RestoreContractRecord,
            TestEnvInstance,
            TypedEncoded,
        },
        utils::{
            EnlargeTo,
            Reset,
        },
        Env,
        EnvTypes,
        Error,
        GetProperty,
        Result,
        SetProperty,
        Topics,
    },
    storage::Key,
};
use core::{
    cell::{
        Ref,
        RefCell,
        RefMut,
    },
    marker::PhantomData,
};

thread_local! {
    /// The single thread-local test environment instance.
    ///
    /// # Note
    ///
    /// This needs to be thread local since tests are run
    /// in parallel by default which may lead to data races otherwise.
    pub static INSTANCE: RefCell<TestEnvInstance> = {
        RefCell::new(TestEnvInstance::default())
    };
}

/// Accessor to the test environment instance.
///
/// This acts as the real test environment to the outside.
pub struct TestEnv<T> {
    /// Needed to trick Rust into allowing `T`.
    marker: PhantomData<fn() -> T>,
}

#[cfg(feature = "ink-generate-abi")]
impl<E> type_metadata::HasTypeId for TestEnv<E>
where
    E: type_metadata::Metadata,
{
    fn type_id() -> type_metadata::TypeId {
        type_metadata::TypeIdCustom::new(
            "TestEnv",
            type_metadata::Namespace::from_module_path(module_path!())
                .expect("namespace from module path cannot fail"),
            vec![E::meta_type()],
        )
        .into()
    }
}

#[cfg(feature = "ink-generate-abi")]
impl<E> type_metadata::HasTypeDef for TestEnv<E> {
    fn type_def() -> type_metadata::TypeDef {
        type_metadata::TypeDefStruct::new(vec![]).into()
    }
}

impl<T> TestEnv<T>
where
    T: EnvTypes,
    // The below constraints are satisfied for the default SRLM type configuration.
    <T as EnvTypes>::AccountId: From<[u8; 32]>,
    <T as EnvTypes>::Balance: From<u128>,
    <T as EnvTypes>::BlockNumber: From<u64>,
    <T as EnvTypes>::Moment: From<u64>,
{
    /// Tries to initialize the test environment with proper state.
    ///
    /// The test environment must only be used after it has been initialized.
    /// Otherwise accessing its state will certainly crash the execution due
    /// to type mismatches.
    ///
    /// # Errors
    ///
    /// If the test environment has already been initialized.
    pub fn try_initialize() -> core::result::Result<(), AlreadyInitialized> {
        INSTANCE.with(|instance| instance.borrow_mut().try_initialize::<T>())
    }
}

impl<T> EnvTypes for TestEnv<T>
where
    T: EnvTypes,
{
    /// The type of an address.
    type AccountId = T::AccountId;
    /// The type of balances.
    type Balance = T::Balance;
    /// The type of hash.
    type Hash = T::Hash;
    /// The type of timestamps.
    type Moment = T::Moment;
    /// The type of block number.
    type BlockNumber = T::BlockNumber;
    /// The type of a call into the runtime
    type Call = T::Call;
}

impl<T> SetProperty<property::RentAllowance<Self>> for TestEnv<T>
where
    T: EnvTypes,
{
    fn set_property<O>(
        _buffer: &mut O,
        value: &<property::RentAllowance<Self> as property::WriteProperty>::Out,
    ) where
        O: scale::Output + AsRef<[u8]> + Reset,
    {
        INSTANCE.with(|instance| {
            let mut account = RefMut::map(instance.borrow_mut(), |instance| {
                let account_id = &instance.exec_context.callee;
                instance
                    .accounts
                    .get_mut(account_id)
                    .expect("callee is required to be in the accounts DB")
            });
            account.rent_allowance.assign(value);
        })
    }
}

macro_rules! impl_get_property_for {
    ( $prop_name:ident => |$name:ident| $body:tt $($rest:tt)* ) => {
        impl<T> GetProperty<property::$prop_name<Self>> for TestEnv<T>
        where
            T: EnvTypes,
        {
            fn get_property<I>(
                _buffer: &mut I,
            ) -> <property::$prop_name<Self> as property::ReadProperty>::In
            where
                I: AsMut<[u8]> + EnlargeTo,
            {
                INSTANCE.with(|$name| $body)
            }
        }

        impl_get_property_for!($($rest)*);
    };
    () => {};
}

impl_get_property_for! {
    Input => |instance| {
        instance.borrow().exec_context.call_data.clone()
    }

    Caller => |instance| {
        instance.borrow().exec_context.caller.to_origin()
    }

    TransferredBalance => |instance| {
        instance.borrow().exec_context.transferred_balance.to_origin()
    }

    GasPrice => |instance| {
        instance.borrow().state.gas_price.to_origin()
    }

    GasLeft => |instance| {
        instance.borrow().exec_context.gas_left.to_origin()
    }

    NowInMs => |instance| {
        instance.borrow().block.now_in_ms.to_origin()
    }

    Address => |instance| {
        instance.borrow().exec_context.callee.to_origin()
    }

    Balance => |instance| {
        let account = Ref::map(instance.borrow(), |instance| {
            let account_id = &instance.exec_context.callee;
            instance.accounts
                .get(account_id)
                .expect("callee is required to be in the accounts DB")
        });
        account.balance.to_origin()
    }

    RentAllowance => |instance| {
        let account = Ref::map(instance.borrow(), |instance| {
            let account_id = &instance.exec_context.callee;
            instance.accounts
                .get(account_id)
                .expect("callee is required to be in the accounts DB")
        });
        account.rent_allowance.to_origin()
    }

    BlockNumber => |instance| {
        instance.borrow().block.number.to_origin()
    }

    MinimumBalance => |instance| {
        instance.borrow().state.minimum_balance.to_origin()
    }
}

impl<T> Env for TestEnv<T>
where
    T: EnvTypes,
    <T as EnvTypes>::Hash: From<[u8; 32]>,
    <T as EnvTypes>::AccountId: From<[u8; 32]>,
{
    fn get_contract_storage<I, R>(_buffer: &mut I, key: Key) -> Result<R>
    where
        I: AsMut<[u8]> + EnlargeTo,
        R: scale::Decode,
    {
        INSTANCE.with(|instance| {
            let storage = Ref::map(instance.borrow(), |instance| {
                let account_id = &instance.exec_context.callee;
                &instance
                    .accounts
                    .get(account_id)
                    .expect("callee is required to be in the accounts DB")
                    .contract()
                    .expect("callee must refer to a contract account")
                    .storage
            });
            let encoded = storage
                .read(key)
                .map(|entry| entry.data())
                .ok_or(Error::InvalidStorageRead)?;
            Ok(scale::Decode::decode(&mut &encoded[..])
                .map_err(|_| Error::InvalidStorageRead)?)
        })
    }

    fn set_contract_storage<O, V>(buffer: &mut O, key: Key, value: &V)
    where
        O: scale::Output + AsRef<[u8]> + Reset,
        V: scale::Encode,
    {
        INSTANCE.with(|instance| {
            let mut storage = RefMut::map(instance.borrow_mut(), |instance| {
                let account_id = &instance.exec_context.callee;
                &mut instance
                    .accounts
                    .get_mut(account_id)
                    .expect("callee is required to be in the accounts DB")
                    .contract_mut()
                    .expect("callee must refer to a contract account")
                    .storage
            });
            buffer.reset();
            value.encode_to(buffer);
            storage.write(key, buffer.as_ref());
        })
    }

    fn clear_contract_storage(key: Key) {
        INSTANCE.with(|instance| {
            let mut storage = RefMut::map(instance.borrow_mut(), |instance| {
                let account_id = &instance.exec_context.callee;
                &mut instance
                    .accounts
                    .get_mut(account_id)
                    .expect("callee is required to be in the accounts DB")
                    .contract_mut()
                    .expect("callee must refer to a contract account")
                    .storage
            });
            storage.clear(key);
        })
    }

    fn invoke_contract<O>(_buffer: &mut O, call_data: &CallParams<Self, ()>) -> Result<()>
    where
        O: scale::Output + AsRef<[u8]> + Reset,
    {
        // With the off-chain test environment we have no means to invoke
        // a remote contract on the chain since there is no chain.
        // What we do instead is to log the call and do nothing.
        // The codegen of ink! shall instead call the contract directly
        // and log a call through an invokation of this API.
        INSTANCE.with(|instance| {
            instance
                .borrow_mut()
                .records
                .push(Record::from(CallContractRecord::new(call_data)));
            Ok(())
        })
    }

    fn eval_contract<IO, R>(
        _buffer: &mut IO,
        call_data: &CallParams<Self, ReturnType<R>>,
    ) -> Result<R>
    where
        IO: scale::Output + AsRef<[u8]> + AsMut<[u8]> + EnlargeTo + Reset,
        R: scale::Decode,
    {
        // With the off-chain test environment we have no means to invoke
        // a remote contract on the chain since there is no chain.
        // What we do instead is to log the call and do nothing.
        // The codegen of ink! shall instead call the contract directly
        // and log a call through an invokation of this API.
        //
        // # Note
        //
        // For the sake of simplicity we will return an error here since
        // we cannot generically construct an `R` out of thin air for the
        // return type. The codegen of ink! will have to handle this case.
        INSTANCE.with(|instance| {
            instance
                .borrow_mut()
                .records
                .push(Record::from(CallContractRecord::new(call_data)));
            Err(Error::Call(CallError))
        })
    }

    fn create_contract<IO, C>(
        _buffer: &mut IO,
        create_data: &CreateParams<Self, C>,
    ) -> Result<Self::AccountId>
    where
        IO: scale::Output + AsRef<[u8]> + AsMut<[u8]> + EnlargeTo + Reset,
    {
        // With the off-chain test environment we have no means to instantiate
        // a remote contract on the chain since there is no chain.
        //
        // Instead we register a new contract account into the emulated accounts
        // data base. This is not equivalent to instantiation of a new contract.
        // However, this allows to query certain stats about the newly created contract.
        INSTANCE.with(|instance| {
            // Record the contract instantiation.
            instance
                .borrow_mut()
                .records
                .push(Record::from(CreateContractRecord::new(create_data)));
            // Actual instantiation of a contract.
            let (typed_encoded, account_id) =
                instance.borrow_mut().account_id_gen.next::<T>();
            instance.borrow_mut().accounts.insert(
                typed_encoded,
                Account {
                    balance: TypedEncoded::from_origin(&0),
                    rent_allowance: TypedEncoded::from_origin(&0),
                    kind: AccountKind::Contract(ContractAccount::new(
                        TypedEncoded::from_origin(create_data.code_hash()),
                    )),
                },
            );

            Ok(account_id)
        })
    }

    fn emit_event<O, Event>(_buffer: &mut O, event: Event)
    where
        O: scale::Output + AsRef<[u8]> + Reset,
        Event: Topics<Self> + scale::Encode,
    {
        // With the off-chain test environment we have no means
        // to emit an event on the chain since there is no chain.
        // What we do instead is to log the call and do nothing.
        INSTANCE.with(|instance| {
            instance
                .borrow_mut()
                .records
                .push(Record::from(EmitEventRecord::new(event)));
        })
    }

    fn invoke_runtime<O, V>(_buffer: &mut O, call_data: &V)
    where
        O: scale::Output + AsRef<[u8]> + Reset,
        V: scale::Encode,
    {
        // With the off-chain test environment we have no means
        // to emit an event on the chain since there is no chain.
        // What we do instead is to log the call and do nothing.
        //
        // Since runtime invokations are async fire-and-forget a
        // contract cannot check for it being run anyways.
        INSTANCE.with(|instance| {
            instance
                .borrow_mut()
                .records
                .push(Record::from(InvokeRuntimeRecord::new(call_data.encode())));
        })
    }

    fn restore_to<O>(
        _buffer: &mut O,
        dest: Self::AccountId,
        code_hash: Self::Hash,
        rent_allowance: Self::Balance,
        filtered_keys: &[Key],
    ) where
        O: scale::Output + AsRef<[u8]> + Reset,
    {
        // With the off-chain test environment we have no means
        // to restore another contract on the chain since there is no chain.
        // What we do instead is to log the restoration and do nothing.
        INSTANCE.with(|instance| {
            instance.borrow_mut().records.push(Record::from(
                RestoreContractRecord::new::<T>(
                    &dest,
                    &code_hash,
                    &rent_allowance,
                    filtered_keys,
                ),
            ));
        })
    }

    /// Sets the output of the contract within the test environment.
    ///
    /// # Panics
    ///
    /// If this function is called multiple times from within a contract.
    /// Setting output multiple times is not allows in any environment
    /// so this acts as another safety guard.
    fn output<O, R>(_buffer: &mut O, return_value: &R)
    where
        O: scale::Output + AsRef<[u8]> + Reset,
        R: scale::Encode,
    {
        // With the off-chain test environment we have no means to
        // return a value from a contract since there are no other contracts
        // on the chain since there is no chain (I am not even joking ...).
        //
        // What we do instead is to log the encoded value to make it possible
        // to query for it through the test environment after the successful call.
        INSTANCE.with(|instance| {
            if instance.borrow().exec_context.output.is_some() {
                panic!(
                    "cannot set contract output multiple times within the same execution"
                )
            }
            instance.borrow_mut().exec_context.output = Some(return_value.encode());
        })
    }

    fn random<I>(_buffer: &mut I, subject: &[u8]) -> Self::Hash
    where
        I: AsMut<[u8]> + EnlargeTo,
    {
        // We return a randomized value as best effort.
        // This won't have the same guarantees as the `random_seed` functionality
        // provided by Substrate.
        // Instead we are going to return a unique randomized `Hash` in
        // dependence of the given `subject` buffer.
        // Given the same `subject` buffer we also return the same `Hash`.
        ink_utils::hash::keccak256(subject).into()
    }

    fn println(content: &str) {
        println!("{}", content)
    }
}
