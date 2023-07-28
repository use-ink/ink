// Copyright (C) Parity Technologies (UK) Ltd.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use contract_build::{
    ManifestPath,
    Target,
    BuildArtifacts,
    BuildMode,
    ExecuteArgs,
    Features,
    Network,
    OptimizationPasses,
    OutputType,
    UnstableFlags,
    Verbosity,
};
use std::collections::HashMap;

/// Build contracts for E2E testing
pub struct ContractBuildContext {
    contracts: Vec<String>,
}

impl ContractBuildContext {
    pub fn new<'a>(additional_contracts: impl IntoIterator<Item = &'a str>) -> Self {
        let manifests =
            if additional_contracts.is_empty() {
                ContractManifests::from_cargo_metadata().all_contracts_to_build()
            } else {
                additional_contracts.into_iter().map(|path| path.to_string()).collect()
            };
        Self { manifests }
    }

    pub fn build_contracts(&self) -> HashMap<String, String> {
        self.manifests
            .all_contracts_to_build()
            .into_iter()
            .map(build_contract)
            .collect()
    }
}

#[derive(Debug)]
struct ContractManifests {
    /// The manifest path of the root package where the E2E test is defined.
    /// `None` if the root package is not an `ink!` contract definition.
    root_package: Option<String>,
    /// The manifest paths of any dependencies which are `ink!` contracts.
    contract_dependencies: Vec<String>,
}

impl ContractManifests {
    /// Load any manifests for packages which are detected to be `ink!` contracts. Any
    /// package with the `ink-as-dependency` feature enabled is assumed to be an
    /// `ink!` contract.
    fn from_cargo_metadata() -> Self {
        let cmd = cargo_metadata::MetadataCommand::new();
        let metadata = cmd
            .exec()
            .unwrap_or_else(|err| panic!("Error invoking `cargo metadata`: {}", err));

        fn maybe_contract_package(package: &cargo_metadata::Package) -> Option<String> {
            package
                .features
                .iter()
                .any(|(feat, _)| feat == "ink-as-dependency")
                .then(|| package.manifest_path.to_string())
        }

        let root_package = metadata
            .resolve
            .as_ref()
            .and_then(|resolve| resolve.root.as_ref())
            .and_then(|root_package_id| {
                metadata
                    .packages
                    .iter()
                    .find(|package| &package.id == root_package_id)
            })
            .and_then(maybe_contract_package);

        let contract_dependencies = metadata
            .packages
            .iter()
            .filter_map(maybe_contract_package)
            .collect();

        Self {
            root_package,
            contract_dependencies,
        }
    }

    /// Returns all the contract manifests which are to be built, including the root
    /// package if it is determined to be an `ink!` contract.
    fn all_contracts_to_build(&self) -> Vec<String> {
        let mut all_manifests: Vec<String> = self.root_package.iter().cloned().collect();
        all_manifests.append(&mut self.contract_dependencies.clone());
        all_manifests
    }
}

/// Builds the contract at `manifest_path`, returns the path to the contract
/// Wasm build artifact.
fn build_contract(path_to_cargo_toml: &str) -> String {
    let manifest_path = ManifestPath::new(path_to_cargo_toml).unwrap_or_else(|err| {
        panic!("Invalid manifest path {path_to_cargo_toml}: {err}")
    });
    let args = ExecuteArgs {
        manifest_path,
        verbosity: Verbosity::Default,
        build_mode: BuildMode::Debug,
        features: Features::default(),
        network: Network::Online,
        build_artifact: BuildArtifacts::CodeOnly,
        unstable_flags: UnstableFlags::default(),
        optimization_passes: Some(OptimizationPasses::default()),
        keep_debug_symbols: false,
        lint: false,
        output_type: OutputType::HumanReadable,
        skip_wasm_validation: false,
        target: Target::Wasm,
        ..ExecuteArgs::default()
    };

    match contract_build::execute(args) {
        Ok(build_result) => {
            build_result
                .dest_wasm
                .expect("Wasm code artifact not generated")
                .canonicalize()
                .expect("Invalid dest bundle path")
                .to_string_lossy()
                .into()
        }
        Err(err) => {
            panic!("contract build for {path_to_cargo_toml} failed: {err}")
        }
    }
}