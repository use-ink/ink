use super::fuzz_testing::{FuzzTesting, FuzzTestingRef, Point};
use ink_e2e::ContractsBackend;
use quickcheck_macros::quickcheck;
use quickcheck::{
    Arbitrary,
    Gen,
};

// We need to implement `Arbitrary` for `Point`, so `quickcheck`
// knows how to fuzz the struct.
impl Arbitrary for Point {
    fn arbitrary(g: &mut Gen) -> Point {
        Point {
            x: i32::arbitrary(g),
            y: i32::arbitrary(g),
        }
    }
}

/// We use `#[ink_e2e::test(runtime)]` here. It doesn't start a node for each
/// test, but instead interacts with an in-process `pallet-revive`.
///
/// See <https://use.ink/docs/v6/contract-testing/drink#as-an-alternative-backend-to-inks-e2e-testing-framework>
/// for more details.
#[ink_e2e::test(runtime, replace_test_attr = "#[quickcheck]")]
async fn fuzzing_works_runtime(val: bool) -> bool {
    let mut constructor = FuzzTestingRef::new(val);
    let contract = client
        .instantiate("fuzz_testing", &ink_e2e::bob(), &mut constructor)
        .submit()
        .await
        .expect("instantiate failed");
    let call_builder = contract.call_builder::<FuzzTesting>();

    let get = call_builder.get();
    let get_res = client.call(&ink_e2e::bob(), &get).submit().await.unwrap();
    get_res.return_value() == val
}

/// It's also possible to fuzz with a "real" node as the backend.
///
/// This means that, by default, for every test run a node process will
/// be spawned. You can work around this by setting the env variable
/// `CONTRACTS_NODE_URL`. But still, interactions with a real node will
/// always be more heavy-weight than "just" interacting with an in-process
/// `pallet-revive`.
#[ink_e2e::test(runtime, replace_test_attr = "#[quickcheck]")]
async fn fuzzing_works_node(val: bool) -> bool {
    let mut constructor = FuzzTestingRef::new(val);
    let contract = client
        .instantiate("fuzz_testing", &ink_e2e::bob(), &mut constructor)
        .submit()
        .await
        .expect("instantiate failed");
    let call_builder = contract.call_builder::<FuzzTesting>();

    let get = call_builder.get();
    let get_res = client.call(&ink_e2e::bob(), &get).submit().await.unwrap();
    get_res.return_value() == val
}

#[ink_e2e::test(runtime, replace_test_attr = "#[quickcheck]")]
async fn fuzzing_custom_struct_works(val: Point) -> bool {
    ink_e2e::tracing::info!("fuzzing with value {val:?}");

    let mut constructor = FuzzTestingRef::new(true);
    let contract = client
        .instantiate("fuzz_testing", &ink_e2e::bob(), &mut constructor)
        .submit()
        .await
        .expect("instantiate failed");
    let call_builder = contract.call_builder::<FuzzTesting>();

    let get = call_builder.extract_x(val.clone());
    let get_res = client.call(&ink_e2e::bob(), &get).submit().await.unwrap();
    get_res.return_value() == val.x
}