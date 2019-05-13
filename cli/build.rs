use std::io::prelude::*;
use std::io::Write;
use std::iter::Iterator;
use std::error::Error;
use std::result::Result;
use zip::write::FileOptions;
use zip::result::ZipError;

use walkdir::WalkDir;
use std::path::Path;
use std::fs::File;

const DEFAULT_UNIX_PERMISSIONS: u32 = 0o755;

fn main() {
    let src_dir = "./template";
    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR should be set by cargo");
    let dst_file = format!("{}/template.zip", out_dir); // todo: [AJ] proper path concat

    match zip_dir(src_dir, &dst_file, zip::CompressionMethod::Stored) {
        Ok(_) => println!("done: {} written to {}", src_dir, dst_file),
        Err(e) => eprintln!("Error: {:?}", e),
    };
}

fn zip_dir(src_dir: &str, dst_file: &str, method: zip::CompressionMethod) -> Result<(), Box<Error>> {
    if !Path::new(src_dir).is_dir() {
        return Err(ZipError::FileNotFound.into());
    }

    let path = Path::new(dst_file);
    let file = File::create(&path)?;

    let walkdir = WalkDir::new(src_dir.to_string());
    let it = walkdir
        .into_iter()
        .filter_map(|e| e.ok());

    let mut zip = zip::ZipWriter::new(file);
    let options = FileOptions::default()
        .compression_method(method)
        .unix_permissions(DEFAULT_UNIX_PERMISSIONS);

    let mut buffer = Vec::new();
    for entry in it {
        let path = entry.path();
        let name = path.strip_prefix(Path::new(src_dir))?;

        if path.is_file() {
            println!("adding file {:?} as {:?} ...", path, name);
            zip.start_file_from_path(name, options)?;
            let mut f = File::open(path)?;

            f.read_to_end(&mut buffer)?;
            zip.write_all(&*buffer)?;
            buffer.clear();
        } else if name.as_os_str().len() != 0 {
            println!("adding dir {:?} as {:?} ...", path, name);
            zip.add_directory_from_path(name, options)?;
        }
    }
    zip.finish()?;

    Ok(())
}
