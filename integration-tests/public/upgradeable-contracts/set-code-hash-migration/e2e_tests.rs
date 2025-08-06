use super::incrementer::*;
use ink_e2e::ContractsBackend;

type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[ink_e2e::test]
async fn migration_works<Client: E2EBackend>(mut client: Client) -> E2EResult<()> {
    // Given
    let mut constructor = IncrementerRef::new();
    let contract = client
        .instantiate("incrementer", &ink_e2e::alice(), &mut constructor)
        .submit()
        .await
        .expect("instantiate failed");
    let mut call_builder = contract.call_builder::<Incrementer>();

    let get = call_builder.get();
    let get_res = client.call(&ink_e2e::alice(), &get).dry_run().await?;
    assert_eq!(get_res.return_value(), 0);

    let inc = call_builder.inc();
    let _inc_result = client
        .call(&ink_e2e::alice(), &inc)
        .submit()
        .await
        .expect("`inc` failed");

    let get = call_builder.get();
    let get_res = client.call(&ink_e2e::alice(), &get).dry_run().await?;
    let pre_migration_value = get_res.return_value();
    assert_eq!(pre_migration_value, 1);

    // Upload the code for the contract to be updated to after the migration.
    let new_code_hash = client
        .upload("updated-incrementer", &ink_e2e::alice())
        .submit()
        .await
        .expect("uploading `updated-incrementer` failed")
        .code_hash;

    // Upload the code for the migration contract.
    let migration_contract = client
        .upload("migration", &ink_e2e::alice())
        .submit()
        .await
        .expect("uploading `migration` failed");
    let migration_code_hash = migration_contract.code_hash;

    // When

    // Set the code hash to the migration contract
    let set_code = call_builder.set_code(migration_code_hash);
    let _set_code_result = client
        .call(&ink_e2e::alice(), &set_code)
        .submit()
        .await
        .expect("`set_code` failed");

    // Call the migration contract with a new value for `inc_by` and the code hash
    // of the updated contract.
    const NEW_INC_BY: u8 = 4;
    let migrate = contract
        .call_builder::<migration::incrementer::Incrementer>()
        .migrate(NEW_INC_BY, new_code_hash);

    let _migration_result = client
        .call(&ink_e2e::alice(), &migrate)
        .submit()
        .await
        .expect("`migrate` failed");

    // Then
    let inc = contract
        .call_builder::<updated_incrementer::incrementer::Incrementer>()
        .inc();

    let _inc_result = client
        .call(&ink_e2e::alice(), &inc)
        .submit()
        .await
        .expect("`inc` failed");

    let get = contract
        .call_builder::<updated_incrementer::incrementer::Incrementer>()
        .get();
    let get_res = client.call(&ink_e2e::alice(), &get).dry_run().await?;

    // Remember, we updated our incrementer contract to increment by `4`.
    assert_eq!(
        get_res.return_value(),
        pre_migration_value as u64 + NEW_INC_BY as u64
    );

    Ok(())
}
