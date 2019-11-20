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

use crate::{
    cmd::{
        CommandError,
        Result,
    },
    AbstractionLayer,
};
use heck::CamelCase as _;
use std::{
    env,
    fs,
    io::{
        Cursor,
        Read,
        Seek,
        SeekFrom,
        Write,
    },
    path::PathBuf,
};

/// Initializes a project structure for the `lang` abstraction layer.
fn initialize_for_lang(name: &str, target_dir: Option<&PathBuf>) -> Result<String> {
    if name.contains('-') {
        return Err("Contract names cannot contain hyphens".into())
    }

    let out_dir = target_dir.unwrap_or(&env::current_dir()?).join(name);
    if out_dir.join("Cargo.toml").exists() {
        return Err(format!("A Cargo package already exists in {}", name).into())
    }
    if !out_dir.exists() {
        fs::create_dir(&out_dir)?;
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
                            file.sanitized_name().display()
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

pub(crate) fn execute_new(
    layer: AbstractionLayer,
    name: &str,
    dir: Option<&PathBuf>,
) -> Result<String> {
    match layer {
        AbstractionLayer::Core => Err(CommandError::UnimplementedAbstractionLayer),
        AbstractionLayer::Model => Err(CommandError::UnimplementedAbstractionLayer),
        AbstractionLayer::Lang => initialize_for_lang(name, dir),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        cmd::{
            execute_new,
            tests::with_tmp_dir,
        },
        AbstractionLayer,
    };

    #[test]
    fn rejects_hyphenated_name() {
        with_tmp_dir(|path| {
            let result = execute_new(
                AbstractionLayer::Lang,
                "rejects-hyphenated-name",
                Some(path),
            );
            assert_eq!(
                format!("{:?}", result),
                r#"Err(Other("Contract names cannot contain hyphens"))"#
            )
        });
    }

    #[test]
    fn contract_cargo_project_already_exists() {
        with_tmp_dir(|path| {
            let name = "test_contract_cargo_project_already_exists";
            let _ = execute_new(AbstractionLayer::Lang, name, Some(path));
            let result = execute_new(AbstractionLayer::Lang, name, Some(path));
            assert_eq!(
                format!("{:?}", result),
                r#"Err(Other("A Cargo package already exists in test_contract_cargo_project_already_exists"))"#
            )
        });
    }

    #[test]
    fn dont_overwrite_existing_files_not_in_cargo_project() {
        with_tmp_dir(|path| {
            let name = "dont_overwrite_existing_files";
            let dir = path.join(name);
            fs::create_dir_all(&dir).unwrap();
            fs::File::create(dir.join(".gitignore")).unwrap();
            let result = execute_new(AbstractionLayer::Lang, name, Some(path));
            assert_eq!(
                format!("{:?}", result),
                r#"Err(Other("New contract file .gitignore already exists"))"#
            )
        });
    }
}
