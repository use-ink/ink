// Copyright 2018-2021 Parity Technologies (UK) Ltd.
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
    engine::experimental_off_chain::impls::TopicsBuilder,
    topics::TopicsBuilderBackend,
    Result,
};

#[test]
fn topics_builder() -> Result<()> {
    crate::test::run_test::<crate::DefaultEnvironment, _>(|_| {
        // given
        let mut builder = TopicsBuilder::default();

        // when
        TopicsBuilderBackend::<crate::DefaultEnvironment>::push_topic(&mut builder, &13);
        TopicsBuilderBackend::<crate::DefaultEnvironment>::push_topic(&mut builder, &17);

        // then
        assert_eq!(builder.topics.len(), 2);

        let topics_len_compact = &scale::Compact(2u32);
        let topics_len_encoded = scale::Encode::encode(&topics_len_compact);
        let output = TopicsBuilderBackend::<crate::DefaultEnvironment>::output(builder);
        #[rustfmt::skip]
        let expected = vec![topics_len_encoded[0], 13, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 17, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        assert_eq!(output, expected);

        Ok(())
    })
}
