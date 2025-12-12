use super::*;
use virtual_contract::VirtualContractRef;
use virtual_contract_ver1::VirtualContractVer1Ref;
use virtual_contract_ver2::VirtualContractVer2Ref;

fn instantiate_contract1(
    contract: &ContractTester,
    code_hash: H256,
    salt: u32,
) -> Address {
    let cr = contract.instantiate_contract1(code_hash, salt);
    ink::ToAddr::to_addr(&cr)
}

fn instantiate_contract2(
    contract: &ContractTester,
    code_hash: H256,
    salt: u32,
) -> Address {
    let cr = contract.instantiate_contract2(code_hash, salt);
    ink::ToAddr::to_addr(&cr)
}

#[ink::test]
fn test_invoke() {
    let contract = ContractTester::new();
    let code_hash1 = ink::env::test::upload_code::<
        ink::env::DefaultEnvironment,
        Contract1Ref,
    >();
    let code_hash2 = ink::env::test::upload_code::<
        ink::env::DefaultEnvironment,
        Contract2Ref,
    >();

    let contract1_address1 = instantiate_contract1(&contract, code_hash1, 1);
    let contract1_address2 = instantiate_contract1(&contract, code_hash1, 2);
    let contract2_address1 = instantiate_contract2(&contract, code_hash2, 3);
    let contract2_address2 = instantiate_contract2(&contract, code_hash2, 4);

    let check_hashes = |a, b, c| {
        let x = ink::env::code_hash(a).expect("failed to get code hash");
        let y = ink::env::code_hash(b).expect("failed to get code hash");

        assert_eq!(x, c);
        assert_eq!(y, c);
    };
    check_hashes(&contract1_address1, &contract1_address2, code_hash1);
    check_hashes(&contract2_address1, &contract2_address2, code_hash2);

    let check_values1 = |a, b| {
        let x = contract.contract1_get_x(contract1_address1);
        let y = contract.contract1_get_x(contract1_address2);
        assert_eq!(x, a);
        assert_eq!(y, b);
    };
    let check_values2 = |a, b| {
        let x = contract.contract2_get_x(contract2_address1);
        let y = contract.contract2_get_x(contract2_address2);
        assert_eq!(x, a);
        assert_eq!(y, b);
    };

    check_values1(42, 42);
    check_values2(123456, 123456);
    contract.contract2_set_x(contract2_address1, 78);
    check_values1(42, 42);
    check_values2(123456, 123456);
    contract.contract1_set_x(contract1_address1, 123);
    contract.contract2_set_x(contract2_address2, 87);
    check_values1(123, 42);
    check_values2(123456, 123456);
    contract.contract1_set_x(contract1_address2, 321);
    check_values1(123, 321);
    check_values2(123456, 123456);
}

#[ink::test]
fn test_invoke_delegate() {
    let code_hash1 = ink::env::test::upload_code::<
        ink::env::DefaultEnvironment,
        VirtualContractRef,
    >();
    let code_hash2 = ink::env::test::upload_code::<
        ink::env::DefaultEnvironment,
        VirtualContractVer1Ref,
    >();
    let code_hash3 = ink::env::test::upload_code::<
        ink::env::DefaultEnvironment,
        VirtualContractVer2Ref,
    >();

    let create_params = build_create::<VirtualContractVer1Ref>()
        .code_hash(code_hash2)
        .endowment(0.into())
        .exec_input(ExecutionInput::new(Selector::new(ink::selector_bytes!(
            Abi::Ink,
            "new"
        ))))
        .returns::<VirtualContractVer1Ref>()
        .params();

    let addr2 = ink::env::instantiate_contract(&create_params)
        .unwrap_or_else(|error| {
            panic!(
                "Received an error from `pallet-revive` while instantiating: {error:?}"
            )
        })
        .unwrap_or_else(|error| {
            panic!("Received a `LangError` while instantiating: {error:?}")
        });

    use ink::ToAddr;
    let addr2: Address = addr2.to_addr();

    let create_params = build_create::<VirtualContractVer2Ref>()
        .code_hash(code_hash3)
        .endowment(0.into())
        .exec_input(ExecutionInput::new(Selector::new(ink::selector_bytes!(
            Abi::Ink,
            "new"
        ))))
        .returns::<VirtualContractVer2Ref>()
        .params();

    let addr3 = ink::env::instantiate_contract(&create_params)
        .unwrap_or_else(|error| {
            panic!(
                "Received an error from `pallet-revive` while instantiating: {error:?}"
            )
        })
        .unwrap_or_else(|error| {
            panic!("Received a `LangError` while instantiating: {error:?}")
        });
    let addr3: Address = addr3.to_addr();

    // creates `code_hash1` contract and puts `hash` + `x` as the constructor
    // arguments
    let instantiate = |delegate_addr: Address, salt: u32, x| {
        let mut salt_bytes = [0u8; 32];
        salt_bytes[..4].copy_from_slice(&salt.to_le_bytes());
        let create_params = build_create::<VirtualContractRef>()
            .code_hash(code_hash1)
            .endowment(0.into())
            .exec_input(
                ExecutionInput::new(Selector::new(ink::selector_bytes!(Abi::Ink, "new")))
                    //.push_arg(H256::zero()) // todo should result in err, but doesn't
                    .push_arg(delegate_addr)
                    .push_arg(x),
            )
            .salt_bytes(Some(salt_bytes))
            .returns::<VirtualContractRef>()
            .params();

        ink::env::instantiate_contract(&create_params)
            .unwrap_or_else(|error| {
                panic!(
                    "Received an error from `pallet-revive` while instantiating: {error:?}"
                )
            })
            .unwrap_or_else(|error| {
                panic!("Received a `LangError` while instantiating: {error:?}")
            })
    };

    let mut ref1 = instantiate(addr2, 1, 42);
    let mut ref2 = instantiate(addr3, 2, 74);

    let check_values =
        |r1: &VirtualContractRef, r2: &VirtualContractRef, a, b, c, d| {
            let v1 = r1.real_get_x();
            let v2 = r2.real_get_x();
            let v3 = r1.get_x();
            let v4 = r2.get_x();
            assert_eq!(v1, a);
            assert_eq!(v2, b);
            assert_eq!(v3, c);
            assert_eq!(v4, d);
        };

    check_values(&ref1, &ref2, 42, 74, 43, 148);
    ref1.set_x(15);
    check_values(&ref1, &ref2, 42, 74, 43, 148);
    ref1.real_set_x(15);
    check_values(&ref1, &ref2, 15, 74, 16, 148);
    ref2.set_x(39);
    check_values(&ref1, &ref2, 15, 74, 16, 148);
    ref2.real_set_x(39);
    check_values(&ref1, &ref2, 15, 39, 16, 78);
}
