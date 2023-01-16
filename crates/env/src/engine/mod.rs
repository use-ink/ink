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
    call::{
        utils::InstantiateResult,
        FromAccountId,
    },
    Error as EnvError,
    Error,
    Result as EnvResult,
};
use cfg_if::cfg_if;
use ink_primitives::{
    ConstructorResult,
    LangError,
};

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

// We only use this function when 1) compiling to Wasm 2) compiling for tests.
#[cfg_attr(all(feature = "std", not(test)), allow(dead_code))]
pub(crate) fn decode_instantiate_result<I, E, R, ContractStorage, ContractRef>(
    instantiate_result: EnvResult<()>,
    out_address: &mut I,
    out_return_value: &mut I,
) -> EnvResult<
    ConstructorResult<<R as InstantiateResult<ContractStorage>>::Output<ContractRef>>,
>
where
    I: scale::Input,
    E: crate::Environment,
    R: InstantiateResult<ContractStorage>,
    ContractRef: FromAccountId<E>,
{
    match instantiate_result {
        Ok(()) => {
            let account_id = scale::Decode::decode(out_address)?;
            let contract_ref =
                <ContractRef as FromAccountId<E>>::from_account_id(account_id);
            let output = <R as InstantiateResult<ContractStorage>>::ok(contract_ref);
            Ok(Ok(output))
        }
        Err(EnvError::CalleeReverted) => {
            let constructor_result_variant = out_return_value.read_byte()?;
            match constructor_result_variant {
                // 0 == `ConstructorResult::Ok` variant
                0 => {
                    if <R as InstantiateResult<ContractStorage>>::IS_RESULT {
                        let result_variant = out_return_value.read_byte()?;
                        match result_variant {
                            // 0 == `Ok` variant
                            0 => panic!("The callee reverted, but did not encode an error in the output buffer."),
                            // 1 == `Err` variant
                            1 => {
                                let contract_err = <<R as InstantiateResult<ContractStorage>>::Error
                                    as scale::Decode>::decode(out_return_value)?;
                                let err = <R as InstantiateResult<ContractStorage>>::err(contract_err);
                                Ok(Ok(err))
                            }
                            _ => Err(Error::Decode("Invalid inner constructor Result encoding, expected 0 or 1 as the first byte".into()))
                        }
                    } else {
                        panic!("The callee reverted, but did not encode an error in the output buffer.")
                    }
                }
                // 1 == `ConstructorResult::Err` variant
                1 => {
                    let lang_err = <LangError as scale::Decode>::decode(out_return_value)?;
                    Ok(Err(lang_err))
                }
                _ => Err(Error::Decode("Invalid outer constructor Result encoding, expected 0 or 1 as the first byte".into()))
            }
        }
        Err(actual_error) => Err(actual_error.into()),
    }
}

#[cfg(test)]
mod decode_instantiate_result_tests {
    use super::*;
    use crate::{
        Environment,
        Error,
    };
    use scale::Encode;

    // The `Result` type used to represent the programmer defined contract output.
    type ContractResult<T, E> = Result<T, E>;

    #[derive(scale::Encode, scale::Decode)]
    struct ContractError(String);

    struct TestContractRef<E: Environment>(<E as Environment>::AccountId);

    impl<E: Environment> FromAccountId<E> for TestContractRef<E> {
        fn from_account_id(account_id: <E as Environment>::AccountId) -> Self {
            Self(account_id)
        }
    }

    fn encode_and_decode_return_value(
        return_value: ConstructorResult<Result<(), ContractError>>,
    ) -> EnvResult<
        ConstructorResult<
            Result<TestContractRef<crate::DefaultEnvironment>, ContractError>,
        >,
    > {
        let out_address = Vec::new();
        let encoded_return_value = return_value.encode();
        decode_return_value_fallible(
            &mut &out_address[..],
            &mut &encoded_return_value[..],
        )
    }

    fn decode_return_value_fallible<I: scale::Input>(
        out_address: &mut I,
        out_return_value: &mut I,
    ) -> EnvResult<
        ConstructorResult<
            Result<TestContractRef<crate::DefaultEnvironment>, ContractError>,
        >,
    > {
        decode_instantiate_result::<
            I,
            crate::DefaultEnvironment,
            Result<(), ContractError>,
            (),
            TestContractRef<crate::DefaultEnvironment>,
        >(Err(Error::CalleeReverted), out_address, out_return_value)
    }

    #[test]
    #[should_panic(
        expected = "The callee reverted, but did not encode an error in the output buffer."
    )]
    fn revert_branch_rejects_valid_output_buffer_with_success_case() {
        let return_value = ConstructorResult::Ok(ContractResult::Ok(()));

        let _decoded_result = encode_and_decode_return_value(return_value);
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
        let out_address = Vec::new();
        let invalid_encoded_return_value = vec![69];

        let decoded_result = decode_return_value_fallible(
            &mut &out_address[..],
            &mut &invalid_encoded_return_value[..],
        );

        assert!(matches!(decoded_result, Err(crate::Error::Decode(_))))
    }
}
