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

use core::marker::PhantomData;

/// Guards that an ink! event definitions respects the topic limit.
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
/// impl ::ink_lang::codegen::EventLenTopics for ExampleEvent {
///     type LenTopics = ::ink_lang::codegen::EventTopics<LEN_TOPICS>;
/// }
///
/// // The below code only compiles successfully if the example ink! event
/// // definitions respects the topic limitation: it must have an amount of
/// // topics less than or equal to the topic limit.
/// const _: () = ::ink_lang::codegen::utils::identity_type::<
///     ::ink_lang::codegen::EventRespectsTopicLimit<
///         ExampleEvent,
///         TOPICS_LIMIT,
///     >
/// >();
/// ```
pub struct EventRespectsTopicLimit<Event, const LEN_MAX_TOPICS: usize>
where
    Event: EventLenTopics,
    <Event as EventLenTopics>::LenTopics: RespectTopicLimit<LEN_MAX_TOPICS>,
{
    marker: PhantomData<fn() -> Event>,
}

/// Guards that an amount of event topics respects the event topic limit.
///
/// # Note
///
/// Implemented by `EventTopics<M>` if M is less or equal to N.
/// Automatically implemented for up to 12 event topics.
pub trait RespectTopicLimit<const N: usize> {}

/// Represents an the amount of topics for an ink! event definition.
pub struct EventTopics<const N: usize>;

macro_rules! impl_is_smaller_or_equals {
    ( $first:literal $( , $rest:literal )* $(,)? ) => {
        impl RespectTopicLimit<$first> for EventTopics<$first> {}
        $(
            impl RespectTopicLimit<$rest> for EventTopics<$first> {}
        )*

        impl_is_smaller_or_equals! { $( $rest ),* }
    };
    ( ) => {};
}
impl_is_smaller_or_equals! {
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12
}

/// Stores the number of event topics of the ink! event definition.
pub trait EventLenTopics {
    /// Type denoting the number of event topics.
    ///
    /// # Note
    ///
    /// We use an associated type instead of an associated constant here
    /// because Rust does not yet allow for generics in constant parameter
    /// position which would be required in the `EventRespectsTopicLimit`
    /// trait definition.
    /// As soon as this is possible in Rust we might change this to a constant.
    type LenTopics;
}
