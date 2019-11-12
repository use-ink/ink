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

use std::{
    io::{
        self,
        Write,
    },
    path::PathBuf,
    process::Command,
};

mod abi;
mod build;
mod deploy;
mod error;
mod new;

pub(crate) use self::{
    abi::execute_generate_abi,
    build::execute_build,
    deploy::execute_deploy,
    error::{
        CommandError,
        Result,
    },
    new::execute_new,
};

fn exec_cargo(command: &str, args: &[&'static str], working_dir: Option<&PathBuf>) -> Result<()> {
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

    let output = cmd
        .arg(command)
        .args(args)
        .output()?;

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
