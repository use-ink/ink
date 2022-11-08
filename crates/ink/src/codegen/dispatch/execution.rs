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

use crate::reflect::{
    ContractEnv,
    DispatchError,
};
use core::mem::ManuallyDrop;
use ink_env::{
    Environment,
    ReturnFlags,
};
use ink_storage::traits::{
    Storable,
    StorageKey,
};
use scale::Encode;

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
    let transferred = ink_env::transferred_value::<E>();
    if transferred != <E as Environment>::Balance::from(0_u32) {
        return Err(DispatchError::PaidUnpayableMessage)
    }
    Ok(())
}

// /// Executes the given ink! constructor.
// ///
// /// # Note
// ///
// /// The closure is supposed to already contain all the arguments that the real
// /// constructor message requires and forwards them.
// #[inline]
// pub fn execute_constructor<Contract, F, R>(f: F) -> Result<(), DispatchError>
// where
//     Contract: Storable + StorageKey + ContractEnv,
//     F: FnOnce() -> R,
//     <private::Seal<R> as ConstructorReturnType<Contract>>::Error: Encode,
//     private::Seal<R>: ConstructorReturnType<Contract>,
// {
//     let result = ManuallyDrop::new(private::Seal(f()));
//     match result.as_result() {
//         Ok(contract) => {
//             // Constructor is infallible or is fallible but succeeded.
//             //
//             // This requires us to sync back the changes of the contract storage.
//             ink_env::set_contract_storage::<ink_primitives::Key, Contract>(
//                 &Contract::KEY,
//                 contract,
//             );
//             ink_env::return_value(ReturnFlags::default().set_reverted(false), &());
//         }
//         Err(error) => {
//             // Constructor is fallible and failed.
//             //
//             // We need to revert the state of the transaction.
//             ink_env::return_value::<
//                 <private::Seal<R> as ConstructorReturnType<Contract>>::Error,
//             >(ReturnFlags::default().set_reverted(true), error)
//         }
//     }
// }
