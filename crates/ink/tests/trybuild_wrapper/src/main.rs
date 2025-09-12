// Copyright (C) ink! contributors.
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

//! A `trybuild` wrapper that allows passing extra `rustc` flags to ui tests.
//!
//! # Motivation
//!
//! `trybuild` sets compiler flags using CLI config overrides (i.e.
//! `--config=build.rustflags=[...]` and similar) which take precedence over `RUSTFLAGS`
//! (and similar) env vars.
//!
//! `trybuild` also doesn't provide an interface for passing extra `rustc` flags to ui
//! tests, and there's some reason to believe that its maintainer considers such an
//! interface to be out of scope (see https://github.com/dtolnay/trybuild/issues/183 and
//! https://github.com/dtolnay/trybuild/issues/108).
//!
//! # Usage
//!
//! - Extra `rustc` flags are specified via the `TRYBUILD_WRAPPER_ENCODED_FLAGS` env var.
//! - `CARGO` env var should be set to this binary (allows us to pass extra `rustc` flags
//!   to cargo).
//! - Original `CARGO` env var (if any), should be specified via the
//!   `TRYBUILD_WRAPPER_CARGO`
//! - Extra `cargo` args are specified via the `TRYBUILD_WRAPPER_CARGO_ARGS` env var.
//!
//! ```shell
//! $ TRYBUILD_WRAPPER_ENCODED_FLAGS="..." TRYBUILD_WRAPPER_CARGO="..." CARGO="trybuild_wrapper" \
//!   trybuild_wrapper --path <path/to/ui/tests> --expected <pass|fail>
//! ```
//!
//! # Note
//!
//! Preferably, the env vars should be defined inline because `std::env::set_var` is not
//! thread safe.
//!
//! # References
//!
//! - https://github.com/dtolnay/trybuild/blob/e3d8dab0bb4002d077c35edda73c61cb1c0c8703/src/cargo.rs#L47-L48
//! - https://github.com/dtolnay/trybuild/blob/e3d8dab0bb4002d077c35edda73c61cb1c0c8703/src/cargo.rs#L42
//! - https://doc.rust-lang.org/cargo/reference/config.html#command-line-overrides
//! - https://doc.rust-lang.org/std/env/fn.set_var.html

use std::{
    env,
    process::{
        Command,
        exit,
    },
};

use clap::Parser;
use trybuild::TestCases;

fn main() {
    // First arg is this executable so we skip it.
    let mut args = env::args().skip(1);

    // Next arg should either be `trybuild` or a `cargo` subcommand.
    let subcommand = args.next().unwrap();
    if subcommand == "trybuild" {
        // Calls `trybuild`.
        trybuild(env::args().skip(1));
    } else {
        // Calls `cargo` subcommand and passes extra `rustc` flags (if specified).
        cargo(env::args().skip(1));
    }
}

/// Calls `trybuild` with passed args.
fn trybuild<T>(args: T)
where
    T: Iterator<Item = String>,
{
    let Args { path, expected } = Args::parse_from(args);
    let t = TestCases::new();
    match expected {
        Expected::Pass => t.pass(path),
        Expected::Fail => t.compile_fail(path),
    }
}

/// `trybuild` wrapper args.
#[derive(Parser, Debug)]
#[command(about, long_about = None)]
struct Args {
    /// Path to ui tests.
    #[arg(long)]
    path: String,
    /// Expected test result.
    #[arg(long)]
    expected: Expected,
}

/// Expected test result.
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
enum Expected {
    Pass,
    Fail,
}

/// Calls `cargo` with extra flags `rustc` flags
/// specified in `TRYBUILD_WRAPPER_ENCODED_FLAGS` env var (if any)
/// and extra args specified via `TRYBUILD_WRAPPER_CARGO_ARGS` (if any).
fn cargo<T>(args: T)
where
    T: Iterator<Item = String>,
{
    // Composes `cargo` command.
    let mut cmd = Command::new(
        env::var("TRYBUILD_WRAPPER_CARGO")
            .as_deref()
            .unwrap_or("cargo"),
    );

    // Passes extra `rustc` flags to `cargo` (if specified).
    // See `with_abi_flags` doc for details.
    with_abi_flags(&mut cmd, args);

    // Passes extra cargo args if specified via `TRYBUILD_WRAPPER_CARGO_ARGS`.
    if let Ok(extra_args) = env::var("TRYBUILD_WRAPPER_CARGO_ARGS") {
        let extra_args: Vec<_> = extra_args.split(' ').collect();
        if !extra_args.is_empty() {
            cmd.args(extra_args);
        }
    }

    // Runs `cargo`.
    let exit_status = cmd.status().expect("Failed to spawn `cargo` process");
    if !exit_status.success() {
        // Exits with appropriate exit code on failure.
        exit(exit_status.code().unwrap_or(-1));
    }
}

/// Passes extra `rustc` flags to `cargo` (if specified).
///
/// # Note
///
/// `trybuild` sets compiler flags using CLI overrides as they take precedence over env
/// vars, So we parse the provided `--config=build.rustflags` arg, and add the extra
/// `rustc` flags specified by the `TRYBUILD_WRAPPER_ENCODED_FLAGS` env var (if any).
///
/// Ref: <https://doc.rust-lang.org/cargo/reference/config.html#command-line-overrides>
///
/// Ref: <https://github.com/dtolnay/trybuild/blob/e3d8dab0bb4002d077c35edda73c61cb1c0c8703/src/cargo.rs#L47-L48>
fn with_abi_flags<T>(cmd: &mut Command, args: T)
where
    T: Iterator<Item = String>,
{
    // Leaves args unchanged if extra flags are specified via
    // `TRYBUILD_WRAPPER_ENCODED_FLAGS`.
    let Ok(extra_rustflags) = env::var("TRYBUILD_WRAPPER_ENCODED_FLAGS") else {
        cmd.args(args);
        return;
    };
    let rustflags: Vec<_> = extra_rustflags.split('\x1f').collect();
    if rustflags.is_empty() {
        cmd.args(args);
        return;
    }

    // Adds extra flags to `build.rustflags` config set by `trybuild`.
    let rustflags_toml = toml::Value::try_from(rustflags)
        .expect("Failed to parse flags into TOML syntax")
        .to_string();
    let rustflags_extra_items = rustflags_toml
        .strip_prefix('[')
        .expect("Expected TOML array")
        .strip_suffix(']')
        .expect("Expected TOML array");
    let mut new_args = Vec::new();
    let mut ignore_next = false;
    for arg in args {
        // Skips ignored arg values (e.g. the `--target` value).
        if ignore_next {
            ignore_next = false;
            continue;
        }

        // `trybuild` uses `--config=build.rustflags=[..]` syntax when overriding `cargo`
        // config. NOTE: Below also updates the
        // `--config=build.<target_triple>.rustflags=[..]` arg (if specified).
        if arg.starts_with("--config=")
            && arg.contains(".rustflags=")
            && arg.ends_with("]")
        {
            let mut new_arg = arg
                .strip_suffix(']')
                .expect("Expected TOML array")
                .to_string();
            new_arg.push_str(", ");
            new_arg.push_str(rustflags_extra_items);
            new_arg.push(']');
            new_args.push(new_arg);
        } else if arg == "--target" {
            // Removes the `--target` arg if included.
            // Ideally, `trybuild` should be compiled with `--cfg trybuild_no_target`,
            // so that it doesn't include the `--target` arg, but it doesn't hurt to be
            // extra safe. Ref: <https://github.com/dtolnay/trybuild/blob/a2eb852409a69841ccca1acbf61813e8e8abb792/src/cargo.rs#L191-L204>
            ignore_next = true;
        } else {
            new_args.push(arg);
        }
    }
    cmd.args(new_args);
}
