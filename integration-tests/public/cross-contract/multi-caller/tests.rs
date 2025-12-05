use super::multi_contract_caller::{MultiContractCaller, MultiContractCallerRef};
use ink_e2e::ContractsBackend;

type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[ink_e2e::test]
async fn e2e_multi_contract_caller(mut client: Client) -> E2EResult<()> {
    // Given
    let accumulator_hash = client
        .upload("accumulator", &ink_e2e::alice())
        .submit()
        .await
        .expect("uploading `accumulator` failed")
        .code_hash;

    let adder_hash = client
        .upload("adder", &ink_e2e::alice())
        .submit()
        .await
        .expect("uploading `adder` failed")
        .code_hash;

    let subber_hash = client
        .upload("subber", &ink_e2e::alice())
        .submit()
        .await
        .expect("uploading `subber` failed")
        .code_hash;

    let mut constructor = MultiContractCallerRef::new(
        1234, // initial value
        1337, // salt
        accumulator_hash,
        adder_hash,
        subber_hash,
    );

    let multi_contract_caller = client
        .instantiate("multi_contract_caller", &ink_e2e::alice(), &mut constructor)
        .value(100_000_000_000)
        .submit()
        .await
        .expect("instantiate failed");
    let mut call_builder =
        multi_contract_caller.call_builder::<MultiContractCaller>();

    // When
    let get = call_builder.get();
    let value = client
        .call(&ink_e2e::bob(), &get)
        .dry_run()
        .await?
        .return_value();
    assert_eq!(value, 1234);
    
    let change = call_builder.change(6);
    let _ = client
        .call(&ink_e2e::bob(), &change)
        .submit()
        .await
        .expect("calling `change` failed");

    // Then
    let get = call_builder.get();
    let value = client
        .call(&ink_e2e::bob(), &get)
        .dry_run()
        .await?
        .return_value();
    assert_eq!(value, 1234 + 6);

    // When
    let switch = call_builder.switch();
    let _ = client
        .call(&ink_e2e::bob(), &switch)
        .submit()
        .await
        .expect("calling `switch` failed");
        
    let change = call_builder.change(3);
    let _ = client
        .call(&ink_e2e::bob(), &change)
        .submit()
        .await
        .expect("calling `change` failed");

    // Then
    let get = call_builder.get();
    let value = client
        .call(&ink_e2e::bob(), &get)
        .dry_run()
        .await?
        .return_value();
    assert_eq!(value, 1234 + 6 - 3);

    Ok(())
}