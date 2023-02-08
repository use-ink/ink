#![cfg_attr(not(feature = "std"), no_std)]

use ink::env::Environment;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum EnvironmentWithManyTopics {}

impl Environment for EnvironmentWithManyTopics {
    const MAX_EVENT_TOPICS: usize =
        <ink::env::DefaultEnvironment as Environment>::MAX_EVENT_TOPICS + 1;

    type AccountId = <ink::env::DefaultEnvironment as Environment>::AccountId;
    type Balance = <ink::env::DefaultEnvironment as Environment>::Balance;
    type Hash = <ink::env::DefaultEnvironment as Environment>::Hash;
    type BlockNumber = <ink::env::DefaultEnvironment as Environment>::BlockNumber;
    type Timestamp = <ink::env::DefaultEnvironment as Environment>::Timestamp;

    type ChainExtension = <ink::env::DefaultEnvironment as Environment>::ChainExtension;
}

#[ink::contract(env = crate::EnvironmentWithManyTopics)]
mod runtime_call {
    #[ink(storage)]
    #[derive(Default)]
    pub struct Topicer;

    #[ink(event)]
    #[derive(Default)]
    pub struct TopicedEvent {
        #[ink(topic)]
        first_topic: Balance,
        #[ink(topic)]
        second_topic: Balance,
        #[ink(topic)]
        third_topic: Balance,
        #[ink(topic)]
        fourth_topic: Balance,
        #[ink(topic)]
        fifth_topic: Balance,
    }

    impl Topicer {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {}
        }

        #[ink(message)]
        pub fn trigger(&mut self) {
            self.env().emit_event(TopicedEvent::default());
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[ink::test]
        fn emits_event_with_many_topics() {
            let mut contract = Topicer::new();
            contract.trigger();

            let emitted_events = ink::env::test::recorded_events().collect::<Vec<_>>();
            assert_eq!(1, emitted_events.len());

            let _ = <<Topicer as ink::reflect::ContractEventBase>::Type as scale::Decode>::decode(&mut &emitted_events[0].data[..])
                .expect("encountered invalid contract event data buffer");
        }
    }

    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        use super::*;
        use crate::EnvironmentWithManyTopics;
        use ink_e2e::MessageBuilder;

        type E2EResult<T> = Result<T, Box<dyn std::error::Error>>;

        #[ink_e2e::test(environment = crate::EnvironmentWithManyTopics)]
        #[ignore = "Requires that the pallet contract is configured with a schedule allowing for \
            more event topics. For example:\
            ```rust
            pub Schedule: pallet_contracts::Schedule<Runtime> = pallet_contracts::Schedule::<Runtime> {
        		limits: pallet_contracts::Limits {
		    	    event_topics: 6,
			        ..Default::default()
		        },
           		..Default::default()
	        };
            ```"]
        async fn it_works(mut client: Client<C, E>) -> E2EResult<()> {
            // given
            let constructor = TopicerRef::new();
            let contract_acc_id = client
                .instantiate(
                    "custom-environment",
                    &ink_e2e::alice(),
                    constructor,
                    0,
                    None,
                )
                .await
                .expect("instantiate failed")
                .account_id;

            // when
            let message =
                MessageBuilder::<EnvironmentWithManyTopics, TopicerRef>::from_account_id(
                    contract_acc_id,
                )
                .call(|caller| caller.trigger());

            let call_res = client
                .call(&ink_e2e::alice(), message, 0, None)
                .await
                .expect("call failed");

            // then
            call_res.contains_event("Contracts", "ContractEmitted");

            Ok(())
        }
    }
}
