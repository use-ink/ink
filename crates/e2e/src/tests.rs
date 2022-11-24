#[tokio::test]
#[should_panic(
    expected = "Error establishing connection to a node at ws://0.0.0.0:9944. Make sure you run a node behind the given url!"
)]
async fn fail_initialization_with_no_node() {
    let _ = crate::Client::<crate::PolkadotConfig, ink_env::DefaultEnvironment>::new(
        "ws://0.0.0.0:9944",
        [],
    )
    .await;
}
