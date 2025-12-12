use super::*;
use ink::H256;
use ink_e2e::ContractsBackend;

type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[ink_e2e::test]
async fn emits_foreign_event(mut client: Client) -> E2EResult<()> {
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
async fn emits_inline_event(mut client: Client) -> E2EResult<()> {
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
async fn emits_inline_anonymous_event(mut client: Client) -> E2EResult<()> {
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
async fn emits_event_with_option_topic_none(mut client: Client) -> E2EResult<()> {
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
async fn emits_custom_signature_event(mut client: Client) -> E2EResult<()> {
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
