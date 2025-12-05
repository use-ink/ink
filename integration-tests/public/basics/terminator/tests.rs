use super::just_terminates::*;

#[ink::test]
fn terminating_works() {
    // given
    let accounts = ink::env::test::default_accounts();
    let contract_id = ink::env::test::callee();
    ink::env::test::set_caller(accounts.alice);
    ink::env::test::set_contract_balance(contract_id, 100.into());
    let mut contract = JustTerminate::new();

    // when
    let should_terminate = move || contract.terminate_me();

    // then
    ink::env::test::assert_contract_termination::<ink::env::DefaultEnvironment, _>(
        should_terminate,
        accounts.alice,
        100.into(),
    );
}

#[cfg(all(test, feature = "e2e-tests"))]
mod e2e_tests {
    use super::*;
    use ink_e2e::ContractsBackend;

    type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

    #[ink_e2e::test]
    async fn e2e_contract_terminates(mut client: Client) -> E2EResult<()> {
        // given
        let mut constructor = JustTerminateRef::new();
        let contract = client
            .instantiate("contract_terminate", &ink_e2e::alice(), &mut constructor)
            .submit()
            .await
            .expect("instantiate failed");
        let mut call_builder = contract.call_builder::<JustTerminate>();

        // when
        let terminate_me = call_builder.terminate_me();
        let call_res = client
            .call(&ink_e2e::alice(), &terminate_me)
            .submit()
            .await
            .expect("terminate_me messages failed");

        assert!(
            call_res.return_data().is_empty(),
            "Terminated contract never returns"
        );

        // then
        assert!(call_res.contains_event("System", "KilledAccount"));
        assert!(call_res.contains_event("Balances", "Withdraw"));
        // todo this event below no longer exists, but we could try getting
        // info for the contract and asserting that it fails.
        // assert!(call_res.contains_event("Revive", "Terminated"));

        Ok(())
    }
}