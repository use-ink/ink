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
        CommandErrorKind,
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
fn initialize_for_lang(name: &str) -> Result<()> {
    if name.contains("-") {
        return Err("Contract names cannot contain hyphens".into())
    }
    fs::create_dir(name)?;
    let out_dir = path::Path::new(name);

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
            let mut outfile = fs::File::create(&outpath)?;
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

    Ok(())
}

pub(crate) fn execute_new(layer: AbstractionLayer, name: &str) -> Result<()> {
    match layer {
        AbstractionLayer::Core => {
            Err(CommandError::new(
                CommandErrorKind::UnimplementedAbstractionLayer,
            ))
        }
        AbstractionLayer::Model => {
            Err(CommandError::new(
                CommandErrorKind::UnimplementedAbstractionLayer,
            ))
        }
        AbstractionLayer::Lang => initialize_for_lang(name),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn rejects_hyphenated_name() {
        let result = super::initialize_for_lang("should-fail");
        assert_eq!(
            format!("{:?}", result),
            r#"Err(CommandError { kind: Other("Contract names cannot contain hyphens") })"#
        )
    }
}
