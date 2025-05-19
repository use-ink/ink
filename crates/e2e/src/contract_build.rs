// Copyright (C) Use Ink (UK) Ltd.
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

use std::{
    collections::{
        hash_map::Entry,
        HashMap,
    },
    env,
    path::{
        Path,
        PathBuf,
    },
    sync::{
        Mutex,
        OnceLock,
    },
};

use contract_build::{
    package_abi,
    util::rustc_wrapper,
    Abi,
    BuildArtifacts,
    BuildMode,
    ExecuteArgs,
    Features,
    ImageVariant,
    ManifestPath,
    MetadataSpec,
    Network,
    OutputType,
    UnstableFlags,
    Verbosity,
};
use itertools::Itertools;

use crate::log_info;

/// Builds the "root" contract (the contract in which the E2E tests are defined) together
/// with any contracts which are a dependency of the root contract.
///
/// todo explain features
pub fn build_root_and_contract_dependencies(features: Vec<String>) -> Vec<PathBuf> {
    let contract_project = ContractProject::new();
    let contract_manifests = contract_project.root_with_contract_dependencies();
    if contract_project.package_abi.is_some() {
        // Generates a custom `rustc` wrapper which passes compiler flags to `rustc`,
        // because `cargo` doesn't pass compiler flags to proc macros and build
        // scripts when the `--target` flag is set.
        // See `contract_build::util::rustc_wrapper::generate` docs for details.
        if let Ok(rustc_wrapper) = rustc_wrapper::generate(&contract_project.target_dir) {
            // SAFETY: The `rustc` wrapper is safe to reuse across all threads.
            env::set_var("INK_RUSTC_WRAPPER", rustc_wrapper);
        }
    }
    build_contracts(&contract_manifests, features)
}

/// Access manifest paths of contracts which are part of the project in which the E2E
/// tests are defined.
struct ContractProject {
    root_package: Option<PathBuf>,
    contract_dependencies: Vec<PathBuf>,
    package_abi: Option<Abi>,
    target_dir: PathBuf,
}

impl ContractProject {
    fn new() -> Self {
        let mut cmd = cargo_metadata::MetadataCommand::new();
        let env_target_dir = env::var_os("CARGO_TARGET_DIR")
            .map(|target_dir| PathBuf::from(target_dir))
            .filter(|target_dir| target_dir.is_absolute());
        if let Some(target_dir) = env_target_dir.as_ref() {
            cmd.env("CARGO_TARGET_DIR", target_dir);
        }
        let metadata = cmd
            .exec()
            .unwrap_or_else(|err| panic!("Error invoking `cargo metadata`: {}", err));

        fn maybe_contract_package(package: &cargo_metadata::Package) -> Option<PathBuf> {
            package
                .features
                .iter()
                .any(|(feat, _)| {
                    feat == "ink-as-dependency"
                        && !package.name.eq("ink")
                        && !package.name.eq("ink_env")
                })
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
        log_info(&format!("found root package: {:?}", root_package));

        let contract_dependencies: Vec<PathBuf> = metadata
            .packages
            .iter()
            .filter_map(maybe_contract_package)
            .collect();
        log_info(&format!(
            "found those contract dependencies: {:?}",
            contract_dependencies
        ));

        let package_abi = metadata
            .root_package()
            .and_then(|package| package_abi(package))
            .and_then(Result::ok);
        log_info(&format!("found root package abi: {:?}", package_abi));

        let target_dir = env_target_dir
            .unwrap_or_else(|| metadata.target_directory.into_std_path_buf());
        log_info(&format!("found target dir: {:?}", target_dir));

        Self {
            root_package,
            contract_dependencies,
            package_abi,
            target_dir,
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
        all_manifests.into_iter().unique().collect()
    }

    fn root_with_contract_dependencies(&self) -> Vec<PathBuf> {
        self.root_with_additional_contracts(&self.contract_dependencies)
    }
}

/// Build all the of the contracts of the supplied `contract_manifests`.
///
/// Only attempts to build a contract at the given path once only per test run, to avoid
/// the attempt for different tests to build the same contract concurrently.
fn build_contracts(
    contract_manifests: &[PathBuf],
    features: Vec<String>,
) -> Vec<PathBuf> {
    static CONTRACT_BUILD_JOBS: OnceLock<Mutex<HashMap<PathBuf, PathBuf>>> =
        OnceLock::new();
    let mut contract_build_jobs = CONTRACT_BUILD_JOBS
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
        .unwrap();

    let mut blob_paths = Vec::new();
    for manifest in contract_manifests {
        let contract_binary_path = match contract_build_jobs.entry(manifest.clone()) {
            Entry::Occupied(entry) => entry.get().clone(),
            Entry::Vacant(entry) => {
                let contract_binary_path = build_contract(manifest, features.clone());
                entry.insert(contract_binary_path.clone());
                contract_binary_path
            }
        };
        blob_paths.push(contract_binary_path);
    }
    blob_paths
}

/// Builds the contract at `manifest_path`, returns the path to the contract
/// PolkaVM build artifact.
fn build_contract(
    path_to_cargo_toml: &Path,
    additional_features: Vec<String>,
) -> PathBuf {
    let manifest_path = ManifestPath::new(path_to_cargo_toml).unwrap_or_else(|err| {
        panic!(
            "Invalid manifest path {}: {err}",
            path_to_cargo_toml.display()
        )
    });
    // todo add method in Features to just construct with new(features)
    let mut features = Features::default();
    additional_features.iter().for_each(|f| features.push(f));
    let args = ExecuteArgs {
        manifest_path,
        verbosity: Verbosity::Default,
        build_mode: BuildMode::Debug,
        features,
        network: Network::Online,
        build_artifact: BuildArtifacts::All,
        unstable_flags: UnstableFlags::default(),
        keep_debug_symbols: false,
        extra_lints: false,
        output_type: OutputType::HumanReadable,
        image: ImageVariant::Default,
        metadata_spec: MetadataSpec::Ink,
    };

    match contract_build::execute(args) {
        Ok(build_result) => {
            build_result
                .dest_binary
                .expect("PolkaVM code artifact not generated")
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
