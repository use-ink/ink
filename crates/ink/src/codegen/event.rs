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

/// Guards that an ink! event definitions respects the topic limit.
/// todo: update docs
///
/// # Usage
///
/// ```
/// // #[ink(event)]
/// pub struct ExampleEvent {}
///
/// /// The amount of the topics of the example event struct.
/// const LEN_TOPICS: usize = 3;
///
/// /// The limit for the amount of topics per ink! event definition.
/// const TOPICS_LIMIT: usize = 4;
///
/// impl ::ink::codegen::EventLenTopics for ExampleEvent {
///     type LenTopics = ::ink::codegen::EventTopics<LEN_TOPICS>;
/// }
///
/// // The below code only compiles successfully if the example ink! event
/// // definitions respects the topic limitation: it must have an amount of
/// // topics less than or equal to the topic limit.
/// const _: () = ::ink::codegen::utils::consume_type::<
///     ::ink::codegen::EventRespectsTopicLimit<ExampleEvent, TOPICS_LIMIT>,
/// >();
/// ```
pub struct EventRespectsTopicLimit<const LEN_EVENT_TOPICS: usize, const LEN_MAX_TOPICS: usize>;

impl<const LEN_EVENT_TOPICS: usize, const LEN_MAX_TOPICS: usize> EventRespectsTopicLimit<
    LEN_EVENT_TOPICS,
    LEN_MAX_TOPICS> {
    pub const ASSERT: () = assert!(LEN_EVENT_TOPICS <= LEN_MAX_TOPICS, "The event definition exceeded the maximum number of topics.");
}
