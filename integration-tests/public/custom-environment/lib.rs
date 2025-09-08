#![cfg_attr(not(feature = "std"), no_std, no_main)]

use ink::env::{
    DefaultEnvironment,
    Environment,
};

/// Our custom environment diverges from the `DefaultEnvironment` in the event topics
/// limit.
#[derive(Debug, Clone, PartialEq, Eq)]
#[ink::scale_derive(TypeInfo)]
pub enum EnvironmentWithManyTopics {}

impl Environment for EnvironmentWithManyTopics {
    const NATIVE_TO_ETH_RATIO: u32 =
        <DefaultEnvironment as Environment>::NATIVE_TO_ETH_RATIO;

    type AccountId = <DefaultEnvironment as Environment>::AccountId;
    type Balance = <DefaultEnvironment as Environment>::Balance;
    type Hash = <DefaultEnvironment as Environment>::Hash;
    type BlockNumber = <DefaultEnvironment as Environment>::BlockNumber;
    type Timestamp = <DefaultEnvironment as Environment>::Timestamp;
    type EventRecord = <DefaultEnvironment as Environment>::EventRecord;
}

#[ink::contract(env = crate::EnvironmentWithManyTopics)]
mod runtime_call {
    /// Trivial contract with a single message that emits an event with many topics.
    #[ink(storage)]
    #[derive(Default)]
    pub struct Topics;

    /// An event that would be forbidden in the default environment, but is completely
    /// valid in our custom one.
    #[ink(event)]
    #[derive(Default)]
    pub struct EventWithTopics {
        #[ink(topic)]
        first_topic: Balance,
        #[ink(topic)]
        second_topic: Balance,
    }

    impl Topics {
        #[ink(constructor)]
        pub fn new() -> Self {
            Default::default()
        }

        /// Emit an event with many topics.
        #[ink(message)]
        pub fn trigger(&mut self) {
            self.env().emit_event(EventWithTopics::default());
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[ink::test]
        fn emits_event_with_many_topics() {
            let mut contract = Topics::new();
            contract.trigger();

            let emitted_events = ink::env::test::recorded_events();
            assert_eq!(emitted_events.len(), 1);

            let emitted_event = <EventWithTopics as ink::scale::Decode>::decode(
                &mut &emitted_events[0].data[..],
            );

            assert!(emitted_event.is_ok());
        }
    }

    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        use super::*;
        use ink_e2e::ContractsBackend;

        type E2EResult<T> = Result<T, Box<dyn std::error::Error>>;

        #[cfg(feature = "permissive-node")]
        #[ink_e2e::test(environment = crate::EnvironmentWithManyTopics)]
        async fn calling_custom_environment_works<Client: E2EBackend>(
            mut client: Client,
        ) -> E2EResult<()> {
            // given
            let mut constructor = TopicsRef::new();
            let contract = client
                .instantiate("custom-environment", &ink_e2e::alice(), &mut constructor)
                .submit()
                .await
                .expect("instantiate failed");
            let mut call_builder = contract.call_builder::<Topics>();

            // when
            let message = call_builder.trigger();

            let call_res = client
                .call(&ink_e2e::alice(), &message)
                .submit()
                .await
                .expect("call failed");

            // then
            assert!(call_res.contains_event("Revive", "ContractEmitted"));

            Ok(())
        }

        #[cfg(not(feature = "permissive-node"))]
        #[ink_e2e::test(environment = crate::EnvironmentWithManyTopics)]
        async fn calling_custom_environment_fails_if_incompatible_with_node<
            Client: E2EBackend,
        >(
            mut client: Client,
        ) -> E2EResult<()> {
            // given
            let mut constructor = TopicsRef::new();
            let contract = client
                .instantiate("custom-environment", &ink_e2e::alice(), &mut constructor)
                .submit()
                .await
                .expect("instantiate failed");
            let mut call_builder = contract.call_builder::<Topics>();

            let message = call_builder.trigger();

            // when
            let call_res = client.call(&ink_e2e::alice(), &message).dry_run().await;

            // then
            assert!(call_res.is_err());

            Ok(())
        }
    }
}
