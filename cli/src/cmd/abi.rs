// Copyright 2018-2019 Parity Technologies (UK) Ltd.
// This file is part of ink!.
//
// ink! is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// ink! is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with ink!.  If not, see <http://www.gnu.org/licenses/>.

use crate::cmd::{
    Result,
};
use cargo_metadata::MetadataCommand;
use std::path::PathBuf;

/// Executes build of the smart-contract which produces a wasm binary that is ready for deploying.
///
/// It does so by invoking build by cargo and then post processing the final binary.
pub(crate) fn execute_generate_abi(dir: Option<&PathBuf>) -> Result<String> {
    println!(" Generating abi");

    super::exec_cargo("run", &[
        "--package",
        "abi-gen",
        "--release",
        "--no-default-features",
        "--verbose",
    ], dir)?;

    let metadata = MetadataCommand::new().exec()?;
    let mut abi_path = metadata.target_directory.clone();
    abi_path.push("abi.json");

    Ok(format!(
        "Your abi file is ready.\nYou can find it here:\n{}",
        abi_path.display()
    ))
}

#[cfg(test)]
mod tests {
    use crate::{
        cmd::{
            execute_new,
            execute_generate_abi,
            tests::with_tmp_dir,
        },
        AbstractionLayer,
    };

    #[cfg(feature = "test-ci-only")]
    #[test]
    fn generate_abi() {
        with_tmp_dir(|path| {
            execute_new(AbstractionLayer::Lang, "new_project", Some(path))
                .expect("new project creation failed");
            let working_dir = path.join("new_project");
            super::execute_generate_abi(Some(&working_dir)).expect("generate abi failed");

            let mut abi_file = working_dir.clone();
            abi_file.push("target");
            abi_file.push("abi.json");
            assert!(abi_file.exists())
        });
    }
}
