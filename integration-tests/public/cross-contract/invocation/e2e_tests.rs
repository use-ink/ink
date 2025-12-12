use super::*;
use ink_e2e::{
    ChainBackend,
    ContractsBackend,
    E2EBackend,
    InstantiationResult,
};
use virtual_contract::{
    VirtualContractRef,
    virtual_contract::VirtualContract,
};
use virtual_contract_ver1::VirtualContractVer1Ref;
use virtual_contract_ver2::VirtualContractVer2Ref;

type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

async fn check_values<Client, E, B>(
    origin: &ink_e2e::Keypair,
    client: &mut Client,

    ver1: &InstantiationResult<E, B, ink::env::DefaultAbi>,
    ver2: &InstantiationResult<E, B, ink::env::DefaultAbi>,

    a: u32,
    b: u32,
    c: u32,
    d: u32,
) where
    Client: E2EBackend<ink::env::DefaultEnvironment>,
    E: ink::env::Environment,
{
    let r1 = ver1.call_builder::<VirtualContract>();
    let r2 = ver2.call_builder::<VirtualContract>();

    let v1_get = r1.real_get_x();
    let v1 = client
        .call(&origin, &v1_get)
        .dry_run()
        .await
        .unwrap_or_else(|_| panic!("foo"))
        .return_value();

    let v2_get = r2.real_get_x();
    let v2 = client
        .call(&origin, &v2_get)
        .dry_run()
        .await
        .unwrap_or_else(|_| panic!("foo"))
        .return_value();

    let v3_get = r1.get_x();
    let v3 = client
        .call(&origin, &v3_get)
        .dry_run()
        .await
        .unwrap_or_else(|_| panic!("foo"))
        .return_value();

    let v4_get = r2.get_x();
    let v4 = client
        .call(&origin, &v4_get)
        .dry_run()
        .await
        .unwrap_or_else(|_| panic!("foo"))
        .return_value();

    assert_eq!(v1, a);
    assert_eq!(v2, b);
    assert_eq!(v3, c);
    assert_eq!(v4, d);
}

#[ink_e2e::test]
async fn test_invoke_delegate_e2e(mut client: Client) -> E2EResult<()> {
    let origin = client
        .create_and_fund_account(&ink_e2e::alice(), 10_000_000_000_000)
        .await;

    let mut constructor = VirtualContractVer1Ref::new();
    let addr_virtual_ver1 = client
        .instantiate("virtual_contract_ver1", &origin, &mut constructor)
        .submit()
        .await
        .expect("instantiate `delegatee` failed");

    let mut constructor = VirtualContractVer2Ref::new();
    let addr_virtual_ver2 = client
        .instantiate("virtual_contract_ver2", &origin, &mut constructor)
        .submit()
        .await
        .expect("instantiate `delegatee` failed");

    let mut constructor = VirtualContractRef::new(addr_virtual_ver1.addr, 42);
    let ver1 = client
        .instantiate("virtual_contract", &origin, &mut constructor)
        .submit()
        .await
        .expect("instantiate `delegatee` failed");

    let mut constructor = VirtualContractRef::new(addr_virtual_ver2.addr, 74);
    let ver2 = client
        .instantiate("virtual_contract", &origin, &mut constructor)
        .submit()
        .await
        .expect("instantiate `delegatee` failed");

    // contract code_hash1 with argument code_hash2
    check_values(&origin, &mut client, &ver1, &ver2, 42, 74, 43, 148).await;

    let mut call_builder = ver1.call_builder::<VirtualContract>();
    let call = call_builder.set_x(15);
    let _call_result = client
        .call(&origin, &call)
        .submit()
        .await
        .expect("Calling `call_builder::call` failed");

    check_values(&origin, &mut client, &ver1, &ver2, 42, 74, 43, 148).await;

    let mut call_builder = ver1.call_builder::<VirtualContract>();
    let call = call_builder.real_set_x(15);
    let _call_result = client
        .call(&origin, &call)
        .submit()
        .await
        .expect("Calling `call_builder::call` failed");

    check_values(&origin, &mut client, &ver1, &ver2, 15, 74, 16, 148).await;

    let mut call_builder = ver2.call_builder::<VirtualContract>();
    let call = call_builder.set_x(39);
    let _call_result = client
        .call(&origin, &call)
        .submit()
        .await
        .expect("Calling `call_builder::call` failed");

    check_values(&origin, &mut client, &ver1, &ver2, 15, 74, 16, 148).await;

    let mut call_builder = ver2.call_builder::<VirtualContract>();
    let call = call_builder.real_set_x(39);
    let _call_result = client
        .call(&origin, &call)
        .submit()
        .await
        .expect("Calling `call_builder::call` failed");

    check_values(&origin, &mut client, &ver1, &ver2, 15, 39, 16, 78).await;

    Ok(())
}
