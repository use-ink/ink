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

use crate::{
    backend::{
        EnvBackend,
        TypedEnvBackend,
    },
    Result as EnvResult,
};
use cfg_if::cfg_if;
use ink_primitives::ConstructorResult;

pub trait OnInstance: EnvBackend + TypedEnvBackend {
    fn on_instance<F, R>(f: F) -> R
    where
        F: FnOnce(&mut Self) -> R;
}

cfg_if! {
    if #[cfg(all(not(feature = "std"), target_arch = "wasm32"))] {
        mod on_chain;
        pub use self::on_chain::EnvInstance;
    } else if #[cfg(feature = "std")] {
        pub mod off_chain;
        pub use self::off_chain::EnvInstance;
    } else {
        compile_error! {
            "ink! only support compilation as `std` or `no_std` + `wasm32-unknown`"
        }
    }
}

// The `Result` type used to represent the programmer defined contract output.
type ContractResult<T, E> = core::result::Result<T, E>;

// We only use this function when 1) compiling to Wasm 2) compiling for tests.
#[cfg_attr(all(feature = "std", not(test)), allow(dead_code))]
pub(crate) fn decode_fallible_constructor_reverted_return_value<I, E, ContractError>(
    out_return_value: &mut I,
) -> EnvResult<ConstructorResult<ContractResult<E::AccountId, ContractError>>>
where
    I: scale::Input,
    E: crate::Environment,
    ContractError: scale::Decode,
{
    let out = <ConstructorResult<Result<(), ContractError>> as scale::Decode>::decode(
        out_return_value,
    )?;

    match out {
        ConstructorResult::Ok(ContractResult::Ok(())) => {
            // Since the contract reverted we don't expect an `Ok` return value from the
            // constructor, otherwise we'd be in the `AccountId` decoding branch.
            Err(crate::Error::Decode(
                "TODO: probably shouldn't be a `Decode` error".into(),
            ))
        }
        ConstructorResult::Ok(ContractResult::Err(contract_error)) => {
            Ok(ConstructorResult::Ok(ContractResult::Err(contract_error)))
        }
        ConstructorResult::Err(lang_error) => Ok(ConstructorResult::Err(lang_error)),
    }
}

#[cfg(test)]
mod fallible_constructor_reverted_tests {
    use super::*;
    use scale::Encode;

    #[derive(scale::Encode, scale::Decode)]
    struct ContractError(String);

    fn encode_and_decode_return_value(
        return_value: ConstructorResult<Result<(), ContractError>>,
    ) -> EnvResult<ConstructorResult<Result<ink_primitives::AccountId, ContractError>>>
    {
        let encoded_return_value = return_value.encode();
        decode_return_value(&mut &encoded_return_value[..])
    }

    fn decode_return_value<I: scale::Input>(
        input: &mut I,
    ) -> EnvResult<ConstructorResult<Result<ink_primitives::AccountId, ContractError>>>
    {
        decode_fallible_constructor_reverted_return_value::<
            I,
            crate::DefaultEnvironment,
            ContractError,
        >(input)
    }

    #[test]
    fn revert_branch_rejects_valid_output_buffer_with_success_case() {
        let return_value = ConstructorResult::Ok(ContractResult::Ok(()));

        let decoded_result = encode_and_decode_return_value(return_value);

        assert!(matches!(decoded_result, Err(crate::Error::Decode(_))))
    }

    #[test]
    fn succesful_dispatch_with_error_from_contract_constructor() {
        let return_value = ConstructorResult::Ok(ContractResult::Err(ContractError(
            "Contract's constructor failed.".to_owned(),
        )));

        let decoded_result = encode_and_decode_return_value(return_value);

        assert!(matches!(
            decoded_result,
            EnvResult::Ok(ConstructorResult::Ok(ContractResult::Err(ContractError(_))))
        ))
    }

    #[test]
    fn dispatch_error_gets_decoded_correctly() {
        let return_value =
            ConstructorResult::Err(ink_primitives::LangError::CouldNotReadInput);

        let decoded_result = encode_and_decode_return_value(return_value);

        assert!(matches!(
            decoded_result,
            EnvResult::Ok(ConstructorResult::Err(
                ink_primitives::LangError::CouldNotReadInput
            ))
        ))
    }

    #[test]
    fn invalid_bytes_in_output_buffer_fail_decoding() {
        let invalid_encoded_return_value = vec![69];

        let decoded_result = decode_return_value(&mut &invalid_encoded_return_value[..]);

        assert!(matches!(decoded_result, Err(crate::Error::Decode(_))))
    }
}
