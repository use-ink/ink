#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::event(anonymous)]
pub struct AnonymousEvent {
    #[ink(topic)]
    pub topic: ink::sol::FixedBytes<32>,
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
        pub topic: ink::sol::FixedBytes<32>,
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
        pub fn emit_32_byte_topic_event(
            &self,
            maybe_hash: Option<ink::sol::FixedBytes<32>>,
        ) {
            self.env().emit_event(event_def::ThirtyTwoByteTopics {
                hash: ink::sol::FixedBytes([0x42; 32]),
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

        /// Emit inline and standalone anonymous events
        #[ink(message)]
        pub fn emit_anonymous_events(&self, topic: ink::sol::FixedBytes<32>) {
            self.env()
                .emit_event(InlineAnonymousEvent { topic, field_1: 42 });
            self.env()
                .emit_event(super::AnonymousEvent { topic, field_1: 42 });
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

        /// Computes the Keccak-256 hash for the given input and stores it in output.
        fn keccak_256(input: &[u8]) -> [u8; 32] {
            use sha3::Digest as _;
            let mut hasher = sha3::Keccak256::new();
            hasher.update(input);
            hasher.finalize().into()
        }

        #[test]
        fn collects_solidity_info_for_all_linked_and_used_events() {
            let event_specs = ink::collect_events_sol();

            assert_eq!(8, event_specs.len());

            assert!(event_specs.iter().any(|evt| evt.name == "ForeignFlipped"));
            assert!(event_specs.iter().any(|evt| evt.name == "InlineFlipped"));
            assert!(event_specs
                .iter()
                .any(|evt| evt.name == "InlineCustomFlipped"));
            assert!(event_specs
                .iter()
                .any(|evt| evt.name == "ThirtyTwoByteTopics"));
            assert!(event_specs
                .iter()
                .any(|evt| evt.name == "EventDefAnotherCrate"));
            assert!(event_specs.iter().any(|evt| evt.name == "AnonymousEvent"));
            assert!(event_specs
                .iter()
                .any(|evt| evt.name == "InlineAnonymousEvent"));
            assert!(event_specs.iter().any(|evt| evt.name == "EventDefUnused"));

            assert!(event_specs.iter().any(|evt| {
                evt.name == "InlineAnonymousEvent"
                    && evt
                        .params
                        .iter()
                        .any(|param| param.name == "topic" && param.is_topic)
            }));
        }

        #[ink::test]
        fn it_works() {
            let mut events = Events::new(false);
            events.flip_with_foreign_event();

            let emitted_events = ink::env::test::recorded_events().collect::<Vec<_>>();
            assert_eq!(1, emitted_events.len());
            let event = &emitted_events[0];

            let decoded_data = ink::sol::decode_sequence::<(bool,)>(&event.data)
                .expect("encountered invalid contract event data buffer");
            assert!(decoded_data.0);
        }

        #[ink::test]
        fn option_topic_some_has_topic() {
            let events = Events::new(false);
            events.emit_32_byte_topic_event(Some(ink::sol::FixedBytes([0xAA; 32])));

            let emitted_events = ink::env::test::recorded_events().collect::<Vec<_>>();
            assert_eq!(1, emitted_events.len());
            let event = &emitted_events[0];

            assert_eq!(event.topics.len(), 3);
            let signature_topic =
                <event_def::ThirtyTwoByteTopics as ink::env::Event>::SIGNATURE_TOPIC;
            assert_eq!(Some(&event.topics[0]), signature_topic.as_ref());
            assert_eq!(event.topics[1], [0x42; 32]);
            // `Some` is encoded as the `Keccak-256` of the encoding of
            // `(true, FixedBytes<[0xAA; 32]>)`
            let mut topic_preimage = vec![0x00; 31];
            topic_preimage.push(0x01);
            topic_preimage.extend([0xAA; 32]);
            assert_eq!(
                event.topics[2],
                keccak_256(&topic_preimage),
                "option topic should be published"
            );
        }

        #[ink::test]
        fn option_topic_none_encoded_as_tuple_with_defaults() {
            let events = Events::new(false);
            events.emit_32_byte_topic_event(None);

            let emitted_events = ink::env::test::recorded_events().collect::<Vec<_>>();
            assert_eq!(1, emitted_events.len());
            let event = &emitted_events[0];

            let signature_topic =
                <event_def::ThirtyTwoByteTopics as ink::env::Event>::SIGNATURE_TOPIC
                    .unwrap();

            let expected_topics = vec![
                signature_topic,
                [0x42; 32],
                // `None` is encoded as the `Keccak-256` of the encoding of
                // `(false, FixedBytes<[0u8; 32]>)`.
                keccak_256(&[0x00; 64]),
            ];
            assert_eq!(expected_topics, event.topics);
        }

        #[ink::test]
        fn custom_signature_topic_ignored() {
            let mut events = Events::new(false);
            events.flip_with_inline_custom_event();

            let emitted_events = ink::env::test::recorded_events().collect::<Vec<_>>();
            assert_eq!(1, emitted_events.len());

            let signature_topic =
                <InlineCustomFlipped as ink::env::Event>::SIGNATURE_TOPIC;

            assert_eq!(
                Some(ink::keccak_256!("InlineCustomFlipped(bool)")),
                signature_topic
            );
        }

        #[ink::test]
        fn anonymous_events_emit_no_signature_topics() {
            let events = Events::new(false);
            let topic = ink::sol::FixedBytes([0x42; 32]);
            events.emit_anonymous_events(topic);

            let emitted_events = ink::env::test::recorded_events().collect::<Vec<_>>();
            assert_eq!(2, emitted_events.len());

            let event = &emitted_events[0];
            assert_eq!(event.topics.len(), 1);
            assert_eq!(event.topics[0], topic.0);

            let event = &emitted_events[1];
            assert_eq!(event.topics.len(), 1);
            assert_eq!(event.topics[0], topic.0);

            let signature_topic =
                <InlineAnonymousEvent as ink::env::Event>::SIGNATURE_TOPIC;
            assert_eq!(None, signature_topic);
        }
    }
}
