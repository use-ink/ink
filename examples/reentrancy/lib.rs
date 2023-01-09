#[cfg(test)]
mod test {
    use contract1::{
        Contract1,
        Contract1Ref,
    };
    use contract2::{
        Contract2,
        Contract2Ref,
    };
    use ink::primitives::Hash;

    #[test]
    pub fn reentrancy_works() {
        let hash1 = Hash::from([10u8; 32]);
        let hash2 = Hash::from([20u8; 32]);

        ink::env::test::register_contract::<Contract1>(hash1.as_ref());
        ink::env::test::register_contract::<Contract2>(hash2.as_ref());

        let mut contract1 = Contract1Ref::new()
            .code_hash(hash1.clone())
            .endowment(0)
            .salt_bytes([0u8; 0])
            .instantiate()
            .expect("failed at instantiating the `Contract1Ref` contract");
        let mut contract2 = Contract2Ref::new(contract1.clone())
            .code_hash(hash2.clone())
            .endowment(0)
            .salt_bytes([0u8; 0])
            .instantiate()
            .expect("failed at instantiating the `Contract2Ref` contract");

        let address1 = contract1.get_address();

        let address2 = contract2.get_address();

        contract1.set_callee(address2);

        assert_eq!(contract1.get_callee(), address2);
        assert_eq!(contract2.get_callee(), address1);

        assert_eq!(contract1.inc(), 2);
    }
}
