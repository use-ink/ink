use super::Error;
use super::fallible_setter::*;

#[ink::test]
fn it_works() {
    // given
    let mut fallible_setter = FallibleSetter::new(0).expect("init failed");
    assert_eq!(fallible_setter.get(), 0);

    // when
    let res = fallible_setter.try_set(1);
    assert!(res.is_ok());

    // when: trying to set same value
    let res = fallible_setter.try_set(1);
    assert_eq!(res, Err(Error::NoChange));

    // when: trying to set value > 100
    let res = fallible_setter.try_set(101);
    assert_eq!(res, Err(Error::TooLarge));

    // then
    assert_eq!(fallible_setter.get(), 1);
}

#[cfg(all(test, feature = "e2e-tests"))]
mod e2e_tests {
    use super::*;
    use ink_e2e::ContractsBackend;

    type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

    #[ink_e2e::test]
    async fn it_works(mut client: Client) -> E2EResult<()> {
        // given
        let mut constructor = FallibleSetterRef::new(0);
        let contract = client
            .instantiate("fallible_setter", &ink_e2e::bob(), &mut constructor)
            .submit()
            .await
            .expect("instantiate failed");
        let mut call_builder = contract.call_builder::<FallibleSetter>();

        let get = call_builder.get();
        let get_res = client.call(&ink_e2e::bob(), &get).submit().await?;
        assert_eq!(get_res.return_value(), 0);

        // when
        let set = call_builder.try_set(1);
        let set_res = client
            .call(&ink_e2e::bob(), &set)
            .submit()
            .await
            .expect("set failed");
        assert!(set_res.return_value().is_ok());

        // when: trying to set same value (should fail)
        let set = call_builder.try_set(1);
        let set_res = client.call(&ink_e2e::bob(), &set).submit().await;
        // In this specific e2e environment configuration, the contract returning Result::Err
        // is surfacing as a CallExtrinsic error.
        assert!(matches!(set_res, Err(ink_e2e::Error::CallExtrinsic(_, _))));

        // when: trying to set value > 100 (should fail)
        let set = call_builder.try_set(101);
        let set_res = client.call(&ink_e2e::bob(), &set).submit().await;
        assert!(matches!(set_res, Err(ink_e2e::Error::CallExtrinsic(_, _))));

        // then
        let get = call_builder.get();
        let get_res = client.call(&ink_e2e::bob(), &get).dry_run().await?;
        assert_eq!(get_res.return_value(), 1);

        Ok(())
    }
}