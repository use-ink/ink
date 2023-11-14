use crate::get_contract_node_bin;
use zombienet_sdk::{
    NetworkConfig,
    NetworkConfigBuilder,
    RegistrationStrategy,
};

/// Build a network configuration with the default environment variables.
pub fn build_network_with_env_or_default() -> NetworkConfig {
    let missing_polkadot_binaries = [
        "polkadot",
        "polkadot-execute-worker",
        "polkadot-prepare-worker",
    ]
    .into_iter()
    .filter(|binary| which::which(binary).is_err())
    .collect::<Vec<_>>();

    if !missing_polkadot_binaries.is_empty() {
        panic!("The following binaries were not found in the PATH: {missing_polkadot_binaries:?}");
    }

    NetworkConfigBuilder::new()
        .with_relaychain(|r| {
            r.with_chain("rococo-local")
                .with_default_command("polkadot")
                .with_node(|node| node.with_name("alice"))
                .with_node(|node| node.with_name("bob"))
        })
        .with_parachain(|p| {
            p.with_id(100)
                .with_registration_strategy(RegistrationStrategy::InGenesis)
                .with_chain("contracts-parachain-local")
                .cumulus_based(true)
                .with_collator(|n| {
                    n.with_name("contract-collator")
                        .with_command(get_contract_node_bin().as_str())
                })
        })
        .build()
        .unwrap()
}
