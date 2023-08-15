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

// todo: [AJ] should this be two different methods or use an Option instead of checking additional_contracts.is_empty()
pub fn build_contracts<'a>(additional_contracts: impl IntoIterator<Item = &'a str>) -> Vec<String> {
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

    let mut all_manifests: Vec<String> = root_package.iter().cloned().collect();

    if additional_contracts.is_empty() {
        let mut contract_dependencies = metadata
            .packages
            .iter()
            .filter_map(maybe_contract_package)
            .collect();
        all_manifests.append(&mut contract_dependencies.clone());
    } else {
        for additional_contract in additional_contracts {
            all_manifests.push(additional_contract.to_string())
        }
    };

    all_manifests
        .all_contracts_to_build()
        .into_iter()
        .map(build_contract)
        .collect()
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