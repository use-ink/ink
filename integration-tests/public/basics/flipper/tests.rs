use super::flipper::*;

#[ink::test]
fn it_works() {
    let mut flipper = Flipper::new(false);
    assert!(!flipper.get());
    flipper.flip();
    assert!(flipper.get());
}

#[cfg(all(test, feature = "e2e-tests"))]
mod e2e_tests {
    use super::*;
    use ink_e2e::ContractsBackend;

    type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

    #[ink_e2e::test]
    async fn it_works(mut client: Client) -> E2EResult<()> {
        // given
        let mut constructor = FlipperRef::new(false);
        let contract = client
            .instantiate("flipper", &ink_e2e::bob(), &mut constructor)
            .submit()
            .await
            .expect("instantiate failed");
        let mut call_builder = contract.call_builder::<Flipper>();

        let get = call_builder.get();
        let get_res = client.call(&ink_e2e::bob(), &get).submit().await?;
        assert!(!get_res.return_value());

        // when
        let flip = call_builder.flip();
        let _flip_res = client
            .call(&ink_e2e::bob(), &flip)
            .submit()
            .await
            .expect("flip failed");

        // then
        let get = call_builder.get();
        let get_res = client.call(&ink_e2e::bob(), &get).dry_run().await?;
        assert!(get_res.return_value());

        Ok(())
    }

    /// This test illustrates how to test an existing on-chain contract.
    ///
    /// You can utilize this to e.g. create a snapshot of a production chain
    /// and run the E2E tests against a deployed contract there.
    /// This process is explained [here](https://use.ink/5.x/basics/contract-testing/chain-snapshot).
    ///
    /// Before executing the test:
    ///   * Make sure you have a node running in the background,
    ///   * Supply the environment variable `CONTRACT_HEX` that points to a deployed
    ///     flipper contract. You can take the SS58 address which `cargo contract
    ///     instantiate` gives you and convert it to hex using `subkey inspect
    ///     <SS58>`.
    ///
    /// The test is then run like this:
    ///
    /// ```
    /// # The env variable needs to be set, otherwise `ink_e2e` will spawn a new
    /// # node process for each test.
    /// $ export CONTRACTS_NODE_URL=ws://127.0.0.1:9944
    ///
    /// $ export CONTRACT_ADDR_HEX=0x2c75f0aa09dbfbfd49e6286a0f2edd3b4913f04a58b13391c79e96782f5713e3
    /// $ cargo test --features e2e-tests e2e_test_deployed_contract -- --ignored
    /// ```
    ///
    /// # Developer Note
    ///
    /// The test is marked as ignored, as it has the above pre-conditions to succeed.
    #[ink_e2e::test]
    #[ignore]
    async fn e2e_test_deployed_contract(mut client: Client) -> E2EResult<()> {
        // given
        use ink::Address;
        let addr = std::env::var("CONTRACT_ADDR_HEX")
            .unwrap()
            .replace("0x", "");
        let addr_bytes: Vec<u8> = hex::decode(addr).unwrap();
        let addr = Address::from_slice(&addr_bytes[..]);

        use std::str::FromStr;
        let suri = ink_e2e::subxt_signer::SecretUri::from_str("//Alice").unwrap();
        let caller = ink_e2e::Keypair::from_uri(&suri).unwrap();

        // when
        // Invoke `Flipper::get()` from `caller`'s account
        let call_builder = ink_e2e::create_call_builder::<Flipper>(addr);
        let get = call_builder.get();
        let get_res = client.call(&caller, &get).dry_run().await?;

        // then
        assert!(get_res.return_value());

        Ok(())
    }
}