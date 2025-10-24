#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::event(anonymous)]
pub struct AnonymousEvent {
    #[ink(topic)]
    pub topic: [u8; 32],
    pub field_1: u32,
}

#[ink::contract]
pub mod events {
    #[ink(storage)]
    pub struct Events {
        value: bool,
    }

    #[ink(event)]
    pub struct InlineFlipped {
        value: bool,
    }

    #[ink(
        event,
        signature_topic = "1111111111111111111111111111111111111111111111111111111111111111"
    )]
    pub struct InlineCustomFlipped {
        value: bool,
    }

    #[ink(event)]
    #[ink(anonymous)]
    pub struct InlineAnonymousEvent {
        #[ink(topic)]
        pub topic: [u8; 32],
        pub field_1: u32,
    }

    #[ink(event)]
    #[ink(anonymous)]
    pub struct InlineAnonymousEventHashedTopic {
        #[ink(topic)]
        pub topic: [u8; 64],
        pub field_1: u32,
    }

    impl Events {
        /// Creates a new events smart contract initialized with the given value.
        #[ink(constructor)]
        pub fn new(init_value: bool) -> Self {
            Self { value: init_value }
        }

        /// Flips the current value of the boolean.
        #[ink(message)]
        pub fn flip_with_foreign_event(&mut self) {
            self.value = !self.value;
            self.env()
                .emit_event(event_def::ForeignFlipped { value: self.value })
        }

        /// Flips the current value of the boolean.
        #[ink(message)]
        pub fn flip_with_inline_event(&mut self) {
            self.value = !self.value;
            self.env().emit_event(InlineFlipped { value: self.value })
        }

        /// Flips the current value of the boolean.
        #[ink(message)]
        pub fn flip_with_inline_custom_event(&mut self) {
            self.value = !self.value;
            self.env()
                .emit_event(InlineCustomFlipped { value: self.value })
        }

        /// Emit an event with a 32 byte topic.
        #[ink(message)]
        pub fn emit_32_byte_topic_event(&self, maybe_hash: Option<[u8; 32]>) {
            self.env().emit_event(event_def::ThirtyTwoByteTopics {
                hash: [0x42; 32],
                maybe_hash,
            })
        }

        /// Emit an event from a different crate.
        #[ink(message)]
        pub fn emit_event_from_a_different_crate(&self, maybe_hash: Option<[u8; 32]>) {
            self.env().emit_event(event_def2::EventDefAnotherCrate {
                hash: [0x42; 32],
                maybe_hash,
            })
        }

        /// Emit an inline and standalone anonymous events
        #[ink(message)]
        pub fn emit_anonymous_events(&self, topic: [u8; 32]) {
            self.env()
                .emit_event(InlineAnonymousEvent { topic, field_1: 42 });
            self.env()
                .emit_event(super::AnonymousEvent { topic, field_1: 42 });

            let mut twotopics = [0u8; 64];
            twotopics[..32].copy_from_slice(&topic[..32]);
            twotopics[32..].copy_from_slice(&topic[..32]);
            self.env().emit_event(InlineAnonymousEventHashedTopic {
                topic: twotopics,
                field_1: 42,
            });
        }
    }

    /// Implementing the trait from the `event_def_unused` crate includes all defined
    /// events there.
    impl event_def_unused::FlipperTrait for Events {
        #[ink(message)]
        fn flip(&mut self) {
            self.value = !self.value;
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use ink::scale::Decode as _;

        #[test]
        fn collects_specs_for_all_linked_and_used_events() {
            let event_specs = ink::collect_events();

            assert!(
                event_specs
                    .iter()
                    .any(|evt| evt.label() == &"ForeignFlipped")
            );
            assert!(
                event_specs
                    .iter()
                    .any(|evt| evt.label() == &"InlineFlipped")
            );
            assert!(
                event_specs
                    .iter()
                    .any(|evt| evt.label() == &"InlineCustomFlipped")
            );
            assert!(
                event_specs
                    .iter()
                    .any(|evt| evt.label() == &"ThirtyTwoByteTopics")
            );
            assert!(
                event_specs
                    .iter()
                    .any(|evt| evt.label() == &"EventDefAnotherCrate")
            );
            assert!(
                event_specs
                    .iter()
                    .any(|evt| evt.label() == &"AnonymousEvent")
            );
            assert!(
                event_specs
                    .iter()
                    .any(|evt| evt.label() == &"InlineAnonymousEvent")
            );
            assert!(
                event_specs
                    .iter()
                    .any(|evt| evt.label() == &"InlineAnonymousEventHashedTopic")
            );

            // The event is not used directly in the code, but is included in the metadata
            // because we implement the trait from the `event_def_unused` crate.
            assert!(
                event_specs
                    .iter()
                    .any(|evt| evt.label() == &"EventDefUnused")
            );

            assert_eq!(9, event_specs.len());
        }

        #[ink::test]
        fn it_works() {
            let mut events = Events::new(false);
            events.flip_with_foreign_event();

            let emitted_events = ink::env::test::recorded_events();
            assert_eq!(1, emitted_events.len());
            let event = &emitted_events[0];

            let decoded_event = <event_def::ForeignFlipped>::decode(&mut &event.data[..])
                .expect("encountered invalid contract event data buffer");
            assert!(decoded_event.value);
        }

        #[ink::test]
        fn option_topic_some_has_topic() {
            let events = Events::new(false);
            events.emit_32_byte_topic_event(Some([0xAA; 32]));

            let emitted_events = ink::env::test::recorded_events();
            assert_eq!(1, emitted_events.len());
            let event = &emitted_events[0];

            assert_eq!(event.topics.len(), 3);
            let signature_topic =
                <event_def::ThirtyTwoByteTopics as ink::env::Event>::SIGNATURE_TOPIC;
            assert_eq!(Some(&event.topics[0]), signature_topic.as_ref());
            assert_eq!(event.topics[1], [0x42; 32]);
            assert_eq!(
                event.topics[2], [0xAA; 32],
                "option topic should be published"
            );
        }

        #[ink::test]
        fn option_topic_none_encoded_as_0() {
            let events = Events::new(false);
            events.emit_32_byte_topic_event(None);

            let emitted_events = ink::env::test::recorded_events();
            assert_eq!(1, emitted_events.len());
            let event = &emitted_events[0];

            let signature_topic =
                <event_def::ThirtyTwoByteTopics as ink::env::Event>::SIGNATURE_TOPIC
                    .unwrap();

            let expected_topics = vec![
                signature_topic,
                [0x42; 32],
                [0x00; 32], // None is encoded as 0x00
            ];
            assert_eq!(expected_topics, event.topics);
        }

        #[ink::test]
        fn custom_signature_topic() {
            let mut events = Events::new(false);
            events.flip_with_inline_custom_event();

            let emitted_events = ink::env::test::recorded_events();
            assert_eq!(1, emitted_events.len());

            let signature_topic =
                <InlineCustomFlipped as ink::env::Event>::SIGNATURE_TOPIC;

            assert_eq!(Some([17u8; 32]), signature_topic);
        }

        #[ink::test]
        fn anonymous_events_emit_no_signature_topics() {
            let events = Events::new(false);
            let topic = [0x42; 32];
            events.emit_anonymous_events(topic);

            let emitted_events = ink::env::test::recorded_events();
            assert_eq!(3, emitted_events.len());

            let event = &emitted_events[0];
            assert_eq!(event.topics.len(), 1);
            assert_eq!(event.topics[0], topic);

            let event = &emitted_events[1];
            assert_eq!(event.topics.len(), 1);
            assert_eq!(event.topics[0], topic);

            let signature_topic =
                <InlineAnonymousEvent as ink::env::Event>::SIGNATURE_TOPIC;
            assert_eq!(None, signature_topic);
        }
    }

    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        use super::*;
        use ink::H256;
        use ink_e2e::ContractsBackend;

        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        #[ink_e2e::test]
        async fn emits_foreign_event<Client: E2EBackend>(
            mut client: Client,
        ) -> E2EResult<()> {
            // given
            let init_value = false;
            let mut constructor = EventsRef::new(init_value);
            let contract = client
                .instantiate("events", &ink_e2e::alice(), &mut constructor)
                .submit()
                .await
                .expect("instantiate failed");
            let mut call_builder = contract.call_builder::<Events>();

            // when
            let flip = call_builder.flip_with_foreign_event();
            let flip_res = client
                .call(&ink_e2e::bob(), &flip)
                .submit()
                .await
                .expect("flip failed");

            let contract_events = flip_res.contract_emitted_events()?;

            // then
            assert_eq!(1, contract_events.len());
            let contract_event = &contract_events[0];
            let flipped: event_def::ForeignFlipped =
                ink::scale::Decode::decode(&mut &contract_event.event.data[..])
                    .expect("encountered invalid contract event data buffer");
            assert_eq!(!init_value, flipped.value);

            let signature_topic =
                <event_def::ForeignFlipped as ink::env::Event>::SIGNATURE_TOPIC
                    .map(H256::from)
                    .unwrap();

            let expected_topics = vec![signature_topic];
            assert_eq!(expected_topics, contract_event.topics);

            Ok(())
        }

        #[ink_e2e::test]
        async fn emits_inline_event<Client: E2EBackend>(
            mut client: Client,
        ) -> E2EResult<()> {
            // given
            let init_value = false;
            let mut constructor = EventsRef::new(init_value);
            let contract = client
                .instantiate("events", &ink_e2e::alice(), &mut constructor)
                .submit()
                .await
                .expect("instantiate failed");
            let mut call_builder = contract.call_builder::<Events>();

            // when
            let flip = call_builder.flip_with_inline_event();
            let flip_res = client
                .call(&ink_e2e::bob(), &flip)
                .submit()
                .await
                .expect("flip failed");

            let contract_events = flip_res.contract_emitted_events()?;

            // then
            assert_eq!(1, contract_events.len());
            let contract_event = &contract_events[0];
            let flipped: InlineFlipped =
                ink::scale::Decode::decode(&mut &contract_event.event.data[..])
                    .expect("encountered invalid contract event data buffer");
            assert_eq!(!init_value, flipped.value);

            let signature_topic = <InlineFlipped as ink::env::Event>::SIGNATURE_TOPIC
                .map(H256::from)
                .unwrap();

            let expected_topics = vec![signature_topic];
            assert_eq!(expected_topics, contract_event.topics);

            Ok(())
        }

        #[ink_e2e::test()]
        async fn emits_inline_anonymous_event<Client: E2EBackend>(
            mut client: Client,
        ) -> E2EResult<()> {
            use ink::env::hash::{
                Blake2x256,
                CryptoHash,
                HashOutput,
            };
            // given
            let init_value = false;
            let mut constructor = EventsRef::new(init_value);
            let contract = client
                .instantiate("events", &ink_e2e::alice(), &mut constructor)
                .submit()
                .await
                .expect("instantiate failed");
            let call_builder = contract.call_builder::<Events>();

            // when
            let topic = [1u8; 32];
            let emit = call_builder.emit_anonymous_events(topic);
            let flip_res = client
                .call(&ink_e2e::bob(), &emit)
                .submit()
                .await
                .expect("emit_anonymous_event failed");
            let contract_events = flip_res.contract_emitted_events()?;

            // then
            assert_eq!(3, contract_events.len());

            let contract_event = &contract_events[0];
            let evt: InlineAnonymousEvent =
                ink::scale::Decode::decode(&mut &contract_event.event.data[..])
                    .expect("encountered invalid contract event data buffer");
            assert_eq!(evt.topic, topic);
            assert_eq!(evt.field_1, 42);

            let contract_event = &contract_events[1];
            let evt: crate::AnonymousEvent =
                ink::scale::Decode::decode(&mut &contract_event.event.data[..])
                    .expect("encountered invalid contract event data buffer");
            assert_eq!(evt.topic, topic);
            assert_eq!(evt.field_1, 42);

            let contract_event = &contract_events[2];
            let evt: InlineAnonymousEventHashedTopic =
                ink::scale::Decode::decode(&mut &contract_event.event.data[..])
                    .expect("encountered invalid contract event data buffer");

            // Using 64 bytes will trigger the `Blake2x_256` hashing for the topic
            let two_topics = [1u8; 64];
            let mut hash_output =
                <<Blake2x256 as HashOutput>::Type as Default>::default();
            <Blake2x256 as CryptoHash>::hash(&two_topics, &mut hash_output);
            // We need to check the `topics[0]` field, as it will contain the hash
            // of `two_topics`
            assert_eq!(contract_event.topics[0], ink::H256(hash_output));

            assert_eq!(evt.topic, two_topics);
            assert_eq!(evt.field_1, 42);

            Ok(())
        }

        #[ink_e2e::test]
        async fn emits_event_with_option_topic_none<Client: E2EBackend>(
            mut client: Client,
        ) -> E2EResult<()> {
            // given
            let init_value = false;
            let mut constructor = EventsRef::new(init_value);
            let contract = client
                .instantiate("events", &ink_e2e::alice(), &mut constructor)
                .submit()
                .await
                .expect("instantiate failed");
            let call_builder = contract.call_builder::<Events>();

            // when
            let call = call_builder.emit_32_byte_topic_event(None);
            let call_res = client
                .call(&ink_e2e::bob(), &call)
                .submit()
                .await
                .expect("emit_32_byte_topic_event failed");

            let contract_events = call_res.contract_emitted_events()?;

            // then
            assert_eq!(1, contract_events.len());
            let contract_event = &contract_events[0];
            let event: event_def::ThirtyTwoByteTopics =
                ink::scale::Decode::decode(&mut &contract_event.event.data[..])
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

        #[ink_e2e::test]
        async fn emits_custom_signature_event<Client: E2EBackend>(
            mut client: Client,
        ) -> E2EResult<()> {
            // given
            let init_value = false;
            let mut constructor = EventsRef::new(init_value);
            let contract = client
                .instantiate("events", &ink_e2e::alice(), &mut constructor)
                .submit()
                .await
                .expect("instantiate failed");
            let mut call_builder = contract.call_builder::<Events>();

            // when
            let call = call_builder.flip_with_inline_custom_event();
            let call_res = client
                .call(&ink_e2e::bob(), &call)
                .submit()
                .await
                .expect("flip_with_inline_custom_event failed");

            let contract_events = call_res.contract_emitted_events()?;

            // then
            assert_eq!(1, contract_events.len());

            // todo the emitted event is not actually checked here
            let signature_topic =
                <InlineCustomFlipped as ink::env::Event>::SIGNATURE_TOPIC;

            assert_eq!(Some([17u8; 32]), signature_topic);

            Ok(())
        }
    }
}
