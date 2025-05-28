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
        pub fn emit_32_byte_topic_event(&self) {
            self.env()
                .emit_event(event_def::ThirtyTwoByteTopics { hash: [0x42; 32] })
        }

        /// Emit an event from a different crate.
        #[ink(message)]
        pub fn emit_event_from_a_different_crate(&self) {
            self.env()
                .emit_event(event_def2::EventDefAnotherCrate { hash: [0x42; 32] })
        }

        /// Emit a inline and standalone anonymous events
        #[ink(message)]
        pub fn emit_anonymous_events(&self, topic: [u8; 32]) {
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
    }
}
