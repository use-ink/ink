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
    #[cfg(ink_abi = "all")] // Appease clippy in CI.
    mod tests {
        use super::*;

        /// Returns the Blake2b 256-bit hash for the given input.
        pub(super) fn blake2x256(input: &[u8]) -> [u8; 32] {
            use ink::env::hash::{
                Blake2x256,
                CryptoHash,
            };

            let mut output = [0u8; 32];
            <Blake2x256 as CryptoHash>::hash(input, &mut output);
            output
        }

        /// Returns the Keccak-256 hash for the given input.
        pub(super) fn keccak_256(input: &[u8]) -> [u8; 32] {
            use ink::env::hash::{
                CryptoHash,
                Keccak256,
            };

            let mut output = [0u8; 32];
            <Keccak256 as CryptoHash>::hash(input, &mut output);
            output
        }

        #[test]
        fn collects_ink_specs_for_all_linked_and_used_events() {
            let event_specs = ink::collect_events();

            assert_eq!(9, event_specs.len());

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
        }

        #[test]
        fn collects_solidity_info_for_all_linked_and_used_events() {
            let event_specs = ink::collect_events_sol();

            assert_eq!(9, event_specs.len());

            assert!(event_specs.iter().any(|evt| evt.name == "ForeignFlipped"));
            assert!(event_specs.iter().any(|evt| evt.name == "InlineFlipped"));
            assert!(
                event_specs
                    .iter()
                    .any(|evt| evt.name == "InlineCustomFlipped")
            );
            assert!(
                event_specs
                    .iter()
                    .any(|evt| evt.name == "ThirtyTwoByteTopics")
            );
            assert!(
                event_specs
                    .iter()
                    .any(|evt| evt.name == "EventDefAnotherCrate")
            );
            assert!(event_specs.iter().any(|evt| evt.name == "AnonymousEvent"));
            assert!(
                event_specs
                    .iter()
                    .any(|evt| evt.name == "InlineAnonymousEvent")
            );
            assert!(
                event_specs
                    .iter()
                    .any(|evt| evt.name == "InlineAnonymousEventHashedTopic")
            );
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

            let emitted_events = ink::env::test::recorded_events();
            // 2 events, one each per ABI.
            assert_eq!(2, emitted_events.len());

            // ink! event.
            let event = &emitted_events[0];
            let decoded_event =
                <event_def::ForeignFlipped as ink::scale::Decode>::decode(
                    &mut &event.data[..],
                )
                .expect("encountered invalid contract event data buffer");
            assert!(decoded_event.value);

            // Solidity event.
            let event = &emitted_events[1];
            let decoded_data: (bool,) = ink::sol::decode_sequence(&event.data)
                .expect("encountered invalid contract event data buffer");
            assert!(decoded_data.0);
        }

        #[ink::test]
        fn option_topic_some() {
            let events = Events::new(false);
            events.emit_32_byte_topic_event(Some(ink::sol::FixedBytes([0xAA; 32])));

            let emitted_events = ink::env::test::recorded_events();
            // 2 events, one each per ABI.
            assert_eq!(2, emitted_events.len());

            // ink! event.
            let event = &emitted_events[0];
            assert_eq!(event.topics.len(), 3);
            let signature_topic = <event_def::ThirtyTwoByteTopics as ink::env::Event<
                ink::abi::Ink,
            >>::SIGNATURE_TOPIC;
            assert_eq!(Some(&event.topics[0]), signature_topic.as_ref());
            assert_eq!(event.topics[1], [0x42; 32]);
            assert_eq!(
                event.topics[2], [0xAA; 32],
                "option topic should be published"
            );

            // Solidity event.
            let event = &emitted_events[1];
            assert_eq!(event.topics.len(), 3);
            let signature_topic = <event_def::ThirtyTwoByteTopics as ink::env::Event<
                ink::abi::Sol,
            >>::SIGNATURE_TOPIC;
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
        fn option_topic_none() {
            let events = Events::new(false);
            events.emit_32_byte_topic_event(None);

            let emitted_events = ink::env::test::recorded_events();
            // 2 events, one each per ABI.
            assert_eq!(2, emitted_events.len());

            // ink! event.
            let event = &emitted_events[0];
            let signature_topic = <event_def::ThirtyTwoByteTopics as ink::env::Event<
                ink::abi::Ink,
            >>::SIGNATURE_TOPIC
                .unwrap();

            let expected_topics = vec![
                signature_topic,
                [0x42; 32],
                [0x00; 32], // None is encoded as 0x00
            ];
            assert_eq!(expected_topics, event.topics);

            // Solidity event.
            let event = &emitted_events[1];
            let signature_topic = <event_def::ThirtyTwoByteTopics as ink::env::Event<
                ink::abi::Sol,
            >>::SIGNATURE_TOPIC
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
        fn custom_signature_topic() {
            let mut events = Events::new(false);
            events.flip_with_inline_custom_event();

            let emitted_events = ink::env::test::recorded_events();
            // 2 events, one each per ABI.
            assert_eq!(2, emitted_events.len());

            // ink! signature uses custom topic.
            let signature_topic =
                <InlineCustomFlipped as ink::env::Event<ink::abi::Ink>>::SIGNATURE_TOPIC;
            assert_eq!(Some([17u8; 32]), signature_topic);

            // Solidity signature ignores custom topic.
            let signature_topic =
                <InlineCustomFlipped as ink::env::Event<ink::abi::Sol>>::SIGNATURE_TOPIC;
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

            let emitted_events = ink::env::test::recorded_events();
            // 6 events, each event is emitted twice, i.e. a separate event for each ABI.
            assert_eq!(6, emitted_events.len());

            // ink! events.
            let event = &emitted_events[0];
            assert_eq!(event.topics.len(), 1);
            assert_eq!(event.topics[0], topic.0);

            let event = &emitted_events[2];
            assert_eq!(event.topics.len(), 1);
            assert_eq!(event.topics[0], topic.0);

            let event = &emitted_events[4];
            assert_eq!(event.topics.len(), 1);
            assert_eq!(event.topics[0], blake2x256(&[0x42; 64]));

            let signature_topic =
                <InlineAnonymousEvent as ink::env::Event<ink::abi::Ink>>::SIGNATURE_TOPIC;
            assert_eq!(None, signature_topic);

            // Solidity events.
            let event = &emitted_events[1];
            assert_eq!(event.topics.len(), 1);
            assert_eq!(event.topics[0], topic.0);

            let event = &emitted_events[3];
            assert_eq!(event.topics.len(), 1);
            assert_eq!(event.topics[0], topic.0);

            let event = &emitted_events[5];
            assert_eq!(event.topics.len(), 1);
            assert_eq!(
                event.topics[0],
                keccak_256(&ink::SolEncode::encode(&[0x42; 64]))
            );

            let signature_topic =
                <InlineAnonymousEvent as ink::env::Event<ink::abi::Sol>>::SIGNATURE_TOPIC;
            assert_eq!(None, signature_topic);
        }
    }

    #[cfg(all(test, feature = "e2e-tests"))]
    #[cfg(ink_abi = "all")] // Appease clippy in CI.
    mod e2e_tests {
        use super::{
            tests::{
                blake2x256,
                keccak_256,
            },
            *,
        };

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
            // 2 events, one each per ABI.
            assert_eq!(2, contract_events.len());

            // ink! event.
            let contract_event = &contract_events[0];
            let flipped: event_def::ForeignFlipped =
                ink::scale::Decode::decode(&mut &contract_event.event.data[..])
                    .expect("encountered invalid contract event data buffer");
            assert_eq!(!init_value, flipped.value);

            let signature_topic = <event_def::ForeignFlipped as ink::env::Event<
                ink::abi::Ink,
            >>::SIGNATURE_TOPIC
                .map(H256::from)
                .unwrap();

            let expected_topics = vec![signature_topic];
            assert_eq!(expected_topics, contract_event.topics);

            // Solidity event.
            let contract_event = &contract_events[1];
            let decoded_data: (bool,) =
                ink::sol::decode_sequence(&contract_event.event.data)
                    .expect("encountered invalid contract event data buffer");
            assert!(decoded_data.0);

            let signature_topic = <event_def::ForeignFlipped as ink::env::Event<
                ink::abi::Sol,
            >>::SIGNATURE_TOPIC
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
            let topic = ink::sol::FixedBytes([1u8; 32]);
            let emit = call_builder.emit_anonymous_events(topic);
            let flip_res = client
                .call(&ink_e2e::bob(), &emit)
                .submit()
                .await
                .expect("emit_anonymous_event failed");
            let contract_events = flip_res.contract_emitted_events()?;

            // then
            // 6 events, each event is emitted twice, i.e. a separate event for each ABI.
            assert_eq!(6, contract_events.len());

            // ink! events.
            let contract_event = &contract_events[0];
            let evt: InlineAnonymousEvent =
                ink::scale::Decode::decode(&mut &contract_event.event.data[..])
                    .expect("encountered invalid contract event data buffer");
            assert_eq!(evt.topic, topic);
            assert_eq!(evt.field_1, 42);

            let contract_event = &contract_events[2];
            let evt: crate::AnonymousEvent =
                ink::scale::Decode::decode(&mut &contract_event.event.data[..])
                    .expect("encountered invalid contract event data buffer");
            assert_eq!(evt.topic, topic);
            assert_eq!(evt.field_1, 42);

            let contract_event = &contract_events[4];
            let evt: InlineAnonymousEventHashedTopic =
                ink::scale::Decode::decode(&mut &contract_event.event.data[..])
                    .expect("encountered invalid contract event data buffer");

            // Using 64 bytes will trigger the `Blake2x256` hashing for the topic
            // We need to check the `topics[0]` field, as it will contain the hash
            // of `two_topics`
            let two_topics = [1u8; 64];
            assert_eq!(contract_event.topics[0], ink::H256(blake2x256(&[1u8; 64])));

            assert_eq!(evt.topic, two_topics);
            assert_eq!(evt.field_1, 42);

            // Solidity events.
            // NOTE: Only non-indexed fields are included in the event data for Solidity
            // ABI events.
            let contract_event = &contract_events[1];
            assert_eq!(contract_event.topics[0], topic.0.into());
            let evt: (u32,) = ink::sol::decode_sequence(&contract_event.event.data[..])
                .expect("encountered invalid contract event data buffer");
            assert_eq!(evt.0, 42);

            let contract_event = &contract_events[3];
            assert_eq!(contract_event.topics[0], topic.0.into());
            let evt: (u32,) = ink::sol::decode_sequence(&contract_event.event.data[..])
                .expect("encountered invalid contract event data buffer");
            assert_eq!(evt.0, 42);

            let contract_event = &contract_events[5];
            // A "regular" array (i.e. `uint8[64]`) triggers the `keccak_256` hashing for
            // the encoded topic.
            let two_topics = [1u8; 64];
            assert_eq!(
                contract_event.topics[0],
                ink::H256(keccak_256(&ink::SolEncode::encode(&two_topics)))
            );
            let evt: (u32,) = ink::sol::decode_sequence(&contract_event.event.data[..])
                .expect("encountered invalid contract event data buffer");
            assert_eq!(evt.0, 42);

            Ok(())
        }

        #[ink_e2e::test]
        async fn emits_event_with_option_topic_some<Client: E2EBackend>(
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
            let call = call_builder
                .emit_32_byte_topic_event(Some(ink::sol::FixedBytes([0xAA; 32])));
            let call_res = client
                .call(&ink_e2e::bob(), &call)
                .submit()
                .await
                .expect("emit_32_byte_topic_event failed");

            let contract_events = call_res.contract_emitted_events()?;

            // then
            // 2 events, one each per ABI.
            assert_eq!(2, contract_events.len());

            // ink! event.
            let contract_event = &contract_events[0];
            let event: event_def::ThirtyTwoByteTopics =
                ink::scale::Decode::decode(&mut &contract_event.event.data[..])
                    .expect("encountered invalid contract event data buffer");
            assert_eq!(event.maybe_hash, Some([0xAA; 32].into()));

            let signature_topic = <event_def::ThirtyTwoByteTopics as ink::env::Event<
                ink::abi::Ink,
            >>::SIGNATURE_TOPIC
                .map(H256::from)
                .unwrap();
            assert_eq!(contract_event.topics[0], signature_topic);
            assert_eq!(contract_event.topics[1], [0x42; 32].into());
            // Some is encoded as the inner type.
            assert_eq!(
                contract_event.topics[2],
                [0xAA; 32].into(),
                "option topic should be published"
            );

            // Solidity event.
            let contract_event = &contract_events[1];
            assert!(contract_event.event.data.is_empty());

            let signature_topic = <event_def::ThirtyTwoByteTopics as ink::env::Event<
                ink::abi::Sol,
            >>::SIGNATURE_TOPIC
                .map(H256::from)
                .unwrap();
            assert_eq!(contract_event.topics[0], signature_topic);
            assert_eq!(contract_event.topics[1], [0x42; 32].into());
            // `Some` is encoded as the `Keccak-256` of the encoding of
            // `(true, FixedBytes<[0xAA; 32]>)`
            let mut topic_preimage = vec![0x00; 31];
            topic_preimage.push(0x01);
            topic_preimage.extend([0xAA; 32]);
            assert_eq!(
                contract_event.topics[2],
                keccak_256(&topic_preimage).into(),
                "option topic should be published"
            );

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
            // 2 events, one each per ABI.
            assert_eq!(2, contract_events.len());

            // ink! event.
            let contract_event = &contract_events[0];
            let event: event_def::ThirtyTwoByteTopics =
                ink::scale::Decode::decode(&mut &contract_event.event.data[..])
                    .expect("encountered invalid contract event data buffer");
            assert!(event.maybe_hash.is_none());

            let signature_topic = <event_def::ThirtyTwoByteTopics as ink::env::Event<
                ink::abi::Ink,
            >>::SIGNATURE_TOPIC
                .map(H256::from)
                .unwrap();

            let expected_topics = vec![
                signature_topic,
                [0x42; 32].into(),
                [0x00; 32].into(), // None is encoded as 0x00
            ];
            assert_eq!(expected_topics, contract_event.topics);

            // Solidity event.
            let contract_event = &contract_events[1];
            assert!(contract_event.event.data.is_empty());

            let signature_topic = <event_def::ThirtyTwoByteTopics as ink::env::Event<
                ink::abi::Sol,
            >>::SIGNATURE_TOPIC
                .map(H256::from)
                .unwrap();

            let expected_topics = vec![
                signature_topic,
                [0x42; 32].into(),
                // `None` is encoded as the `Keccak-256` of the encoding of
                // `(false, FixedBytes<[0u8; 32]>)`.
                keccak_256(&[0x00; 64]).into(),
            ];
            assert_eq!(expected_topics, contract_event.topics);

            Ok(())
        }
    }
}
