//! This is a reference implementation with one approach to decoding
//! (capturing emitted) events within unit and e2e tests.


#![cfg_attr(not(feature = "std"), no_std)]

#[ink::contract]
mod testing_event_decode {

    #[ink(storage)]
    pub struct TestingEventDecode {}

    #[ink(event)]
    pub struct FlipEvent {
        #[ink(topic)]
        flipper: AccountId,
        #[ink(topic)]
        value: bool,
    }

    #[ink(event)]
    pub struct FlopEvent {
        #[ink(topic)]
        flipper: AccountId,
        #[ink(topic)]
        flopper: AccountId,
        #[ink(topic)]
        value: bool,
    }

    impl TestingEventDecode {

        #[ink(constructor)]
        pub fn new() -> Self {
            Self {}
        }

        /// This message emits two events for test verification.
        #[ink(message)]
        pub fn flipflop(&mut self) {

            let caller = self.env().caller();
            let contract = self.env().account_id();

            // Emit FlipEvent.
            self.env().emit_event(FlipEvent {
                flipper: caller,
                value: true,
            });

            // Emit FlopEvent.
            self.env().emit_event(FlopEvent {
                flipper: caller,
                flopper: contract,
                value: false,
            });
        }
    }

    /// This is one way to capture and check event emission in ink unit tests.
    #[cfg(test)]
    mod tests {

        use super::*;
        use ink::env::test::EmittedEvent;

        type Event = <TestingEventDecode as ::ink::reflect::ContractEventBase>::Type;

        fn decode_events(emitted_events: Vec<EmittedEvent>) -> Vec<Event> {
            emitted_events
                .into_iter()
                .map(|event| {
                    <Event as scale::Decode>::decode(&mut &event.data[..])
                        .expect("Invalid event data")
                })
                .collect()
        }

        #[ink::test]
        fn unittest_event_emission_capture_decode() {

            let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);

            let mut contract = TestingEventDecode::new();
            contract.flipflop();

            // Decode event.
            let emitted_events = ink::env::test::recorded_events().collect::<Vec<_>>();
            let decoded_events = decode_events(emitted_events);

            let mut gotflip = false;
            let mut gotflop = false;

            // Check events were emitted by the `flipflop` function.
            for event in &decoded_events {
                match event {
                    Event::FlipEvent(FlipEvent { flipper, value }) => {
                        assert_eq!(*value, true, "unexpected FlipEvent.value");
                        assert_eq!(*flipper, accounts.bob, "unexpected FlipEvent.flipper");
                        gotflip = true;
                    },
                    Event::FlopEvent(FlopEvent { flipper, flopper, value }) => {
                        assert_eq!(*value, false, "unexpected FlopEvent.value");
                        assert_eq!(*flipper, accounts.bob, "unexpected FlopEvent.flipper");
                        assert_eq!(*flopper, accounts.alice, "unexpected FlopEvent.flopper");
                        gotflop = true;
                    },
                };
            }
            assert!(gotflip, "expected flip event not captured");
            assert!(gotflop, "expected flop event not captured");
        }
    }

    /// This is one way to capture and check event emission in ink e2e tests.
    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {

        use super::*;
        use ink_e2e::build_message;

        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        /// Verify that we can capture emitted events and decode to check their fields.
        #[ink_e2e::test]
        async fn e2etest_event_emission_capture_decode(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {

            let bob_account = ink_e2e::account_id(ink_e2e::AccountKeyring::Bob);
            let constructor = TestingEventDecodeRef::new();
            let contract_account_id = client
                .instantiate(
                    "testing_event_decode",
                    &ink_e2e::bob(),
                    constructor,
                    0,
                    None,
                )
                .await
                .expect("instantiate failed")
                .account_id;

            // Check that FlipEvent was successfully emitted.
            let flipflop_msg =
                build_message::<TestingEventDecodeRef>(contract_account_id.clone())
                    .call(|contract| contract.flipflop());
            let flipflop_result = client
                .call(&ink_e2e::bob(), flipflop_msg, 0, None)
                .await
                .expect("flipflop failed");

            // Filter the events.
            let contract_emitted_event = flipflop_result
                .events
                .iter()
                .find(|event| {
                    event
                        .as_ref()
                        .expect("bad event")
                        .event_metadata()
                        .event()
                        == "ContractEmitted"
                        &&
                        String::from_utf8_lossy(
                            event.as_ref().expect("bad event").bytes()).to_string()
                       .contains("TestingEventDecode::FlipEvent")
                })
                .expect("Expected flip event")
                .unwrap();

            // Decode the expected event type.
            let event = contract_emitted_event.field_bytes();
            let decode_event = <FlipEvent as scale::Decode>::decode(&mut &event[34..])
                .expect("invalid data");

            let FlipEvent { flipper, value } = decode_event;

            // Check that FlopEvent was successully emitted.
            assert_eq!(value, true, "unexpected FlipEvent.value");
            assert_eq!(flipper, bob_account, "unexpected FlipEvent.flipper");

            // Build flipflop message.
            let flipflop_msg =
                build_message::<TestingEventDecodeRef>(contract_account_id.clone())
                    .call(|contract| contract.flipflop());
            let flipflop_result = client
                .call(&ink_e2e::bob(), flipflop_msg, 0, None)
                .await
                .expect("flipflop failed");

            // Filter the events.
            let contract_emitted_event = flipflop_result
                .events
                .iter()
                .find(|event| {
                    event
                        .as_ref()
                        .expect("bad event")
                        .event_metadata()
                        .event()
                        == "ContractEmitted"
                        &&
                        String::from_utf8_lossy(
                            event.as_ref().expect("bad event").bytes()).to_string()
                       .contains("TestingEventDecode::FlopEvent")
                })
                .expect("Expected flop event")
                .unwrap();

            // Decode the expected event type.
            let event = contract_emitted_event.field_bytes();
            let decode_event = <FlopEvent as scale::Decode>::decode(&mut &event[35..])
                .expect("invalid data");

            let FlopEvent { flipper, flopper, value } = decode_event;

            // Check event emitted by the `flip` function.
            assert_eq!(value, false, "unexpected FlopEvent.value");
            assert_eq!(flipper, bob_account, "unexpected FlopEvent.flipper");
            assert_eq!(flopper, contract_account_id, "unexpected FlopEvent.flopper");

            Ok(())
        }
    }
}
