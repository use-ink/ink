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
    error::Error,
    fs::File,
    io::{
        prelude::*,
        Write,
    },
    iter::Iterator,
    path::{
        Path,
        PathBuf,
    },
    result::Result,
};

use walkdir::WalkDir;
use zip::{
    result::ZipError,
    write::FileOptions,
    CompressionMethod,
    ZipWriter,
};

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
