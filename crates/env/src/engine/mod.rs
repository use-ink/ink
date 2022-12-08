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

use crate::backend::{
    EnvBackend,
    TypedEnvBackend,
};
use cfg_if::cfg_if;

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

use crate::{
    Environment,
    Result as EnvResult,
};
use ink_primitives::ConstructorResult;

pub(crate) fn decode_fallible_constructor_reverted_return_value<I, E, ContractError>(
    out_return_value: &mut I,
) -> EnvResult<ConstructorResult<Result<E::AccountId, ContractError>>>
where
    I: scale::Input,
    E: Environment,
    ContractError: scale::Decode,
{
    let decoding_result =
        <ConstructorResult<Result<(), ContractError>> as scale::Decode>::decode(
            out_return_value,
        );

    match decoding_result {
        Ok(constructor_result) => {
            let contract_result = constructor_result.expect(
                "If dispatch had failed, we shouldn't have been able to decode \
                             the nested `Result`.",
            );

            let contract_error = contract_result.expect_err(
                "Since the contract reverted, we only expect an `Error` from the constructor. \
                             Otherwise we would be in the `AccountId` branch.");

            Ok(Ok(Err(contract_error)))
        }
        Err(_) => {
            // If we hit this branch it likely means dispatch failed, but we need to
            // check the buffer again to confirm.
            let out = <ink_primitives::ConstructorResult<()> as scale::Decode>::decode(
                out_return_value,
            )?;

            let lang_error = out.expect_err(
                "If dispatch had succeeded, we would either be in the `AccountId` branch \
                             or we would've been able to decode into a nested `Result` earlier."
            );

            Ok(Err(lang_error))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use scale::Encode;

    #[derive(scale::Encode, scale::Decode)]
    struct ContractError(String);

    #[test]
    fn fallible_constructor_reverted_contract_error() {
        let return_value = Ok(Err(ContractError("Constructor error".to_owned())));
        let encoded_return_value =
            <ConstructorResult<Result<(), ContractError>> as Encode>::encode(
                &return_value,
            );

        let decoded_result = decode_fallible_constructor_reverted_return_value::<
            _,
            crate::DefaultEnvironment,
            ContractError,
        >(&mut &encoded_return_value[..]);

        assert!(matches!(
            decoded_result,
            Ok(Ok(Err(ContractError(
                _
            ))))
        ))
    }

    #[test]
    fn fallible_constructor_reverted_lang_error() {
        let return_value = Err(ink_primitives::LangError::CouldNotReadInput);
        let encoded_return_value =
            <ConstructorResult<Result<(), ContractError>> as Encode>::encode(
                &return_value,
            );

        let decoded_result = decode_fallible_constructor_reverted_return_value::<
            _,
            crate::DefaultEnvironment,
            ContractError,
        >(&mut &encoded_return_value[..]);

        assert!(matches!(
            decoded_result,
            Ok(Err(ink_primitives::LangError::CouldNotReadInput))
        ))
    }
}
