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

use std::{
    io::{
        self,
        Write,
    },
    path::PathBuf,
    process::Command,
};

mod metadata;
mod build;
mod deploy;
mod error;
mod new;

pub(crate) use self::{
    metadata::execute_generate_metadata,
    build::execute_build,
    deploy::execute_deploy,
    error::{
        CommandError,
        Result,
    },
    new::execute_new,
};

fn exec_cargo(
    command: &str,
    args: &[&'static str],
    working_dir: Option<&PathBuf>,
) -> Result<()> {
    let mut cmd = Command::new("cargo");
    let mut is_nightly_cmd = Command::new("cargo");
    if let Some(dir) = working_dir {
        cmd.current_dir(dir);
        is_nightly_cmd.current_dir(dir);
    }

    let is_nightly_default = is_nightly_cmd
        .arg("--version")
        .output()
        .map_err(|_| ())
        .and_then(|o| String::from_utf8(o.stdout).map_err(|_| ()))
        .unwrap_or_default()
        .contains("-nightly");

    if !is_nightly_default {
        cmd.arg("+nightly");
    }

    let output = cmd.arg(command).args(args).output()?;

    if !output.status.success() {
        // Dump the output streams produced by cargo into the stdout/stderr.
        io::stdout().write_all(&output.stdout)?;
        io::stderr().write_all(&output.stderr)?;
        return Err(error::CommandError::BuildFailed)
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use tempfile::TempDir;

    pub fn with_tmp_dir<F: FnOnce(&PathBuf)>(f: F) {
        let tmp_dir = TempDir::new().expect("temporary directory creation failed");

        f(&tmp_dir.into_path());
    }
}
