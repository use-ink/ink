use super::debugging_strategies::*;
use ink::env::Environment;
use ink_e2e::ContractsBackend;
#[cfg(feature = "debug")]
use ink::prelude::string::String;

type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// This test illustrates how to use debugging events.
///
/// The contract is build with the `debug` feature enabled, thus
/// we can have code in the contract that is utilized purely
/// for testing, but not for release builds.
#[cfg(feature = "debug")]
#[ink_e2e::test(features = ["debug"])]
async fn e2e_debugging_event_emitted(mut client: Client) -> E2EResult<()> {
    // Given
    let mut constructor = DebuggingStrategiesRef::new();
    let contract = client
        .instantiate("debugging_strategies", &ink_e2e::alice(), &mut constructor)
        .submit()
        .await
        .expect("instantiate failed");
    let call_builder = contract.call_builder::<DebuggingStrategies>();

    // When
    let call_res = client
        .call(&ink_e2e::alice(), &call_builder.get())
        .submit()
        .await
        .expect("calling `get` message failed");

    // Then
    // the contract will have emitted an event
    assert!(call_res.contains_event("Revive", "ContractEmitted"));
    let contract_events = call_res.contract_emitted_events()?;
    assert_eq!(1, contract_events.len());
    let contract_event = &contract_events[0];
    let debug_event: DebugEvent =
        ink::scale::Decode::decode(&mut &contract_event.event.data[..])
            .expect("encountered invalid contract event data buffer");
    assert_eq!(debug_event.message, "received 0");

    Ok(())
}

/// This test illustrates how to decode a `Revive::ContractReverted`.
#[cfg(feature = "debug")]
#[ink_e2e::test(features = ["debug"])]
async fn e2e_decode_intentional_revert(mut client: Client) -> E2EResult<()> {
    // Given
    let mut constructor = DebuggingStrategiesRef::new();
    let contract = client
        .instantiate("debugging_strategies", &ink_e2e::bob(), &mut constructor)
        .submit()
        .await
        .expect("instantiate failed");
    let call_builder = contract.call_builder::<DebuggingStrategies>();

    // When
    let call_res = client
        .call(&ink_e2e::alice(), &call_builder.intentional_revert())
        .dry_run()
        .await
        .expect("calling `get` message failed");

    // Then
    let return_data = call_res.return_data();
    assert!(call_res.did_revert());
    let revert_msg = String::from_utf8_lossy(return_data);
    assert!(revert_msg.contains("reverting with info: 0"));

    Ok(())
}

/// This test illustrates how to decode a `Revive::ContractReverted`.
#[ink_e2e::test]
async fn e2e_decode_revert(mut client: Client) -> E2EResult<()> {
    // Given
    let mut constructor = DebuggingStrategiesRef::new();
    let contract = client
        .instantiate("debugging_strategies", &ink_e2e::bob(), &mut constructor)
        .value(1_337_000_000)
        .dry_run()
        //.submit()
        .await
        .expect("instantiate failed");

    // When
    let return_data = contract.return_data();
    assert!(contract.did_revert());
    let revert_msg = String::from_utf8_lossy(return_data);
    assert!(revert_msg.contains("paid an unpayable message"));

    // todo show same for call
    let contract = client
        .instantiate("debugging_strategies", &ink_e2e::bob(), &mut constructor)
        .submit()
        .await
        .expect("instantiate failed");
    let call_builder = contract.call_builder::<DebuggingStrategies>();

    // When
    let call_res = client
        .call(&ink_e2e::alice(), &call_builder.get())
        .value(1_337_000_000)
        .dry_run()
        .await
        .expect("calling `get` message failed");

    // Then
    let return_data = call_res.return_data();
    assert!(call_res.did_revert());
    let revert_msg = String::from_utf8_lossy(return_data);
    assert!(
        revert_msg.contains(
            "dispatching ink! message failed: paid an unpayable message"
        )
    );

    Ok(())
}

/// This test illustrates how to use the `pallet-revive` tracing functionality.
#[ink_e2e::test]
async fn e2e_tracing(mut client: Client) -> E2EResult<()> {
    // Given
    let mut constructor = DebuggingStrategiesRef::new();
    let contract = client
        .instantiate("debugging_strategies", &ink_e2e::bob(), &mut constructor)
        .submit()
        .await
        .expect("instantiate failed");
    let mut call_builder = contract.call_builder::<DebuggingStrategies>();

    let call = call_builder.instantiate_and_call(contract.code_hash);
    let call_res = client
        .call(&ink_e2e::alice(), &call)
        .value(1_337_000_000)
        .submit()
        .await?;

    // When
    let trace: ink_e2e::CallTrace = call_res.trace.expect("trace must exist");
    
    // Check that we have 2 calls (one top level, one nested instantiation/call)
    assert_eq!(trace.calls.len(), 2);
    
    // Then
    // Verify the value matches what we sent
    assert_eq!(
        trace.value,
        Some(ink::env::DefaultEnvironment::native_to_eth(1_337_000_000))
    );

    Ok(())
}