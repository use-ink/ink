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

use crate::reflect::{
    ContractEnv,
    DispatchError,
};
use core::{
    any::TypeId,
    convert::Infallible,
    mem::ManuallyDrop,
};
use ink_env::{
    Environment,
    ReturnFlags,
};
use ink_primitives::{
    Key,
    KeyPtr,
};
use ink_storage::{
    alloc,
    alloc::ContractPhase,
    traits::{
        pull_spread_root,
        push_spread_root,
        SpreadAllocate,
        SpreadLayout,
    },
};

/// The root key of the ink! smart contract.
///
/// # Note
///
/// - This is the key where storage allocation, pushing and pulling is rooted
///   using the `SpreadLayout` and `SpreadAllocate` traits primarily.
/// - This trait is automatically implemented by the ink! codegen.
/// - The existence of this trait allows to customize the root key in future
///   versions of ink! if needed.
pub trait ContractRootKey {
    const ROOT_KEY: Key;
}

/// Returns `Ok` if the caller did not transfer additional value to the callee.
///
/// # Errors
///
/// If the caller did send some amount of transferred value to the callee.
#[inline]
pub fn deny_payment<E>() -> Result<(), DispatchError>
where
    E: Environment,
{
    let transferred = ink_env::transferred_balance::<E>();
    if transferred != <E as Environment>::Balance::from(0_u32) {
        return Err(DispatchError::PaidUnpayableMessage)
    }
    Ok(())
}

/// Configuration for execution of ink! constructor.
#[derive(Debug, Copy, Clone)]
pub struct ExecuteConstructorConfig {
    /// Yields `true` if the dynamic storage allocator has been enabled.
    ///
    /// # Note
    ///
    /// Authors can enable it via `#[ink::contract(dynamic_storage_allocator = true)]`.
    pub dynamic_storage_alloc: bool,
}

/// Executes the given ink! constructor.
///
/// # Note
///
/// The closure is supposed to already contain all the arguments that the real
/// constructor message requires and forwards them.
#[inline]
pub fn execute_constructor<Contract, F, R>(
    config: ExecuteConstructorConfig,
    f: F,
) -> Result<(), DispatchError>
where
    Contract: SpreadLayout + ContractRootKey,
    F: FnOnce() -> R,
    <private::Seal<R> as ConstructorReturnType<Contract>>::ReturnValue: scale::Encode,
    private::Seal<R>: ConstructorReturnType<Contract>,
{
    if config.dynamic_storage_alloc {
        alloc::initialize(ContractPhase::Deploy);
    }
    let result = ManuallyDrop::new(private::Seal(f()));
    match result.as_result() {
        Ok(contract) => {
            // Constructor is infallible or is fallible but succeeded.
            //
            // This requires us to sync back the changes of the contract storage.
            let root_key = <Contract as ContractRootKey>::ROOT_KEY;
            push_spread_root::<Contract>(contract, &root_key);
            if config.dynamic_storage_alloc {
                alloc::finalize();
            }
            Ok(())
        }
        Err(_) => {
            // Constructor is fallible and failed.
            //
            // We need to revert the state of the transaction.
            ink_env::return_value::<
                <private::Seal<R> as ConstructorReturnType<Contract>>::ReturnValue,
            >(
                ReturnFlags::default().set_reverted(true),
                result.return_value(),
            )
        }
    }
}

/// Initializes the ink! contract using the given initialization routine.
///
/// # Note
///
/// - This uses `SpreadAllocate` trait in order to default initialize the
///   ink! smart contract before calling the initialization routine.
/// - This either returns `Contract` or `Result<Contract, E>` depending
///   on the return type `R` of the initializer closure `F`.
///   If `R` is `()` then `Contract` is returned and if `R` is any type of
///   `Result<(), E>` then `Result<Contract, E>` is returned.
///   Other return types for `F` than the ones listed above are not allowed.
#[inline]
pub fn initialize_contract<Contract, F, R>(
    initializer: F,
) -> <R as InitializerReturnType<Contract>>::Wrapped
where
    Contract: ContractRootKey + SpreadAllocate,
    F: FnOnce(&mut Contract) -> R,
    R: InitializerReturnType<Contract>,
{
    let mut key_ptr = KeyPtr::from(<Contract as ContractRootKey>::ROOT_KEY);
    let mut instance = <Contract as SpreadAllocate>::allocate_spread(&mut key_ptr);
    let result = initializer(&mut instance);
    result.into_wrapped(instance)
}

mod private {
    /// Seals the implementation of `ContractInitializerReturnType`.
    pub trait Sealed {}
    impl Sealed for () {}
    impl<T, E> Sealed for Result<T, E> {}
    /// A thin-wrapper type that automatically seals its inner type.
    ///
    /// Since it is private it can only be used from within this crate.
    /// We need this type in order to properly seal the `ConstructorReturnType`
    /// trait from unwanted external trait implementations.
    #[repr(transparent)]
    pub struct Seal<T>(pub T);
    impl<T> Sealed for Seal<T> {}
}

/// Guards against using invalid contract initializer types.
///
/// # Note
///
/// Currently the only allowed types are `()` and `Result<(), E>`
/// where `E` is some unspecified error type.
/// If the contract initializer returns `Result::Err` the utility
/// method that is used to initialize an ink! smart contract will
/// revert the state of the contract instantiation.
pub trait ConstructorReturnType<C>: private::Sealed {
    /// Is `true` if `Self` is `Result<C, E>`.
    const IS_RESULT: bool = false;

    /// The error type of the constructor return type.
    ///
    /// # Note
    ///
    /// For infallible constructors this is `core::convert::Infallible`.
    type Error;

    /// The type of the return value of the constructor.
    ///
    /// # Note
    ///
    /// For infallible constructors this is `()` whereas for fallible
    /// constructors this is the actual return value. Since we only ever
    /// return a value in case of `Result::Err` the `Result::Ok` value
    /// does not matter.
    type ReturnValue;

    /// Converts the return value into a `Result` instance.
    ///
    /// # Note
    ///
    /// For infallible constructor returns this always yields `Ok`.
    fn as_result(&self) -> Result<&C, &Self::Error>;

    /// Returns the actual return value of the constructor.
    ///
    /// # Note
    ///
    /// For infallible constructor returns this always yields `()`
    /// and is basically ignored since this does not get called
    /// if the constructor did not fail.
    fn return_value(&self) -> &Self::ReturnValue;
}

impl<C> ConstructorReturnType<C> for private::Seal<C> {
    type Error = Infallible;
    type ReturnValue = ();

    #[inline(always)]
    fn as_result(&self) -> Result<&C, &Self::Error> {
        Ok(&self.0)
    }

    #[inline(always)]
    fn return_value(&self) -> &Self::ReturnValue {
        &()
    }
}

impl<C, E> ConstructorReturnType<C> for private::Seal<Result<C, E>> {
    const IS_RESULT: bool = true;
    type Error = E;
    type ReturnValue = Result<C, E>;

    #[inline(always)]
    fn as_result(&self) -> Result<&C, &Self::Error> {
        self.0.as_ref()
    }

    #[inline(always)]
    fn return_value(&self) -> &Self::ReturnValue {
        &self.0
    }
}

/// Trait used to convert return types of contract initializer routines.
///
/// Only `()` and `Result<(), E>` are allowed contract initializer return types.
/// For `WrapReturnType<C>` where `C` is the contract type the trait converts
/// `()` into `C` and `Result<(), E>` into `Result<C, E>`.
pub trait InitializerReturnType<C>: private::Sealed {
    type Wrapped;

    /// Performs the type conversion of the initialization routine return type.
    fn into_wrapped(self, wrapped: C) -> Self::Wrapped;
}

impl<C> InitializerReturnType<C> for () {
    type Wrapped = C;

    #[inline]
    fn into_wrapped(self, wrapped: C) -> C {
        wrapped
    }
}
impl<C, E> InitializerReturnType<C> for Result<(), E> {
    type Wrapped = Result<C, E>;

    #[inline]
    fn into_wrapped(self, wrapped: C) -> Self::Wrapped {
        self.map(|_| wrapped)
    }
}

/// Configuration for execution of ink! messages.
#[derive(Debug, Copy, Clone)]
pub struct ExecuteMessageConfig {
    /// Yields `true` if the ink! message accepts payment.
    ///
    /// # Note
    ///
    /// If no ink! message within the same ink! smart contract
    /// is payable then this flag will be `true` since the check
    /// then is moved before the message dispatch as an optimization.
    pub payable: bool,
    /// Yields `true` if the ink! message might mutate contract storage.
    ///
    /// # Note
    ///
    /// This is usually true for `&mut self` ink! messages.
    pub mutates: bool,
    /// Yields `true` if the dynamic storage allocator has been enabled.
    ///
    /// # Note
    ///
    /// Authors can enable it via `#[ink::contract(dynamic_storage_allocator = true)]`.
    pub dynamic_storage_alloc: bool,
}

/// Executes the given `&mut self` message closure.
///
/// # Note
///
/// The closure is supposed to already contain all the arguments that the real
/// message requires and forwards them.
#[inline]
pub fn execute_message<Storage, Output, F>(
    config: ExecuteMessageConfig,
    f: F,
) -> Result<(), DispatchError>
where
    Storage: SpreadLayout + ContractEnv,
    Output: scale::Encode + 'static,
    F: FnOnce(&mut Storage) -> Output,
{
    if !config.payable {
        deny_payment::<<Storage as ContractEnv>::Env>()?;
    }
    if config.dynamic_storage_alloc {
        alloc::initialize(ContractPhase::Call);
    }
    let root_key = Key::from([0x00; 32]);
    let mut storage = ManuallyDrop::new(pull_spread_root::<Storage>(&root_key));
    let result = f(&mut storage);
    if config.mutates {
        push_spread_root::<Storage>(&storage, &root_key);
    }
    if config.dynamic_storage_alloc {
        alloc::finalize();
    }
    if TypeId::of::<Output>() != TypeId::of::<()>() {
        // We include a check for `is_result_type!(Output)` despite the fact that this
        // is indirectly covered by `is_result_err!(&result)` because the Rust compiler
        // will have more opportunities to optimize the whole conditional away. This is
        // due to the fact that `is_result_type!` relies on constant information whereas
        // is_result_err!` requires `&self`.
        let revert_state = is_result_type!(Output) && is_result_err!(&result);
        ink_env::return_value::<Output>(
            ReturnFlags::default().set_reverted(revert_state),
            &result,
        )
    }
    Ok(())
}
