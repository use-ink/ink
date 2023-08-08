use const_gen::*;
use std::{
    env,
    fs,
    path::Path,
};

fn main() {
    // Default size of the buffer: 16 kB.
    let mut size: usize = 1 << 14;
    // if environmental variable is present we update the size.
    if let Ok(size_str) = std::env::var("STATIC_BUFFER_SIZE") {
        if let Ok(new_size) = size_str.parse::<usize>() {
            size = new_size;
        }
    }
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("const_gen.rs");
    // create a constant with the specified value.
    let const_decl = const_declaration!(STATIC_BUFFER_SIZE = size);
    // Appends it to a file with constants.
    fs::write(dest_path, const_decl).unwrap();
}
