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
            self.env().emit_event(event_def::Flipped { flipped: self.value })
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
            assert_eq!(decoded_event.flipped, true);
        }

        // #[ink::test]
        // fn option_topic_some_has_topic() {
        //     let mut events = Events::new(false);
        //     events.emit_32_byte_topic_event();
        //     // todo: check events.
        // }
        //
        // #[ink::test]
        // fn option_topic_none_missing_topic() {
        //     let mut events = Events::new(false);
        //     events.emit_32_byte_topic_event();
        //     // todo: check events.
        // }
    }

    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        use super::*;
        use ink_e2e::build_message;

        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        #[ink_e2e::test]
        async fn it_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // // given
            // let constructor = FlipperRef::new(false);
            // let contract_acc_id = client
            //     .instantiate("events", &ink_e2e::alice(), constructor, 0, None)
            //     .await
            //     .expect("instantiate failed")
            //     .account_id;
            //
            // let get = build_message::<FlipperRef>(contract_acc_id.clone())
            //     .call(|events| events.get());
            // let get_res = client.call_dry_run(&ink_e2e::bob(), &get, 0, None).await;
            // assert!(matches!(get_res.return_value(), false));
            //
            // // when
            // let flip = build_message::<FlipperRef>(contract_acc_id.clone())
            //     .call(|events| events.flip());
            // let _flip_res = client
            //     .call(&ink_e2e::bob(), flip, 0, None)
            //     .await
            //     .expect("flip failed");
            //
            // // then
            // let get = build_message::<FlipperRef>(contract_acc_id.clone())
            //     .call(|events| events.get());
            // let get_res = client.call_dry_run(&ink_e2e::bob(), &get, 0, None).await;
            // assert!(matches!(get_res.return_value(), true));
            //
            // Ok(())
        }
    }
}
