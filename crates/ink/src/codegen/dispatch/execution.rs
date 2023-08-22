// Copyright (C) Parity Technologies (UK) Ltd.
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

use crate::reflect::DispatchError;
use ink_env::Environment;

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

#[inline]
pub fn deny_reentrancy<E>() -> Result<(), DispatchError>
where
    E: Environment,
{
    let reentrance_count = ink_env::reentrance_count::<E>();

    if reentrance_count != 0 {
        return Err(DispatchError::ReentranceDenied)
    }

    Ok(())
}
