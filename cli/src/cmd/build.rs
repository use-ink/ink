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
    CommandError as Error,
    Result,
};
use cargo_metadata::MetadataCommand;
use parity_wasm::elements::{
    External,
    MemoryType,
    Module,
    Section,
};
use std::{
    io::{
        self,
        Write,
    },
    path::PathBuf,
    process::Command,
};

/// This is the maximum number of pages available for a contract to allocate.
const MAX_MEMORY_PAGES: u32 = 16;

/// Relevant metadata obtained from Cargo.toml.
pub struct CrateMetadata {
    original_wasm: PathBuf,
    dest_wasm: PathBuf,
}

impl CrateMetadata {
    /// Get the path of the wasm destination file
    pub fn dest_wasm(self) -> PathBuf {
        self.dest_wasm
    }
}

/// Parses the contract manifest and returns relevant metadata.
pub fn collect_crate_metadata(working_dir: Option<&PathBuf>) -> Result<CrateMetadata> {
    let mut cmd = MetadataCommand::new();
    if let Some(dir) = working_dir {
        cmd.current_dir(dir);
    }
    let metadata = cmd.exec()?;

    let root_package_id = metadata
        .resolve
        .and_then(|resolve| resolve.root)
        .ok_or_else(|| Error::Other("Cannot infer the root project id".to_string()))?;

    // Find the root package by id in the list of packages. It is logical error if the root
    // package is not found in the list.
    let root_package = metadata
        .packages
        .iter()
        .find(|package| package.id == root_package_id)
        .expect("The package is not found in the `cargo metadata` output");

    // Normalize the package name.
    let package_name = root_package.name.replace("-", "_");

    // {target_dir}/wasm32-unknown-unknown/release/{package_name}.wasm
    let mut original_wasm = metadata.target_directory.clone();
    original_wasm.push("wasm32-unknown-unknown");
    original_wasm.push("release");
    original_wasm.push(package_name.clone());
    original_wasm.set_extension("wasm");

    // {target_dir}/{package_name}.wasm
    let mut dest_wasm = metadata.target_directory.clone();
    dest_wasm.push(package_name);
    dest_wasm.set_extension("wasm");

    Ok(CrateMetadata {
        original_wasm,
        dest_wasm,
    })
}

/// Invokes `cargo build` in the specified directory, defaults to the current directory.
///
/// Currently it assumes that user wants to use `+nightly`.
fn build_cargo_project(working_dir: Option<&PathBuf>) -> Result<()> {
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
        .args(&[
            "build",
            "--no-default-features",
            "--release",
            "--target=wasm32-unknown-unknown",
            "--verbose",
        ])
        .output()?;

    if !output.status.success() {
        // Dump the output streams produced by cargo into the stdout/stderr.
        io::stdout().write_all(&output.stdout)?;
        io::stderr().write_all(&output.stderr)?;
        return Err(Error::BuildFailed)
    }

    Ok(())
}

/// Ensures the wasm memory import of a given module has the maximum number of pages.
///
/// Iterates over the import section, finds the memory import entry if any and adjusts the maximum
/// limit.
fn ensure_maximum_memory_pages(
    module: &mut Module,
    maximum_allowed_pages: u32,
) -> Result<()> {
    let mem_ty = module
        .import_section_mut()
        .and_then(|section| {
            section.entries_mut()
                .iter_mut()
                .find_map(|entry| {
                    match entry.external_mut() {
                        External::Memory(ref mut mem_ty) => Some(mem_ty),
                        _ => None,
                    }
                })
        })
        .ok_or_else(||
            Error::Other(
                "Memory import is not found. Is --import-memory specified in the linker args".to_string()
            )
        )?;

    if let Some(requested_maximum) = mem_ty.limits().maximum() {
        // The module already has maximum, check if it is within the limit bail out.
        if requested_maximum > maximum_allowed_pages {
            return Err(
                Error::Other(
                    format!(
                        "The wasm module requires {} pages. The maximum allowed number of pages is {}",
                        requested_maximum,
                        maximum_allowed_pages,
                    )
                )
            );
        }
    } else {
        let initial = mem_ty.limits().initial();
        *mem_ty = MemoryType::new(initial, Some(MAX_MEMORY_PAGES));
    }

    Ok(())
}

/// Strips all custom sections.
///
/// Presently all custom sections are not required so they can be stripped safely.
fn strip_custom_sections(module: &mut Module) {
    module.sections_mut().retain(|section| {
        match section {
            Section::Custom(_) => false,
            Section::Name(_) => false,
            Section::Reloc(_) => false,
            _ => true,
        }
    });
}

/// Performs required post-processing steps on the wasm artifact.
fn post_process_wasm(crate_metadata: &CrateMetadata) -> Result<()> {
    // Deserialize wasm module from a file.
    let mut module = parity_wasm::deserialize_file(&crate_metadata.original_wasm)?;

    // Perform optimization.
    //
    // In practice only tree-shaking is performed, i.e transitively removing all symbols that are
    // NOT used by the specified entrypoints.
    pwasm_utils::optimize(&mut module, ["call", "deploy"].to_vec())?;
    ensure_maximum_memory_pages(&mut module, MAX_MEMORY_PAGES)?;
    strip_custom_sections(&mut module);

    parity_wasm::serialize_to_file(&crate_metadata.dest_wasm, module)?;
    Ok(())
}

/// Executes build of the smart-contract which produces a wasm binary that is ready for deploying.
///
/// It does so by invoking build by cargo and then post processing the final binary.
pub(crate) fn execute_build(working_dir: Option<&PathBuf>) -> Result<String> {
    println!(" [1/3] Collecting crate metadata");
    let crate_metadata = collect_crate_metadata(working_dir)?;
    println!(" [2/3] Building cargo project");
    build_cargo_project(working_dir)?;
    println!(" [3/3] Post processing wasm file");
    post_process_wasm(&crate_metadata)?;

    Ok(format!(
        "Your contract is ready.\nYou can find it here:\n{}",
        crate_metadata.dest_wasm.display()
    ))
}

#[cfg(test)]
mod tests {
    use crate::{
        cmd::{
            execute_new,
            tests::with_tmp_dir,
        },
        AbstractionLayer,
    };

    #[cfg(feature = "test-ci-only")]
    #[test]
    // FIXME: https://github.com/paritytech/ink/issues/202
    // currently fails on CI because of global RUSTFLAGS overriding required `--import-memory`
    #[ignore]
    fn build_template() {
        with_tmp_dir(|path| {
            execute_new(AbstractionLayer::Lang, "new_project", Some(path))
                .expect("new project creation failed");
            super::execute_build(Some(&path.join("new_project"))).expect("build failed");
        });
    }
}
