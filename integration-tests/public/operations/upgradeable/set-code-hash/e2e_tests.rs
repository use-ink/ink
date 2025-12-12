use super::*;
use ink_e2e::ContractsBackend;

type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[ink_e2e::test]
async fn set_code_works(mut client: Client) -> E2EResult<()> {
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
    assert!(matches!(get_res.return_value(), 0));

    let inc = call_builder.inc();
    let _inc_result = client
        .call(&ink_e2e::alice(), &inc)
        .submit()
        .await
        .expect("`inc` failed");

    let get = call_builder.get();
    let get_res = client.call(&ink_e2e::alice(), &get).dry_run().await?;
    assert!(matches!(get_res.return_value(), 1));

    // When
    let new_code_hash = client
        .upload("updated_incrementer", &ink_e2e::alice())
        .submit()
        .await
        .expect("uploading `updated_incrementer` failed")
        .code_hash;

    let set_code = call_builder.set_code(new_code_hash);

    let _set_code_result = client
        .call(&ink_e2e::alice(), &set_code)
        .submit()
        .await
        .expect("`set_code` failed");

    // Then
    // Note that our contract's `AccountId` (so `contract_acc_id`) has stayed the
    // same between updates!
    let inc = call_builder.inc();

    let _inc_result = client
        .call(&ink_e2e::alice(), &inc)
        .submit()
        .await
        .expect("`inc` failed");

    let get = call_builder.get();
    let get_res = client.call(&ink_e2e::alice(), &get).dry_run().await?;

    // Remember, we updated our incrementer contract to increment by `4`.
    assert!(matches!(get_res.return_value(), 5));

    Ok(())
}
