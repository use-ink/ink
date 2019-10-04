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

use crate::{
    cmd::{
        CommandError,
        Result,
    },
    AbstractionLayer,
};
use heck::CamelCase as _;
use std::{
    fs,
    io::{
        Cursor,
        Read,
        Seek,
        SeekFrom,
        Write,
    },
    path,
};

/// Initializes a project structure for the `lang` abstraction layer.
fn initialize_for_lang(name: &str) -> Result<String> {
    if name.contains("-") {
        return Err("Contract names cannot contain hyphens".into())
    }

    let out_dir = path::Path::new(name);
    if out_dir.join("Cargo.toml").exists() {
        return Err(format!("A Cargo package already exists in {}", name).into())
    }
    if !out_dir.exists() {
        fs::create_dir(out_dir)?;
    }

    let template = include_bytes!(concat!(env!("OUT_DIR"), "/template.zip"));
    let mut cursor = Cursor::new(Vec::new());
    cursor.write_all(template)?;
    cursor.seek(SeekFrom::Start(0))?;

    let mut archive = zip::ZipArchive::new(cursor)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        // replace template placeholders
        let contents = contents.replace("{{name}}", name);
        let contents = contents.replace("{{camel_name}}", &name.to_camel_case());

        let outpath = out_dir.join(file.sanitized_name());

        if (&*file.name()).ends_with('/') {
            fs::create_dir_all(&outpath)?;
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(&p)?;
                }
            }
            let mut outfile = fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(outpath.clone())
                .map_err(|e| {
                    if e.kind() == std::io::ErrorKind::AlreadyExists {
                        CommandError::from(format!(
                            "New contract file {} already exists",
                            outpath.display()
                        ))
                    } else {
                        CommandError::from(e)
                    }
                })?;

            outfile.write_all(contents.as_bytes())?;
        }

        // Get and set permissions
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            if let Some(mode) = file.unix_mode() {
                fs::set_permissions(&outpath, fs::Permissions::from_mode(mode))?;
            }
        }
    }

    Ok(format!("Created contract {}", name))
}

pub(crate) fn execute_new(layer: AbstractionLayer, name: &str) -> Result<String> {
    match layer {
        AbstractionLayer::Core => Err(CommandError::UnimplementedAbstractionLayer),
        AbstractionLayer::Model => Err(CommandError::UnimplementedAbstractionLayer),
        AbstractionLayer::Lang => initialize_for_lang(name),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_hyphenated_name() {
        let result = super::initialize_for_lang("should-fail");
        assert_eq!(
            format!("{:?}", result),
            r#"Err(Other("Contract names cannot contain hyphens"))"#
        )
    }

    #[test]
    fn contract_cargo_project_already_exists() {
        let name = "test_contract_cargo_project_already_exists";
        let _ = super::initialize_for_lang(name);
        let result = super::initialize_for_lang(name);
        // clean up created files
        std::fs::remove_dir_all(name).unwrap();
        assert_eq!(
            format!("{:?}", result),
            r#"Err(Other("A Cargo package already exists in test_contract_cargo_project_already_exists"))"#
        )
    }

    #[test]
    fn dont_overwrite_existing_files_not_in_cargo_project() {
        let name = "dont_overwrite_existing_files";
        let dir = path::Path::new(name);
        fs::create_dir_all(dir).unwrap();
        fs::File::create(dir.join(".gitignore")).unwrap();
        let result = super::initialize_for_lang(name);
        // clean up created files
        std::fs::remove_dir_all(dir).unwrap();
        assert_eq!(
            format!("{:?}", result),
            r#"Err(Other("New contract file dont_overwrite_existing_files/.gitignore already exists"))"#
        )
    }
}
