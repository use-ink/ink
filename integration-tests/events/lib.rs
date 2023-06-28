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
            assert!(decoded_event.value);
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
        fn option_topic_none_encoded_as_0() {
            let mut events = Events::new(false);
            events.emit_32_byte_topic_event(None);

            let emitted_events = ink::env::test::recorded_events().collect::<Vec<_>>();
            assert_eq!(1, emitted_events.len());
            let event = &emitted_events[0];

            let signature_topic =
                <event_def::ThirtyTwoByteTopics as ink::env::Event>::SIGNATURE_TOPIC
                    .map(|topic| topic.to_vec())
                    .unwrap();

            let expected_topics = vec![
                signature_topic,
                [0x42; 32].to_vec(),
                [0x00; 32].to_vec(), // None is encoded as 0x00
            ];
            assert_eq!(expected_topics, event.topics);
        }
    }

    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        use super::*;
        use ink_e2e::H256;

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

            let contract_events = flip_res.contract_emitted_events()?;

            // then
            assert_eq!(1, contract_events.len());
            let contract_event = &contract_events[0];
            let flipped: event_def::Flipped =
                scale::Decode::decode(&mut &contract_event.event.data[..])
                    .expect("encountered invalid contract event data buffer");
            assert_eq!(!init_value, flipped.value);

            let signature_topic =
                <event_def::Flipped as ink::env::Event>::SIGNATURE_TOPIC
                    .map(H256::from)
                    .unwrap();

            let expected_topics = vec![signature_topic];
            assert_eq!(expected_topics, contract_event.topics);

            Ok(())
        }

        #[ink_e2e::test]
        async fn emits_event_with_option_topic_none(
            mut client: ink_e2e::Client<C, E>,
        ) -> E2EResult<()> {
            // given
            let init_value = false;
            let constructor = EventsRef::new(init_value);
            let contract = client
                .instantiate("events", &ink_e2e::alice(), constructor, 0, None)
                .await
                .expect("instantiate failed");
            let mut call = contract.call::<Events>();

            // when
            let call = call.emit_32_byte_topic_event(None);
            let call_res = client
                .call(&ink_e2e::bob(), &call, 0, None)
                .await
                .expect("emit_32_byte_topic_event failed");

            let contract_events = call_res.contract_emitted_events()?;

            // then
            assert_eq!(1, contract_events.len());
            let contract_event = &contract_events[0];
            let event: event_def::ThirtyTwoByteTopics =
                scale::Decode::decode(&mut &contract_event.event.data[..])
                    .expect("encountered invalid contract event data buffer");
            assert!(event.maybe_hash.is_none());

            let signature_topic =
                <event_def::ThirtyTwoByteTopics as ink::env::Event>::SIGNATURE_TOPIC
                    .map(H256::from)
                    .unwrap();

            let expected_topics = vec![
                signature_topic,
                [0x42; 32].into(),
                [0x00; 32].into(), // None is encoded as 0x00
            ];
            assert_eq!(expected_topics, contract_event.topics);

            Ok(())
        }
    }
}