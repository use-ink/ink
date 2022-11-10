#[test]
#[should_panic(
    expected = "\n\nError establishing connection to a node at ws://0.0.0.0:9944\nMake sure your node is running."
)]
fn fail_initialization_with_no_node() {
    let client_init =
        crate::Client::<crate::PolkadotConfig, ink_env::DefaultEnvironment>::new(
            "ws://0.0.0.0:9944",
        );
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap_or_else(|err| panic!("Failed building the Runtime during test: {}", err))
        .block_on(client_init);
}
