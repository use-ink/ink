use super::incrementer::*;
use ink_e2e::ContractsBackend;

type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[ink_e2e::test]
async fn migration_works<Client: E2EBackend>(mut client: Client) -> E2EResult<()> {
    // Given
    let constructor = IncrementerRef::new();
    let contract = client
        .instantiate("incrementer", &ink_e2e::alice(), constructor, 0, None)
        .await
        .expect("instantiate failed");
    let mut call = contract.call::<Incrementer>();

    let get = call.get();
    let get_res = client.call_dry_run(&ink_e2e::alice(), &get, 0, None).await;
    assert_eq!(get_res.return_value(), 0);

    let inc = call.inc();
    let _inc_result = client
        .call(&ink_e2e::alice(), &inc, 0, None)
        .await
        .expect("`inc` failed");

    let get = call.get();
    let get_res = client.call_dry_run(&ink_e2e::alice(), &get, 0, None).await;
    let pre_migration_value = get_res.return_value();
    assert_eq!(pre_migration_value, 1);

    // Upload the code for the contract to be updated to after the migration.
    let new_code_hash = client
        .upload("updated-incrementer", &ink_e2e::alice(), None)
        .await
        .expect("uploading `updated-incrementer` failed")
        .code_hash;
    let new_code_hash = new_code_hash.as_ref().try_into().unwrap();

    // Upload the code for the migration contract.
    let migration_contract = client
        .upload("migration", &ink_e2e::alice(), None)
        .await
        .expect("uploading `migration` failed");
    let migration_code_hash = migration_contract.code_hash.as_ref().try_into().unwrap();

    // When

    // Set the code hash to the migration contract
    let set_code = call.set_code(migration_code_hash);
    let _set_code_result = client
        .call(&ink_e2e::alice(), &set_code, 0, None)
        .await
        .expect("`set_code` failed");

    // Call the migration contract with a new value for `inc_by` and the code hash
    // of the updated contract.
    const NEW_INC_BY: u8 = 4;
    let migrate = contract
        .call::<migration::incrementer::Incrementer>()
        .migrate(NEW_INC_BY, new_code_hash);

    let _migration_result = client
        .call(&ink_e2e::alice(), &migrate, 0, None)
        .await
        .expect("`migrate` failed");

    // Then
    // Note that our contract's `AccountId` (so `contract_acc_id`) has stayed the
    // same between updates!
    let inc = contract
        .call::<updated_incrementer::incrementer::Incrementer>()
        .inc();

    let _inc_result = client
        .call(&ink_e2e::alice(), &inc, 0, None)
        .await
        .expect("`inc` failed");

    let get = call.get();
    let get_res = client.call_dry_run(&ink_e2e::alice(), &get, 0, None).await;

    // Remember, we updated our incrementer contract to increment by `4`.
    assert_eq!(
        get_res.return_value(),
        pre_migration_value + NEW_INC_BY as u32
    );

    Ok(())
}
