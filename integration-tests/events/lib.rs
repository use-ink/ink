#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
pub mod events {
    #[ink(storage)]
    pub struct Events {
        value: bool,
    }

    impl Events {
        /// Creates a new events smart contract initialized with the given value.
        #[ink(constructor)]
        pub fn new(init_value: bool) -> Self {
            Self { value: init_value }
        }

        /// Flips the current value of the boolean.
        #[ink(message)]
        pub fn flip(&mut self) {
            self.value = !self.value;
            self.env()
                .emit_event(event_def::Flipped { value: self.value })
        }

        /// Emit an event with a 32 byte topic.
        #[ink(message)]
        pub fn emit_32_byte_topic_event(&mut self, maybe_hash: Option<[u8; 32]>) {
            self.env().emit_event(event_def::ThirtyTwoByteTopics {
                hash: [0x42; 32],
                maybe_hash,
            })
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use scale::Decode as _;

        #[ink::test]
        fn it_works() {
            let mut events = Events::new(false);
            events.flip();

            let emitted_events = ink::env::test::recorded_events().collect::<Vec<_>>();
            assert_eq!(1, emitted_events.len());
            let event = &emitted_events[0];

            let decoded_event = <event_def::Flipped>::decode(&mut &event.data[..])
                .expect("encountered invalid contract event data buffer");
            assert_eq!(decoded_event.value, true);
        }

        #[ink::test]
        fn option_topic_some_has_topic() {
            let mut events = Events::new(false);
            events.emit_32_byte_topic_event(Some([0xAA; 32]));

            let emitted_events = ink::env::test::recorded_events().collect::<Vec<_>>();
            assert_eq!(1, emitted_events.len());
            let event = &emitted_events[0];

            assert_eq!(event.topics.len(), 3);
            let signature_topic =
                <event_def::ThirtyTwoByteTopics as ink::env::Event>::SIGNATURE_TOPIC
                    .map(|topic| topic.to_vec());
            assert_eq!(Some(&event.topics[0]), signature_topic.as_ref());
            assert_eq!(event.topics[1], [0x42; 32]);
            assert_eq!(
                event.topics[2], [0xAA; 32],
                "option topic should be published"
            );
        }

        #[ink::test]
        fn option_topic_none_missing_topic() {
            let mut events = Events::new(false);
            events.emit_32_byte_topic_event(None);

            let emitted_events = ink::env::test::recorded_events().collect::<Vec<_>>();
            assert_eq!(1, emitted_events.len());
            let event = &emitted_events[0];

            assert_eq!(
                event.topics.len(),
                2,
                "option topic should *not* be published"
            );
        }
    }

    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        use super::*;

        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        #[ink_e2e::test]
        async fn emits_shared_event(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // given
            let init_value = false;
            let constructor = EventsRef::new(init_value);
            let contract = client
                .instantiate("events", &ink_e2e::alice(), constructor, 0, None)
                .await
                .expect("instantiate failed");
            let mut call = contract.call::<Events>();

            // when
            let flip = call.flip();
            let flip_res = client
                .call(&ink_e2e::bob(), &flip, 0, None)
                .await
                .expect("flip failed");

            let contract_events = flip_res.contract_emitted_events();
            assert_eq!(1, contract_events.len());
            let flipped: event_def::Flipped =
                scale::Decode::decode(&mut &contract_events[0].data[..])
                    .expect("encountered invalid contract event data buffer");
            assert_eq!(!init_value, flipped.value);

            Ok(())
        }
    }
}
