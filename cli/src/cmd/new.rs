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

/// Returns a file path from the given segments.
fn filepath_from_segs<I, S>(structure: I) -> String
where
    I: IntoIterator<Item = S>,
    S: std::fmt::Display,
{
    itertools::join(structure, "/")
}

/// Returns the contents of the `Cargo.toml` file for the given smart contract name.
fn cargo_toml_contents(name: &str) -> String {
    format!(
        r##"[package]
name = "{}"
version = "0.1.0"
authors = ["[your_name] <[your_email]>"]
edition = "2018"

[dependencies]
pdsl_core = {{ git = "https://github.com/paritytech/ink", package = "pdsl_core" }}
pdsl_model = {{ git = "https://github.com/paritytech/ink", package = "pdsl_model" }}
pdsl_lang = {{ git = "https://github.com/paritytech/ink", package = "pdsl_lang" }}
parity-codec = {{ version = "3.3", default-features = false, features = ["derive"] }}

[lib]
name = "{}"
crate-type = ["cdylib"]

[features]
default = []
test-env = [
    "pdsl_core/test-env",
    "pdsl_model/test-env",
    "pdsl_lang/test-env",
]
generate-api-description = [
    "pdsl_lang/generate-api-description"
]

[profile.release]
panic = "abort"
lto = true
opt-level = "z""##,
        name, name
    )
}

/// Returns the contents of a generic `.gitignore` file.
fn gitignore_contents() -> String {
    r##"# Ignore build artifacts from the local tests sub-crate.
/target/

# Ignore backup files creates by cargo fmt.
**/*.rs.bk

# Remove Cargo.lock when creating an executable, leave it for libraries
# More information here http://doc.crates.io/guide.html#cargotoml-vs-cargolock
Cargo.lock"##
        .to_owned()
}

/// Returns the contents of the specific `.cargo/config` file.
fn cargo_config_contents() -> String {
    r##"[target.wasm32-unknown-unknown]
rustflags = [
	"-C", "overflow-checks=on",
	"-C", "link-args=-z stack-size=65536 --import-memory"
]"##
    .to_owned()
}

/// Returns the contents of the dummy smart contract.
fn lib_rs_contents(name: &str) -> String {
    use heck::CamelCase as _;
    let camel_name = name.to_camel_case();
    format!(
        r##"#![no_std]

use pdsl_core::storage;
use pdsl_lang::contract;

contract! {{
    /// This simple dummy contract has a `bool` value that can
    /// alter between `true` and `false` using the `flip` message.
    /// Users can retrieve its current state using the `get` message.
    struct {} {{
        /// The current state of our flag.
        value: storage::Value<bool>,
    }}

    impl Deploy for {} {{
        /// Initializes our state to `false` upon deploying our smart contract.
        fn deploy(&mut self) {{
            self.value.set(false)
        }}
    }}

    impl {} {{
        /// Flips the current state of our smart contract.
        pub(external) fn flip(&mut self) {{
            *self.value = !*self.value;
        }}

        /// Returns the current state.
        pub(external) fn get(&self) -> bool {{
            *self.value
        }}
    }}
}}

#[cfg(test)]
mod tests {{
    use super::Flipper;

    #[test]
    fn it_works() {{
        let mut flipper = Flipper::deploy_mock();
        assert_eq!(flipper.get(), true);
        incrementer.flip();
        assert_eq!(flipper.get(), false);
    }}
}}
"##,
        camel_name, camel_name, camel_name,
    )
}

/// Returns the contents of the `build.sh` file.
///
/// # Note
///
/// The `build.sh` file is only a temporary solution until we
/// support the same functionality within `pdsl_cli`.
fn build_sh_contents(name: &str) -> String {
    format!(r##"#!/bin/bash

PROJNAME={}

CARGO_INCREMENTAL=0 &&
cargo build --release --features generate-api-description --target=wasm32-unknown-unknown --verbose &&
wasm2wat -o target/$PROJNAME.wat target/wasm32-unknown-unknown/release/$PROJNAME.wasm &&
cat target/$PROJNAME.wat | sed "s/(import \"env\" \"memory\" (memory (;0;) 2))/(import \"env\" \"memory\" (memory (;0;) 2 16))/" > target/$PROJNAME-fixed.wat &&
wat2wasm -o target/$PROJNAME.wasm target/$PROJNAME-fixed.wat &&
wasm-opt -Oz target/$PROJNAME.wasm -o target/$PROJNAME-opt.wasm"##,
        name
    )
}

fn rust_toolchain_contents() -> String {
    r##"nightly-2019-04-20"##.to_owned()
}

/// Initializes a project structure for the `lang` abstraction layer.
fn initialize_for_lang(name: &str) -> Result<()> {
    use std::fs;
    fs::create_dir(name)?;
    fs::create_dir(filepath_from_segs(&[name, ".cargo"]))?;
    fs::create_dir(filepath_from_segs(&[name, "src"]))?;
    fs::write(
        filepath_from_segs(&[name, ".cargo", "config"]),
        cargo_config_contents(),
    )?;
    fs::write(
        filepath_from_segs(&[name, "Cargo.toml"]),
        cargo_toml_contents(name),
    )?;
    fs::write(
        filepath_from_segs(&[name, ".gitignore"]),
        gitignore_contents(),
    )?;
    fs::write(
        filepath_from_segs(&[name, "src", "lib.rs"]),
        lib_rs_contents(name),
    )?;
    fs::write(
        filepath_from_segs(&[name, "build.sh"]),
        build_sh_contents(name),
    )?;
    fs::write(
        filepath_from_segs(&[name, "rust-toolchain"]),
        rust_toolchain_contents(),
    )?;
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
