use super::wildcard_selector::*;

#[cfg(all(test, feature = "e2e-tests"))]
mod e2e_tests {
    use super::*;
    use ink_e2e::ContractsBackend;

    use ink::{
        env::call::utils::{
            Argument,
            ArgumentList,
            EmptyArgumentList,
        },
        primitives::abi::Ink,
    };

    type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;
    type Environment = <WildcardSelectorRef as ink::env::ContractEnv>::Env;

    fn build_message(
        addr: AccountId, // Changed to AccountId (which is H160 in v6) to match Address alias usage usually found
        selector: [u8; 4],
        message: String,
    ) -> ink_e2e::CallBuilderFinal<
        Environment,
        ArgumentList<Argument<String>, EmptyArgumentList<Ink>, Ink>,
        (),
        Ink,
    > {
        ink::env::call::build_call::<Environment>()
            .call(addr)
            .exec_input(
                ink::env::call::ExecutionInput::new(ink::env::call::Selector::new(
                    selector,
                ))
                .push_arg(message),
            )
            .returns::<()>()
    }

    #[ink_e2e::test(features = ["emit-event"])]
    async fn arbitrary_selectors_handled_by_wildcard(
        mut client: Client,
    ) -> E2EResult<()> {
        // Given
        let mut constructor = WildcardSelectorRef::new();
        let contract_acc_id = client
            .instantiate("wildcard_selector", &ink_e2e::alice(), &mut constructor)
            .submit()
            .await
            .expect("instantiate failed")
            .addr;

        // When
        const ARBITRARY_SELECTOR: [u8; 4] = [0xF9, 0xF9, 0xF9, 0xF9];
        let wildcard_message = "WILDCARD_MESSAGE 1".to_string();
        let wildcard = build_message(
            contract_acc_id,
            ARBITRARY_SELECTOR,
            wildcard_message.clone(),
        );

        let _result = client
            .call(&ink_e2e::bob(), &wildcard)
            .submit()
            .await
            .expect("wildcard failed");

        const ARBITRARY_SELECTOR_2: [u8; 4] = [0x01, 0x23, 0x45, 0x67];
        let wildcard_message2 = "WILDCARD_MESSAGE 2".to_string();
        let wildcard2 = build_message(
            contract_acc_id,
            ARBITRARY_SELECTOR_2,
            wildcard_message2.clone(),
        );

        let _result2 = client
            .call(&ink_e2e::bob(), &wildcard2)
            .submit()
            .await
            .expect("wildcard failed");

        // Then
        let contract_events = _result.contract_emitted_events()?;
        assert_eq!(1, contract_events.len());
        let contract_event = contract_events.first().expect("first event must exist");
        let event: Event =
            ink::scale::Decode::decode(&mut &contract_event.event.data[..])
                .expect("encountered invalid contract event data buffer");
        assert_eq!(
            event.msg,
            format!(
                "Wildcard selector: {ARBITRARY_SELECTOR:?}, message: {wildcard_message}"
            )
        );

        Ok(())
    }

    #[ink_e2e::test]
    async fn wildcard_complement_works(mut client: Client) -> E2EResult<()> {
        // Given
        let mut constructor = WildcardSelectorRef::new();
        let contract_acc_id = client
            .instantiate("wildcard_selector", &ink_e2e::alice(), &mut constructor)
            .submit()
            .await
            .expect("instantiate failed")
            .addr;

        // When
        let wildcard_complement_message = "WILDCARD COMPLEMENT MESSAGE".to_string();
        let wildcard = build_message(
            contract_acc_id,
            ink::IIP2_WILDCARD_COMPLEMENT_SELECTOR,
            wildcard_complement_message.clone(),
        );

        let _result = client
            .call(&ink_e2e::bob(), &wildcard)
            .submit()
            .await
            .expect("wildcard failed");

        Ok(())
    }
}