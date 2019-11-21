// Copyright 2018-2019 Parity Technologies (UK) Ltd.
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

use std::path::PathBuf;

use cargo_metadata::MetadataCommand;

use crate::cmd::Result;

/// Executes build of the smart-contract which produces a wasm binary that is ready for deploying.
///
/// It does so by invoking build by cargo and then post processing the final binary.
pub(crate) fn execute_generate_metadata(dir: Option<&PathBuf>) -> Result<String> {
    println!("  Generating metadata");

    super::exec_cargo(
        "run",
        &[
            "--package",
            "abi-gen",
            "--release",
            // "--no-default-features", // Breaks builds for MacOS (linker errors), we should investigate this issue asap!
            "--verbose",
        ],
        dir,
    )?;

    let cargo_metadata = MetadataCommand::new().exec()?;
    let mut out_path = cargo_metadata.target_directory;
    out_path.push("metadata.json");

    Ok(format!(
        "Your metadata file is ready.\nYou can find it here:\n{}",
        out_path.display()
    ))
}

#[cfg(test)]
mod tests {
    use crate::{
        cmd::{
            execute_generate_metadata,
            execute_new,
            tests::with_tmp_dir,
        },
        AbstractionLayer,
    };

    #[cfg(feature = "test-ci-only")]
    #[test]
    fn generate_metadata() {
        with_tmp_dir(|path| {
            execute_new(AbstractionLayer::Lang, "new_project", Some(path))
                .expect("new project creation failed");
            let working_dir = path.join("new_project");
            execute_generate_metadata(Some(&working_dir))
                .expect("generate metadata failed");

            let mut abi_file = working_dir;
            abi_file.push("target");
            abi_file.push("metadata.json");
            assert!(abi_file.exists())
        });
    }
}
