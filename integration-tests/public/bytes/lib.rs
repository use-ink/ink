#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::event]
pub struct FixedBytesPayload {
    pub data: ink::sol::FixedBytes<8>,
}

#[ink::event]
pub struct DynBytesPayload {
    pub data: ink::sol::DynBytes,
}

#[ink::contract]
pub mod bytes {
    use super::{
        DynBytesPayload,
        FixedBytesPayload,
    };

    #[ink(storage)]
    pub struct Bytes;

    impl Bytes {
        /// Creates a new bytes smart contract.
        #[ink(constructor)]
        #[allow(clippy::new_without_default)]
        pub fn new() -> Self {
            Self {}
        }

        /// Handles fixed-size byte arrays.
        #[ink(message)]
        pub fn handle_fixed_bytes(&mut self, data: ink::sol::FixedBytes<8>) {
            self.env().emit_event(FixedBytesPayload { data })
        }

        /// Handles dynamic size byte arrays.
        #[ink(message)]
        pub fn handle_dyn_bytes(&mut self, data: ink::sol::DynBytes) {
            self.env().emit_event(DynBytesPayload { data })
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[ink::test]
        fn fixed_bytes_works() {
            // given
            let mut bytes = Bytes::new();

            // when
            let fixed_bytes =
                ink::sol::FixedBytes::from([0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7, 0x8]);
            bytes.handle_fixed_bytes(fixed_bytes);

            // then
            let emitted_events = ink::env::test::recorded_events();
            assert_eq!(1, emitted_events.len());

            // then
            let event = &emitted_events[0];
            let mut encoded = vec![0x0; 32];
            encoded.as_mut_slice()[..8].copy_from_slice(fixed_bytes.as_slice());
            assert_eq!(encoded, event.data);

            // then
            let decoded_data =
                ink::sol::decode_sequence::<(ink::sol::FixedBytes<8>,)>(&event.data)
                    .expect("encountered invalid contract event data buffer");
            assert_eq!(decoded_data.0, fixed_bytes);
        }

        #[ink::test]
        fn dyn_bytes_works() {
            // given
            let mut bytes = Bytes::new();

            // when
            let dyn_bytes = ink::sol::DynBytes::from(vec![0x1, 0x2, 0x3, 0x4]);
            bytes.handle_dyn_bytes(dyn_bytes.clone());

            // then
            let emitted_events = ink::env::test::recorded_events();
            assert_eq!(1, emitted_events.len());

            // then
            let event = &emitted_events[0];
            let mut encoded = vec![0x0; 96];
            encoded[31] = 32; // offset
            encoded[63] = dyn_bytes.len() as u8; // length
            encoded.as_mut_slice()[64..64 + dyn_bytes.len()]
                .copy_from_slice(dyn_bytes.as_ref());
            assert_eq!(encoded, event.data);

            // then
            let decoded_data =
                ink::sol::decode_sequence::<(ink::sol::DynBytes,)>(&event.data)
                    .expect("encountered invalid contract event data buffer");
            assert_eq!(decoded_data.0, dyn_bytes);
        }
    }

    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        use super::*;

        use ink_e2e::ContractsBackend;

        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        #[ink_e2e::test]
        async fn fixed_bytes_works<Client: E2EBackend>(
            mut client: Client,
        ) -> E2EResult<()> {
            // given
            let mut constructor = BytesRef::new();
            let contract = client
                .instantiate("bytes", &ink_e2e::alice(), &mut constructor)
                .submit()
                .await
                .expect("instantiate failed");
            let mut call_builder = contract.call_builder::<Bytes>();

            // when
            let fixed_bytes =
                ink::sol::FixedBytes::from([0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7, 0x8]);
            let handler = call_builder.handle_fixed_bytes(fixed_bytes);
            let res = client
                .call(&ink_e2e::bob(), &handler)
                .submit()
                .await
                .expect("fixed bytes handler failed");

            // then
            let contract_events = res.contract_emitted_events()?;
            assert_eq!(1, contract_events.len());

            // then
            let contract_event = &contract_events[0];
            let mut encoded = vec![0x0; 32];
            encoded.as_mut_slice()[..8].copy_from_slice(fixed_bytes.as_slice());
            assert_eq!(encoded, contract_event.event.data);

            // then
            let decoded_data = ink::sol::decode_sequence::<(ink::sol::FixedBytes<8>,)>(
                &contract_event.event.data,
            )
            .expect("encountered invalid contract event data buffer");
            assert_eq!(decoded_data.0, fixed_bytes);

            Ok(())
        }

        #[ink_e2e::test]
        async fn dyn_bytes_works<Client: E2EBackend>(
            mut client: Client,
        ) -> E2EResult<()> {
            // given
            let mut constructor = BytesRef::new();
            let contract = client
                .instantiate("bytes", &ink_e2e::alice(), &mut constructor)
                .submit()
                .await
                .expect("instantiate failed");
            let mut call_builder = contract.call_builder::<Bytes>();

            // when
            let dyn_bytes = ink::sol::DynBytes::from(vec![0x1, 0x2, 0x3, 0x4]);
            let handler = call_builder.handle_dyn_bytes(dyn_bytes.clone());
            let res = client
                .call(&ink_e2e::bob(), &handler)
                .submit()
                .await
                .expect("dyn bytes handler failed");

            // then
            let contract_events = res.contract_emitted_events()?;
            assert_eq!(1, contract_events.len());

            // then
            let contract_event = &contract_events[0];
            let mut encoded = vec![0x0; 96];
            encoded[31] = 32; // offset
            encoded[63] = dyn_bytes.len() as u8; // length
            encoded.as_mut_slice()[64..64 + dyn_bytes.len()]
                .copy_from_slice(dyn_bytes.as_ref());
            assert_eq!(encoded, contract_event.event.data);

            // then
            let decoded_data = ink::sol::decode_sequence::<(ink::sol::DynBytes,)>(
                &contract_event.event.data,
            )
            .expect("encountered invalid contract event data buffer");
            assert_eq!(decoded_data.0, dyn_bytes);

            Ok(())
        }
    }
}
