use super::*;
use delegatee::delegatee::{
    Delegatee,
    DelegateeRef,
};
use delegatee2::delegatee2::{
    Delegatee2,
    Delegatee2Ref,
};
use ink_e2e::{
    ChainBackend,
    ContractsBackend,
};

type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[ink_e2e::test]
async fn e2e_counter_mutated(mut client: Client) -> E2EResult<()> {
    // given
    let origin = client
        .create_and_fund_account(&ink_e2e::alice(), 10_000_000_000_000)
        .await;

    /*
    // todo
    let code_hash = client
        .upload("delegatee", &origin)
        .submit()
        .await
        .expect("upload `delegatee` failed")
        .code_hash;
        */

    let mut constructor = DelegateeRef::new();
    let contract = client
        .instantiate("delegatee", &origin, &mut constructor)
        .submit()
        .await
        .expect("instantiate `delegatee` failed");
    let call_builder = contract.call_builder::<Delegatee>();
    let call_delegatee = call_builder.code_hash();
    let result = client
        .call(&origin, &call_delegatee)
        .dry_run()
        .await
        .expect("code_hash call failed");
    let code_hash = result.return_value();

    let mut constructor = DelegatorRef::new(0, code_hash, contract.addr);
    let contract = client
        .instantiate("delegator", &origin, &mut constructor)
        .submit()
        .await
        .expect("instantiate failed");
    let mut call_builder = contract.call_builder::<Delegator>();

    // when
    let call_delegate = call_builder.inc_delegate();

    let result = client.call(&origin, &call_delegate).submit().await;
    assert!(result.is_ok(), "delegate call failed.");

    let result = client.call(&origin, &call_delegate).submit().await;
    assert!(result.is_ok(), "second delegate call failed.");

    // then
    let expected_value = 4;
    let call_builder = contract.call_builder::<Delegator>();

    let call_get = call_builder.get_counter();
    let call_get_result = client
        .call(&origin, &call_get)
        .dry_run()
        .await?
        .return_value();

    // This fails
    assert_eq!(
        call_get_result, expected_value,
        "Decoded an unexpected value from the call."
    );

    Ok(())
}

#[ink_e2e::test]
async fn e2e_mapping_mutated(mut client: Client) -> E2EResult<()> {
    let origin = client
        .create_and_fund_account(&ink_e2e::alice(), 10_000_000_000_000)
        .await;

    /*
    let code_hash = client
        .upload("delegatee", &origin)
        .submit()
        .await
        .expect("upload `delegatee` failed")
        .code_hash;
        */

    let mut constructor = DelegateeRef::new();
    let contract = client
        .instantiate("delegatee", &origin, &mut constructor)
        .submit()
        .await
        .expect("instantiate `delegatee` failed");
    let call_builder = contract.call_builder::<Delegatee>();
    let call_delegatee = call_builder.code_hash();
    let result = client
        .call(&origin, &call_delegatee)
        .dry_run()
        .await
        .expect("code_hash call failed");
    let code_hash = result.return_value();

    // given
    let mut constructor = DelegatorRef::new(10, code_hash, contract.addr);
    let contract = client
        .instantiate("delegator", &origin, &mut constructor)
        .submit()
        .await
        .expect("instantiate failed");
    let mut call_builder = contract.call_builder::<Delegator>();

    // when
    let call_delegate = call_builder.add_entry_delegate();
    let result = client.call(&origin, &call_delegate).submit().await;
    assert!(result.is_ok(), "delegate call failed.");

    // then

    // because we initialize the counter with `10` we expect this value be
    // assigned to Alice.
    let expected_value = 10;
    // Alice's address
    // todo
    let acc = origin.public_key().to_account_id().0;
    let address = ink::primitives::AccountIdMapper::to_address(&acc);

    let call_get_value = call_builder.get_value(address);
    let call_get_result = client
        .call(&origin, &call_get_value)
        .submit()
        .await
        .unwrap()
        .return_value();

    assert_eq!(
        call_get_result,
        (address, Some(expected_value)),
        "Decoded an unexpected value from the call."
    );

    Ok(())
}

#[ink_e2e::test]
async fn update_delegate(mut client: Client) -> E2EResult<()> {
    // given
    let origin = client
        .create_and_fund_account(&ink_e2e::alice(), 10_000_000_000_000)
        .await;

    /*
    // todo
    let code_hash = client
        .upload("delegatee", &origin)
        .submit()
        .await
        .expect("upload `delegatee` failed")
        .code_hash;
        */
    let mut constructor = DelegateeRef::new();
    let contract = client
        .instantiate("delegatee", &origin, &mut constructor)
        .submit()
        .await
        .expect("instantiate `delegatee` failed");
    let call_builder = contract.call_builder::<Delegatee>();
    let call_delegatee = call_builder.code_hash();
    let result = client
        .call(&origin, &call_delegatee)
        .dry_run()
        .await
        .expect("code_hash call to delegatee failed");
    let code_hash = result.return_value();
    let delegatee_addr = contract.addr;

    /*
    // todo
    let code_hash2 = client
        .upload("delegatee2", &origin)
        .submit()
        .await
        .expect("upload `delegatee2` failed")
        .code_hash;
        */
    let mut constructor = Delegatee2Ref::new();
    let contract2 = client
        .instantiate("delegatee2", &origin, &mut constructor)
        .submit()
        .await
        .expect("instantiate `delegatee2` failed");
    let call_builder2 = contract.call_builder::<Delegatee2>();
    let call_delegatee2 = call_builder2.code_hash();
    let result2 = client
        .call(&origin, &call_delegatee2)
        .dry_run()
        .await
        .expect("code_hash call to delegatee2 failed");
    let code_hash2 = result2.return_value();
    let delegatee2_addr = contract2.addr;

    let mut constructor = DelegatorRef::new(10, code_hash, delegatee_addr);
    let contract = client
        .instantiate("delegator", &origin, &mut constructor)
        .submit()
        .await
        .expect("instantiate failed");
    let mut call_builder = contract.call_builder::<Delegator>();

    // when
    let call_delegate =
        call_builder.update_delegate_to(code_hash2, delegatee2_addr);
    let result = client.call(&origin, &call_delegate).submit().await;
    assert!(result.is_ok(), "update_delegate_to failed.");

    // then

    // todo this doesn't work right now, as the contract is still alive and
    // thus the code in use.
    // remove the original delegatee code.
    // should succeed because the delegate dependency has been removed.
    /*
    let original_code_removed =
        client.remove_code(&origin, code_hash).submit().await;
    assert!(original_code_removed.is_ok());

    // attempt to remove the new delegatee code.
    // should fail because of the delegate dependency.
    let new_code_removed = client.remove_code(&origin, code_hash2).submit().await;
    assert!(new_code_removed.is_err());
    */

    Ok(())
}
