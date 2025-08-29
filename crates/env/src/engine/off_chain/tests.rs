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

use crate::{
    DefaultEnvironment,
    Result,
    engine::off_chain::{
        impls::TopicsBuilder,
        test_api::set_contract_balance,
    },
    event::TopicsBuilderBackend,
};
use ink::{
    Address,
    U256,
};

#[test]
fn topics_builder() -> Result<()> {
    crate::test::run_test::<crate::DefaultEnvironment, _>(|_| {
        // given
        let mut builder = TopicsBuilder::default();

        // when
        TopicsBuilderBackend::<ink::abi::Ink>::push_topic(&mut builder, &13);
        TopicsBuilderBackend::<ink::abi::Ink>::push_topic(&mut builder, &17);

        // then
        assert_eq!(builder.topics.len(), 2);

        let output = TopicsBuilderBackend::<ink::abi::Ink>::output(builder);
        #[rustfmt::skip]
        let expected = vec![
            [13, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            [17, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
        ];
        assert_eq!(output, expected);
        Ok(())
    })
}

#[test]
fn test_set_contract_balance() -> Result<()> {
    pub use ink_engine::ext::ChainSpec;

    crate::test::run_test::<DefaultEnvironment, _>(|_| {
        let minimum_balance = ChainSpec::default().minimum_balance;

        let result = std::panic::catch_unwind(|| {
            set_contract_balance(Address::from([0x1; 20]), minimum_balance - 1)
        });

        assert!(result.is_err());

        set_contract_balance(Address::from([0x1; 20]), U256::zero());

        set_contract_balance(Address::from([0x1; 20]), minimum_balance + 1);

        Ok(())
    })
}
