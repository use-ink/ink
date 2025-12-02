use super::runtime_call::{EventWithTopics, Topics, TopicsRef};
use super::EnvironmentWithManyTopics;

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

#[cfg(all(test, feature = "e2e-tests"))]
mod e2e_tests {
    use super::*;
    use ink_e2e::ContractsBackend;

    type E2EResult<T> = Result<T, Box<dyn std::error::Error>>;

    #[cfg(feature = "permissive-node")]
    #[ink_e2e::test(environment = crate::EnvironmentWithManyTopics)]
    async fn calling_custom_environment_works(mut client: Client) -> E2EResult<()> {
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
    async fn calling_custom_environment_fails_if_incompatible_with_node(
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