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
    error::Error,
    io::{
        prelude::*,
        Write,
    },
    iter::Iterator,
    result::Result,
};
use zip::{
    result::ZipError,
    write::FileOptions,
    CompressionMethod,
    ZipWriter,
};

use std::{
    fs::File,
    path::{
        Path,
        PathBuf,
    },
};
use walkdir::WalkDir;

const DEFAULT_UNIX_PERMISSIONS: u32 = 0o755;

fn main() {
    let src_dir = PathBuf::from("./template");
    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR should be set by cargo");
    let dst_file = Path::new(&out_dir).join("template.zip");

    match zip_dir(&src_dir, &dst_file, CompressionMethod::Stored) {
        Ok(_) => {
            println!(
                "done: {} written to {}",
                src_dir.display(),
                dst_file.display()
            )
        }
        Err(e) => eprintln!("Error: {:?}", e),
    };
}

fn zip_dir(
    src_dir: &PathBuf,
    dst_file: &PathBuf,
    method: CompressionMethod,
) -> Result<(), Box<dyn Error>> {
    if !src_dir.is_dir() {
        return Err(ZipError::FileNotFound.into())
    }

    let file = File::create(dst_file)?;

    let walkdir = WalkDir::new(src_dir);
    let it = walkdir.into_iter().filter_map(|e| e.ok());

    let mut zip = ZipWriter::new(file);
    let options = FileOptions::default()
        .compression_method(method)
        .unix_permissions(DEFAULT_UNIX_PERMISSIONS);

    let mut buffer = Vec::new();
    for entry in it {
        let path = entry.path();
        let name = path.strip_prefix(&src_dir)?;

        if path.is_file() {
            zip.start_file_from_path(name, options)?;
            let mut f = File::open(path)?;

            f.read_to_end(&mut buffer)?;
            zip.write_all(&*buffer)?;
            buffer.clear();
        } else if name.as_os_str().len() != 0 {
            zip.add_directory_from_path(name, options)?;
        }
    }
    zip.finish()?;

    Ok(())
}
