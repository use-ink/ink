use super::custom_allocator::*;

#[ink::test]
fn default_works() {
    let custom_allocator = CustomAllocator::default();
    assert!(!custom_allocator.get());
}

#[ink::test]
fn it_works() {
    let mut custom_allocator = CustomAllocator::new(false);
    assert!(!custom_allocator.get());
    custom_allocator.flip();
    assert!(custom_allocator.get());
}

#[cfg(all(test, feature = "e2e-tests"))]
mod e2e_tests {
    use super::*;
    use ink_e2e::ContractsBackend;

    type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

    /// We test that we can upload and instantiate the contract using its default
    /// constructor.
    #[ink_e2e::test]
    async fn default_works(mut client: Client) -> E2EResult<()> {
        // Given
        let mut constructor = CustomAllocatorRef::default();

        // When
        let contract = client
            .instantiate("custom_allocator", &ink_e2e::alice(), &mut constructor)
            .submit()
            .await
            .expect("instantiate failed");
        let call_builder = contract.call_builder::<CustomAllocator>();

        // Then
        let get = call_builder.get();
        let get_result = client.call(&ink_e2e::alice(), &get).dry_run().await?;
        assert!(!get_result.return_value());

        Ok(())
    }

    /// We test that we can read and write a value from the on-chain contract.
    #[ink_e2e::test]
    async fn it_works(mut client: Client) -> E2EResult<()> {
        // Given
        let mut constructor = CustomAllocatorRef::new(false);
        let contract = client
            .instantiate("custom_allocator", &ink_e2e::bob(), &mut constructor)
            .submit()
            .await
            .expect("instantiate failed");
        let mut call_builder = contract.call_builder::<CustomAllocator>();

        let get = call_builder.get();
        let get_result = client.call(&ink_e2e::bob(), &get).dry_run().await?;
        assert!(!get_result.return_value());

        // When
        let flip = call_builder.flip();
        let _flip_result = client
            .call(&ink_e2e::bob(), &flip)
            .submit()
            .await
            .expect("flip failed");

        // Then
        let get = call_builder.get();
        let get_result = client.call(&ink_e2e::bob(), &get).dry_run().await?;
        assert!(get_result.return_value());

        Ok(())
    }
}