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

use cfg_if::cfg_if;
use ink_primitives::ConstructorResult;
use pallet_revive_uapi::ReturnErrorCode;

use crate::{
    Error,
    Result as EnvResult,
    backend::{
        EnvBackend,
        TypedEnvBackend,
    },
    call::{
        ConstructorReturnType,
        FromAddr,
        utils::{
            ConstructorError,
            DecodeConstructorError,
        },
    },
};

/// Convert a slice into an array reference.
///
/// Creates an array reference of size `$len` pointing to `$offset` within `$arr`.
///
/// # Panics
///
/// - The selected range is out of bounds given the supplied slice
/// - Integer overflow on `$offset + $len`
macro_rules! array_mut_ref {
    ($arr:expr, $offset:expr, $len:expr) => {{
        {
            fn as_array<T>(slice: &mut [T]) -> &mut [T; $len] {
                slice.try_into().unwrap()
            }
            let offset: usize = $offset;
            let slice = &mut $arr[offset..offset.checked_add($len).unwrap()];
            as_array(slice)
        }
    }};
}

pub trait OnInstance: EnvBackend + TypedEnvBackend {
    fn on_instance<F, R>(f: F) -> R
    where
        F: FnOnce(&mut Self) -> R;
}

cfg_if! {
    if #[cfg(not(feature = "std"))] {
        mod on_chain;
        pub use self::on_chain::EnvInstance;
    } else {
        pub mod off_chain;
        pub use self::off_chain::EnvInstance;
    }
}

// We only use this function when 1) compiling for PolkaVM 2) compiling for tests.
#[cfg_attr(all(feature = "std", not(test)), allow(dead_code))]
pub(crate) fn decode_instantiate_result<I, ContractRef, R, Abi>(
    instantiate_result: EnvResult<()>,
    out_address: &mut I,
    out_return_value: &[u8],
) -> EnvResult<ConstructorResult<<R as ConstructorReturnType<ContractRef, Abi>>::Output>>
where
    I: scale::Input,
    ContractRef: FromAddr,
    R: ConstructorReturnType<ContractRef, Abi>,
{
    match instantiate_result {
        Ok(()) => {
            let addr = scale::Decode::decode(out_address)?;
            let contract_ref = <ContractRef as FromAddr>::from_addr(addr);
            let output = <R as ConstructorReturnType<ContractRef, Abi>>::ok(contract_ref);
            Ok(Ok(output))
        }
        Err(Error::ReturnError(ReturnErrorCode::CalleeReverted)) => {
            decode_instantiate_err::<ContractRef, R, Abi>(out_return_value)
        }
        Err(actual_error) => Err(actual_error),
    }
}

#[cfg_attr(all(feature = "std", not(test)), allow(dead_code))]
fn decode_instantiate_err<ContractRef, R, Abi>(
    out_return_value: &[u8],
) -> EnvResult<ConstructorResult<<R as ConstructorReturnType<ContractRef, Abi>>::Output>>
where
    ContractRef: FromAddr,
    R: ConstructorReturnType<ContractRef, Abi>,
{
    let decoded_error =
        <<R as ConstructorReturnType<ContractRef, Abi>>::Error as DecodeConstructorError<
            Abi,
        >>::decode_error_output(out_return_value);
    match decoded_error {
        ConstructorError::Contract(contract_err) => {
            let error_output =
                <R as ConstructorReturnType<ContractRef, Abi>>::err(contract_err);
            match error_output {
                Some(output) => Ok(Ok(output)),
                None => {
                    // No user defined error variant, and successful error decoding
                    // (i.e. to unit), so we maintain the `CalleeReverted`
                    // environmental error.
                    Err(Error::ReturnError(ReturnErrorCode::CalleeReverted))
                }
            }
        }
        ConstructorError::Lang(lang_err) => Ok(Err(lang_err)),
        ConstructorError::Env(env_err) => Err(env_err),
    }
}

#[cfg(test)]
mod decode_instantiate_result_tests {
    use super::*;
    use crate::DefaultEnvironment;
    use ink_primitives::Address;
    use scale::Encode;

    // The `Result` type used to represent the programmer defined contract output.
    type ContractResult<T, E> = Result<T, E>;

    #[derive(Debug, PartialEq, scale::Encode, scale::Decode)]
    struct ContractError(String);

    // The `allow(dead_code)` is for the `AccountId` in the struct.
    #[allow(dead_code)]
    #[derive(Debug, PartialEq)]
    struct TestContractRef(Address);

    impl crate::ContractEnv for TestContractRef {
        type Env = DefaultEnvironment;
    }

    impl FromAddr for TestContractRef {
        fn from_addr(addr: Address) -> Self {
            Self(addr)
        }
    }

    fn encode_and_decode_return_value(
        return_value: ConstructorResult<Result<(), ContractError>>,
    ) -> EnvResult<ConstructorResult<Result<TestContractRef, ContractError>>> {
        let out_address = Vec::new();
        let encoded_return_value = return_value.encode();
        decode_return_value_fallible(&mut &out_address[..], &encoded_return_value[..])
    }

    fn decode_return_value_fallible<I: scale::Input>(
        out_address: &mut I,
        out_return_value: &[u8],
    ) -> EnvResult<ConstructorResult<Result<TestContractRef, ContractError>>> {
        decode_instantiate_result::<
            I,
            TestContractRef,
            Result<TestContractRef, ContractError>,
            ink_primitives::abi::Ink,
        >(
            Err(ReturnErrorCode::CalleeReverted.into()),
            out_address,
            out_return_value,
        )
    }

    #[test]
    fn revert_branch_rejects_valid_output_buffer_with_success_case() {
        let return_value = ConstructorResult::Ok(ContractResult::Ok(()));

        let decoded_result = encode_and_decode_return_value(return_value);

        let expected_error: EnvResult<
            ConstructorResult<Result<TestContractRef, ContractError>>,
        > = EnvResult::Err(crate::Error::Decode(
            "The callee reverted, but did not encode an error in the output buffer."
                .into(),
        ));
        assert_eq!(decoded_result, expected_error);
    }

    #[test]
    fn successful_dispatch_with_error_from_contract_constructor() {
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
        let out_address = Vec::new();
        let invalid_encoded_return_value = [69];

        let decoded_result = decode_return_value_fallible(
            &mut &out_address[..],
            &invalid_encoded_return_value[..],
        );

        assert!(matches!(decoded_result, Err(crate::Error::Decode(_))))
    }
}
