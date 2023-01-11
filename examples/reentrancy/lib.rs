#[cfg(test)]
mod test {
    use fallback_contract::{
        FallbackContract,
        FallbackContractRef,
    };
    use ink::primitives::Hash;
    use main_contract::{
        MainContract,
        MainContractRef,
    };

    #[test]
    pub fn reentrancy_works() {
        let hash1 = Hash::from([10u8; 32]);
        let hash2 = Hash::from([20u8; 32]);

        ink::env::test::register_contract::<MainContract>(hash1.as_ref());
        ink::env::test::register_contract::<FallbackContract>(hash2.as_ref());

        let mut main_contract = MainContractRef::new()
            .code_hash(hash1.clone())
            .endowment(0)
            .salt_bytes([0u8; 0])
            .instantiate()
            .expect("failed at instantiating the `main_contractRef` contract");

        let fallback_contract = FallbackContractRef::new(main_contract.clone())
            .code_hash(hash2.clone())
            .endowment(0)
            .salt_bytes([0u8; 0])
            .instantiate()
            .expect("failed at instantiating the `fallback_contractRef` contract");

        let address1 = main_contract.get_address();

        let address2 = fallback_contract.get_address();

        main_contract.set_callee(address2);

        assert_eq!(main_contract.get_callee(), address2);
        assert_eq!(fallback_contract.get_callee(), address1);

        assert_eq!(main_contract.inc(), Ok(2));
        assert_eq!(main_contract.get(), 2);
        assert_eq!(fallback_contract.get(), 0);
    }
}
