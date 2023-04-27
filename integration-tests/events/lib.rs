#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
pub mod events {
    use event_def::Flipped;

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
            self.env().emit_event(Flipped { flipped: self.value })
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[ink::test]
        fn it_works() {
            let mut events = Events::new(false);
            events.flip();
            // todo: check events.
        }
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
