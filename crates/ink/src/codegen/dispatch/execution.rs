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

use ink_env::DispatchError;

/// Returns `Ok` if the caller did not transfer additional value to the callee.
///
/// # Errors
///
/// If the caller did send some amount of transferred value to the callee.
#[inline]
pub fn deny_payment() -> Result<(), DispatchError> {
    let transferred = ink_env::transferred_value();
    if !transferred.is_zero() {
        return Err(DispatchError::PaidUnpayableMessage)
    }
    Ok(())
}
