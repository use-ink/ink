#[test]
#[should_panic(
    expected = "Error establishing connection to a node at ws://0.0.0.0:9944. Make sure you run a node behind the given url!"
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
