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

//! Assertion helpers for E2E tests with node backend.
//!
//! These macros provide convenient assertions similar to the sandbox test framework.

/// Assert that a contract call succeeded without reverting.
///
/// This works with `CallResult` types returned from contract calls via the node backend.
///
/// # Examples
///
/// ```ignore
/// let result = client.call(&alice, &contract_call.transfer(bob_address, amount))
///     .submit()
///     .await?;
/// assert_ok!(&result);
/// ```
#[macro_export]
macro_rules! assert_ok {
    ($result:expr) => {{
        let result = $result;
        if result.dry_run.did_revert() {
            panic!(
                "Expected call to succeed but it reverted.\nError: {:?}",
                result.extract_error()
            );
        }
    }};
}

/// Assert that a contract call reverted with a specific error message.
///
/// This works with `CallResult` types returned from contract calls via the node backend.
///
/// # Examples
///
/// ```ignore
/// let result = client.call(&alice, &contract_call.transfer(bob_address, huge_amount))
///     .submit()
///     .await?;
/// assert_noop!(&result, "BalanceLow");
/// ```
#[macro_export]
macro_rules! assert_noop {
    ($result:expr, $expected_err:expr) => {{
        let result = $result;
        if !result.dry_run.did_revert() {
            panic!(
                "Expected call to revert with '{}' but it succeeded",
                $expected_err
            );
        }

        let actual_error = result.extract_error();
        if let Some(error) = actual_error {
            if !error.contains($expected_err) {
                panic!(
                    "Expected error containing '{}' but got: {}",
                    $expected_err, error
                );
            }
        } else {
            panic!(
                "Expected error containing '{}' but got no error",
                $expected_err
            );
        }
    }};
}

/// Assert that the last event from a contract call matches the expected event.
///
/// This macro extracts events from the contract result and compares the last
/// emitted event with the expected event structure by comparing encoded bytes.
///
/// # Examples
///
/// ```ignore
/// let result = client.call(&alice, &contract_call.transfer(bob_address, amount))
///     .submit()
///     .await?;
///
/// assert_last_event!(
///     &result,
///     Transfer {
///         from: contract.addr,
///         to: bob_address,
///         value: amount
///     }
/// );
/// ```
#[macro_export]
macro_rules! assert_last_event {
    ($result:expr, $expected_event:expr) => {{ $crate::assert_last_event_internal($result, $expected_event) }};
}

use crate::CallResult;
use ink_env::Environment;
use scale::{
    Decode,
    Encode,
};
use subxt::{
    blocks::ExtrinsicEvents,
    config::HashFor,
};

/// A trait for types that can expose the last contract-emitted event for assertions.
#[allow(dead_code)]
pub trait ContractEventReader {
    fn fetch_last_contract_event(self) -> Result<Vec<u8>, String>;
}

impl<'a, E, V, C, Abi> ContractEventReader
    for &'a CallResult<E, V, ExtrinsicEvents<C>, Abi>
where
    E: Environment,
    C: subxt::Config,
    HashFor<C>: Into<sp_core::H256>,
{
    fn fetch_last_contract_event(self) -> Result<Vec<u8>, String> {
        let events = self
            .contract_emitted_events()
            .map_err(|err| format!("failed to get contract events: {err:?}"))?;

        let last_event = events
            .last()
            .ok_or_else(|| "no contract events were emitted".to_string())?;

        Ok(last_event.event.data.clone())
    }
}

/// Shared implementation that decodes the last contract event and compares it against the
/// expected value.
#[allow(dead_code)]
pub fn assert_last_event_internal<R, E>(reader: R, expected_event: E)
where
    R: ContractEventReader,
    E: Decode + Encode + core::fmt::Debug,
{
    let last_event_data = reader
        .fetch_last_contract_event()
        .unwrap_or_else(|err| panic!("Contract event assertion failed: {err}"));

    let expected_bytes = expected_event.encode();

    if expected_bytes != last_event_data {
        let decoded_event =
            E::decode(&mut &last_event_data[..]).unwrap_or_else(|error| {
                panic!(
                    "failed to decode last contract event as {}: bytes={:?}, error={:?}",
                    core::any::type_name::<E>(),
                    last_event_data,
                    error
                );
            });

        panic!(
            "event mismatch!\nExpected: {:?}\nActual: {:?}",
            expected_event, decoded_event
        );
    }
}
