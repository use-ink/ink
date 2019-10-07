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

mod build;
mod deploy;
mod error;
mod new;

pub(crate) use self::{
    build::execute_build,
    deploy::execute_deploy,
    error::{
        CommandError,
        Result,
    },
    new::execute_new,
};

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use tempfile::TempDir;

    pub fn with_tmp_dir<F: FnOnce(&PathBuf)>(f: F) {
        let tmp_dir = TempDir::new().expect("temporary directory creation failed");

        f(&tmp_dir.into_path());
    }
}
