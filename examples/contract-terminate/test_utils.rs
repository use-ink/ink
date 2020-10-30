// Copyright 2018-2020 Parity Technologies (UK) Ltd.
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

/// Tests if a contract terminates successfully after `self.env().terminate()`
/// has been called.
///
/// # Usage
///
/// The macro is used like this:
///
/// ```no_compile
/// let should_terminate = move || your_contract.fn_which_should_terminate();
/// ink_env::assert_contract_termination!(
///     should_terminate,
///     expected_beneficiary,
///     expected_value_transferred_to_beneficiary
/// );
/// ```
#[cfg(feature = "std")]
#[macro_export]
macro_rules! assert_contract_termination {
    (
        $should_terminate:tt,
        $beneficiary:expr,
        $balance:expr
    ) => {{
        let __act_value_any = ::std::panic::catch_unwind($should_terminate)
            .expect_err("contract did not terminate");
        let __act_encoded_input: &::std::vec::Vec<::core::primitive::u8> =
            __act_value_any.downcast_ref::<::std::vec::Vec<::core::primitive::u8>>().expect("must work");
        let __act_info: ::ink_env::test::ContractTerminationResult<Environment> =
            ::scale::Decode::decode(&mut &__act_encoded_input[..]).expect("must work");

        let __act_expected_beneficiary: <Environment as  ::ink_env::Environment>::AccountId = $beneficiary;
        assert_eq!(__act_info.beneficiary, __act_expected_beneficiary);

        let __act_expected_balance: <Environment as ::ink_env::Environment>::Balance = $balance;
        ::std::assert_eq!(__act_info.transferred, __act_expected_balance);
    }};
}
