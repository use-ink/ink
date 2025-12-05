use super::lazyvec::{LazyVector, LazyVectorRef};
use ink_e2e::ContractsBackend;

type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[ink_e2e::test]
async fn create_and_vote(mut client: Client) -> E2EResult<()> {
    // Given
    let mut constructor = LazyVectorRef::default();
    let contract = client
        .instantiate("lazyvec", &ink_e2e::alice(), &mut constructor)
        .submit()
        .await
        .expect("instantiate failed");
    let mut call_builder = contract.call_builder::<LazyVector>();

    // When
    let create = call_builder.create_proposal(vec![0x41], 15, 1);
    let _ = client
        .call(&ink_e2e::alice(), &create)
        .submit()
        .await
        .expect("Calling `create_proposal` failed");

    let approve = call_builder.approve();
    let _ = client
        .call(&ink_e2e::alice(), &approve)
        .submit()
        .await
        .expect("Voting failed");
    let _ = client
        .call(&ink_e2e::bob(), &approve)
        .submit()
        .await
        .expect("Voting failed");

    // Then
    let value = client
        .call(&ink_e2e::alice(), &create)
        .dry_run()
        .await
        .expect("create trapped when it shouldn't")
        .return_value();
    assert_eq!(value, None);

    let value = client
        .call(&ink_e2e::alice(), &call_builder.get(0))
        .dry_run()
        .await
        .expect("get trapped when it shouldn't")
        .return_value();
    
    // We can access .approvals here because we made the struct fields pub in lib.rs
    assert_eq!(value.unwrap().approvals, 2);

    Ok(())
}