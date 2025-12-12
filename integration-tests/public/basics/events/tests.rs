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
