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
    BuildArtifacts,
    BuildMode,
    ExecuteArgs,
    Features,
    ImageVariant,
    ManifestPath,
    Network,
    OptimizationPasses,
    OutputType,
    Target,
    UnstableFlags,
    Verbosity,
    DEFAULT_MAX_MEMORY_PAGES,
};
use std::{
    collections::{
        hash_map::Entry,
        HashMap,
    },
    path::{
        Path,
        PathBuf,
    },
    sync::{
        Mutex,
        OnceLock,
    },
};

/// Builds the "root" contract (the contract in which the E2E tests are defined) together
/// with the additional contracts specified in the `additional_contracts` argument.
pub fn build_root_and_additional_contracts<P>(
    additional_contracts: impl IntoIterator<Item = P>,
) -> Vec<PathBuf>
where
    PathBuf: From<P>,
{
    let contract_project = ContractProject::new();
    let contract_manifests =
        contract_project.root_with_additional_contracts(additional_contracts);
    build_contracts(&contract_manifests)
}

/// Builds the "root" contract (the contract in which the E2E tests are defined) together
/// with any contracts which are a dependency of the root contract.
pub fn build_root_and_contract_dependencies() -> Vec<PathBuf> {
    let contract_project = ContractProject::new();
    let contract_manifests = contract_project.root_with_contract_dependencies();
    build_contracts(&contract_manifests)
}

/// Access manifest paths of contracts which are part of the project in which the E2E
/// tests are defined.
struct ContractProject {
    root_package: Option<PathBuf>,
    contract_dependencies: Vec<PathBuf>,
}

impl ContractProject {
    fn new() -> Self {
        let cmd = cargo_metadata::MetadataCommand::new();
        let metadata = cmd
            .exec()
            .unwrap_or_else(|err| panic!("Error invoking `cargo metadata`: {}", err));

        fn maybe_contract_package(package: &cargo_metadata::Package) -> Option<PathBuf> {
            package
                .features
                .iter()
                .any(|(feat, _)| feat == "ink-as-dependency")
                .then(|| package.manifest_path.clone().into_std_path_buf())
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

        let contract_dependencies: Vec<PathBuf> = metadata
            .packages
            .iter()
            .filter_map(maybe_contract_package)
            .collect();

        Self {
            root_package,
            contract_dependencies,
        }
    }

    fn root_with_additional_contracts<P>(
        &self,
        additional_contracts: impl IntoIterator<Item = P>,
    ) -> Vec<PathBuf>
    where
        PathBuf: From<P>,
    {
        let mut all_manifests: Vec<_> = self.root_package.iter().cloned().collect();
        let mut additional_contracts: Vec<_> = additional_contracts
            .into_iter()
            .map(PathBuf::from)
            .collect();
        all_manifests.append(&mut additional_contracts);
        all_manifests
    }

    fn root_with_contract_dependencies(&self) -> Vec<PathBuf> {
        self.root_with_additional_contracts(&self.contract_dependencies)
    }
}

/// Build all the of the contracts of the supplied `contract_manifests`.
///
/// Only attempts to build a contract at the given path once only per test run, to avoid
/// the attempt for different tests to build the same contract concurrently.
fn build_contracts(contract_manifests: &[PathBuf]) -> Vec<PathBuf> {
    static CONTRACT_BUILD_JOBS: OnceLock<Mutex<HashMap<PathBuf, PathBuf>>> =
        OnceLock::new();
    let mut contract_build_jobs = CONTRACT_BUILD_JOBS
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
        .unwrap();

    let mut wasm_paths = Vec::new();
    for manifest in contract_manifests {
        let wasm_path = match contract_build_jobs.entry(manifest.clone()) {
            Entry::Occupied(entry) => entry.get().clone(),
            Entry::Vacant(entry) => {
                let wasm_path = build_contract(manifest);
                entry.insert(wasm_path.clone());
                wasm_path
            }
        };
        wasm_paths.push(wasm_path);
    }
    wasm_paths
}

/// Builds the contract at `manifest_path`, returns the path to the contract
/// Wasm build artifact.
fn build_contract(path_to_cargo_toml: &Path) -> PathBuf {
    let manifest_path = ManifestPath::new(path_to_cargo_toml).unwrap_or_else(|err| {
        panic!(
            "Invalid manifest path {}: {err}",
            path_to_cargo_toml.display()
        )
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
        dylint: false,
        output_type: OutputType::HumanReadable,
        skip_wasm_validation: false,
        target: Target::Wasm,
        max_memory_pages: DEFAULT_MAX_MEMORY_PAGES,
        image: ImageVariant::Default,
    };

    match contract_build::execute(args) {
        Ok(build_result) => {
            build_result
                .dest_wasm
                .expect("Wasm code artifact not generated")
                .canonicalize()
                .expect("Invalid dest bundle path")
        }
        Err(err) => {
            panic!(
                "contract build for {} failed: {err}",
                path_to_cargo_toml.display()
            )
        }
    }
}
