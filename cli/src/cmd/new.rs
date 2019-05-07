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

/// Initializes a project structure for the `lang` abstraction layer.
fn initialize_for_lang(name: &str) -> Result<()> {
    use std::{fs, io};
    use std::io::{Cursor, Read, Seek, SeekFrom, Write};
    fs::create_dir(name)?;

    let mut template = include_bytes!(concat!(env!("OUT_DIR"), "/template.zip"));
    let mut cursor = Cursor::new(Vec::new());
    cursor.write_all(template).unwrap();
    cursor.seek(SeekFrom::Start(0)).unwrap();

    let mut archive = zip::ZipArchive::new(cursor).unwrap();

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        let outpath = file.sanitized_name();

        {
            let comment = file.comment();
            if !comment.is_empty() {
                println!("File {} comment: {}", i, comment);
            }
        }

        if (&*file.name()).ends_with('/') {
            println!("File {} extracted to \"{}\"", i, outpath.as_path().display());
            fs::create_dir_all(&outpath).unwrap();
        } else {
            println!("File {} extracted to \"{}\" ({} bytes)", i, outpath.as_path().display(), file.size());
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(&p).unwrap();
                }
            }
            let mut outfile = fs::File::create(&outpath).unwrap();
            io::copy(&mut file, &mut outfile).unwrap();
        }

        // Get and Set permissions
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            if let Some(mode) = file.unix_mode() {
                fs::set_permissions(&outpath, fs::Permissions::from_mode(mode)).unwrap();
            }
        }
    }

    Ok(())
}

pub(crate) fn execute_new(layer: &AbstractionLayer, name: &str) -> Result<()> {
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
