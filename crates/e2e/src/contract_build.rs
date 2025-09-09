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
        HashMap,
        hash_map::Entry,
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

use crate::log_info;
use contract_build::{
    BuildArtifacts,
    BuildMode,
    ExecuteArgs,
    Features,
    ImageVariant,
    ManifestPath,
    Network,
    OutputType,
    UnstableFlags,
    Verbosity,
};
use itertools::Itertools;

/// Builds the "root" contract (the contract in which the E2E tests are defined) together
/// with any contracts which are a dependency of the root contract.
///
/// Builds the root contract with `features`.
pub fn build_root_and_contract_dependencies(features: Vec<String>) -> Vec<PathBuf> {
    let contract_project = ContractProject::new();
    let contract_manifests_and_features =
        contract_project.root_with_contract_dependencies(features);
    build_contracts(
        &contract_manifests_and_features,
        contract_project.target_dir,
    )
}

/// Access manifest paths of contracts which are part of the project in which the E2E
/// tests are defined.
struct ContractProject {
    root_package: Option<PathBuf>,
    contract_dependencies: Vec<PathBuf>,
    target_dir: PathBuf,
}

impl ContractProject {
    fn new() -> Self {
        let mut cmd = cargo_metadata::MetadataCommand::new();
        let env_target_dir = env::var_os("CARGO_TARGET_DIR")
            .map(PathBuf::from)
            .filter(|target_dir| target_dir.is_absolute());
        if let Some(target_dir) = env_target_dir.as_ref() {
            cmd.env("CARGO_TARGET_DIR", target_dir);
        }
        let metadata = cmd
            .exec()
            .unwrap_or_else(|err| panic!("Error invoking `cargo metadata`: {err}"));

        fn maybe_contract_package(package: &cargo_metadata::Package) -> Option<PathBuf> {
            package
                .features
                .iter()
                .any(|(feat, _)| {
                    feat == "ink-as-dependency"
                        && !package.name.as_str().eq("ink")
                        && !package.name.as_str().eq("ink_env")
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
        log_info(&format!("found root package: {root_package:?}"));

        let contract_dependencies: Vec<PathBuf> = metadata
            .packages
            .iter()
            .filter_map(maybe_contract_package)
            .collect();
        log_info(&format!(
            "found those contract dependencies: {contract_dependencies:?}"
        ));

        let target_dir = env_target_dir
            .unwrap_or_else(|| metadata.target_directory.into_std_path_buf());
        log_info(&format!("found target dir: {target_dir:?}"));

        Self {
            root_package,
            contract_dependencies,
            target_dir,
        }
    }

    fn root_with_additional_contracts<P>(
        &self,
        additional_contracts: impl IntoIterator<Item = P>,
        features: Vec<String>,
    ) -> Vec<(PathBuf, Vec<String>)>
    where
        PathBuf: From<P>,
    {
        let mut all_manifests: Vec<_> = self
            .root_package
            .iter()
            .cloned()
            .map(|path| (path, features.clone()))
            .collect();
        let mut additional_contracts: Vec<_> = additional_contracts
            .into_iter()
            .map(PathBuf::from)
            .map(|path| (path, vec![]))
            .collect();
        all_manifests.append(&mut additional_contracts);
        all_manifests.into_iter().unique().collect()
    }

    fn root_with_contract_dependencies(
        &self,
        features: Vec<String>,
    ) -> Vec<(PathBuf, Vec<String>)> {
        self.root_with_additional_contracts(&self.contract_dependencies, features)
    }
}

/// Build all contracts of the supplied `contract_manifests`.
///
/// Only attempts to build a contract at the given path once only per test run, to avoid
/// the attempt for different tests to build the same contract concurrently.
fn build_contracts(
    contract_manifests: &[(PathBuf, Vec<String>)],
    target_dir: PathBuf,
) -> Vec<PathBuf> {
    #[allow(clippy::type_complexity)]
    static CONTRACT_BUILD_JOBS: OnceLock<
        Mutex<HashMap<(PathBuf, Vec<String>), PathBuf>>,
    > = OnceLock::new();
    let mut contract_build_jobs = CONTRACT_BUILD_JOBS
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
        .unwrap();

    let mut blob_paths = Vec::new();
    for (manifest, features) in contract_manifests {
        let key = (manifest.clone(), features.clone());
        let contract_binary_path = match contract_build_jobs.entry(key) {
            Entry::Occupied(entry) => entry.get().clone(),
            Entry::Vacant(entry) => {
                let contract_binary_path =
                    build_contract(manifest, features.clone(), target_dir.clone());
                let path_with_features =
                    add_features_to_filename(contract_binary_path, features);
                entry.insert(path_with_features.clone());
                path_with_features
            }
        };
        blob_paths.push(contract_binary_path);
    }
    blob_paths
}

fn add_features_to_filename(
    contract_binary_path: PathBuf,
    features: &[String],
) -> PathBuf {
    // add features to file name
    let mut path_with_features = contract_binary_path.clone();
    let filename = path_with_features
        .file_stem()
        .expect("no file name")
        .to_string_lossy()
        .into_owned();
    let extension = path_with_features
        .extension()
        .expect("no file name")
        .to_string_lossy()
        .into_owned();
    path_with_features.pop();

    let features_str = features.join("-");
    let mut new_filename =
        format!("{}-features-{}", filename, features_str.replace("/", "-"));
    if features.is_empty() {
        new_filename.push_str("no");
    }
    new_filename.push_str(&format!(".{extension}"));
    path_with_features.push(new_filename);
    std::fs::copy(contract_binary_path, path_with_features.as_path())
        .expect("failed copying binary");
    path_with_features
}

/// Builds the contract at `manifest_path`, returns the path to the contract
/// PolkaVM build artifact.
fn build_contract(
    cargo_toml: &Path,
    features: Vec<String>,
    target_dir: PathBuf,
) -> PathBuf {
    let manifest_path = ManifestPath::new(cargo_toml).unwrap_or_else(|err| {
        panic!("Invalid manifest path {}: {err}", cargo_toml.display())
    });
    let args = ExecuteArgs {
        manifest_path,
        verbosity: Verbosity::Default,
        build_mode: BuildMode::Debug,
        features: Features::from(features),
        network: Network::Online,
        build_artifact: BuildArtifacts::CodeOnly,
        unstable_flags: UnstableFlags::default(),
        keep_debug_symbols: false,
        extra_lints: false,
        output_type: OutputType::HumanReadable,
        image: ImageVariant::Default,
        metadata_spec: None,
        target_dir: Some(target_dir),
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
            panic!("contract build for {} failed: {err}", cargo_toml.display())
        }
    }
}
