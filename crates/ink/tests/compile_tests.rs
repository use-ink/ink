// Copyright (C) Use Ink (UK) Ltd.
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
    env,
    process::{
        Command,
        ExitStatus,
    },
};

#[test]
fn ui_tests_blake2b_pass() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/blake2b/pass/*.rs");
}

#[test]
fn ui_tests_blake2b_fail() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/blake2b/fail/*.rs");
}

#[test]
fn ui_tests_selector_id_pass() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/selector_id/pass/*.rs");
}

#[test]
fn ui_tests_selector_id_fail() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/selector_id/fail/*.rs");
}

#[test]
fn ui_tests_selector_bytes_pass() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/selector_bytes/pass/*.rs");
}

#[test]
fn ui_tests_selector_bytes_fail() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/selector_bytes/fail/*.rs");
}

#[test]
fn ui_tests_contract_pass() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/contract/pass/*.rs");
}

#[test]
fn ui_tests_contract_constructor_pass() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/contract/pass/constructor/*.rs");
}

#[test]
fn ui_tests_contract_message_pass() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/contract/pass/message/*.rs");
}

#[test]
fn ui_tests_contract_example_pass() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/contract/pass/example/*.rs");
}

#[test]
fn ui_tests_contract_event_pass() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/contract/pass/event/*.rs");
}

#[test]
fn ui_tests_contract_fail() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/contract/fail/*.rs");
}

#[test]
fn ui_tests_contract_cfg_fail() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/contract/fail/cfg/*.rs");
}

#[test]
fn ui_tests_contract_constructor_fail() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/contract/fail/constructor/*.rs");
}

#[test]
fn ui_tests_contract_event_fail() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/contract/fail/event/*.rs");
}

#[test]
fn ui_tests_contract_impl_fail() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/contract/fail/impl/*.rs");
}

#[test]
fn ui_tests_contract_message_fail() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/contract/fail/message/*.rs");
}

#[test]
fn ui_tests_contract_trait_fail() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/contract/fail/trait/*.rs");
}

#[test]
fn ui_tests_event_pass() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/event/pass/*.rs");
}

#[test]
fn ui_tests_event_fail() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/event/fail/*.rs");
}

#[test]
fn ui_tests_storage_item_pass() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/storage_item/pass/*.rs");
}

#[test]
fn ui_tests_storage_item_fail() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/storage_item/fail/*.rs");
}

#[test]
fn ui_tests_trait_def_pass() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/trait_def/pass/*.rs");
}

#[test]
fn ui_tests_trait_def_fail() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/trait_def/fail/*.rs");
}

#[test]
fn ui_tests_pay_with_call_pass() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/pay_with_call/pass/multiple_args.rs");
}

#[test]
fn ui_tests_scale_derive_pass() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/scale_derive/pass/*.rs");
}

#[test]
fn ui_tests_scale_derive_fail() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/scale_derive/fail/*.rs");
}

#[test]
// Builds `trybuild_wrapper` and runs all tests that require ABI `cfg` flags with it.
// See `tests/trybuild_wrapper` crate docs for motivation.
fn ui_tests_abi() {
    let tests = [
        // "sol" ABI.
        ("sol", "tests/ui/abi/sol/pass/*.rs", "pass"),
        ("sol", "tests/ui/abi/sol/fail/*.rs", "fail"),
        // "all" ABI.
        ("all", "tests/ui/abi/all/pass/*.rs", "pass"),
        ("all", "tests/ui/abi/all/fail/*.rs", "fail"),
    ];

    // Retrieves or generates path to `trybuild` wrapper.
    let wrapper = env::var("TRYBUILD_WRAPPER").unwrap_or_else(|_| {
        generate_trybuild_wrapper().unwrap_or_else(|err| panic!("Error: {err}"))
    });

    // Runs tests and tracks failures.
    let mut failures = Vec::new();
    for (abi, path, expected) in tests {
        let res = trybuild_wrapper_test(&wrapper, abi, path, expected);
        if res.is_err() {
            failures.push(format!("    {path}"));
        }
    }

    // Only pass if there are no failures.
    assert!(failures.is_empty(), "failures:\n{}", failures.join("\n"));
}

/// Generates a `trybuild` wrapper executable and returns its path.
fn generate_trybuild_wrapper() -> Result<String, String> {
    // Composes `cargo build` command.
    let mut cmd = Command::new("cargo");
    // Removes `CARGO_*` env vars (except `CARGO_TARGET_DIR`).
    cmd.env_clear();
    cmd.envs(
        env::vars()
            .filter(|(key, _)| !key.starts_with("CARGO_") || key == "CARGO_TARGET_DIR"),
    );
    cmd.current_dir("tests/trybuild_wrapper").args([
        "build",
        "--release",
        // JSON output is easier to parse.
        "--message-format=json",
    ]);

    // Compiles `trybuild` wrapper.
    let error_msg = "Failed to generate `trybuild` wrapper";
    let output = cmd.output().map_err(|err| format!("{error_msg}: {err}"))?;
    if !output.status.success() {
        return Err(if output.stderr.is_empty() {
            error_msg.to_string()
        } else {
            format!("{error_msg}: {}", String::from_utf8_lossy(&output.stderr))
        })
    }

    // Parses JSON output for path to executable.
    // Ref: <https://doc.rust-lang.org/cargo/reference/external-tools.html#artifact-messages>
    // Ref: <https://doc.rust-lang.org/rustc/json.html>
    let stdout = String::from_utf8_lossy(&output.stdout);
    let exec_path = stdout.lines().find_map(|line| {
        if !line.contains("\"compiler-artifact\"") {
            return None;
        }
        let pat = "\"executable\":\"";
        let pat_start = line.find(pat)?;
        let path_start = pat_start + pat.len();
        let target_substr = &line[path_start..];
        let end_pos = target_substr.find("\"")?;
        Some(&target_substr[..end_pos])
    });
    exec_path
        .map(ToString::to_string)
        .ok_or_else(|| error_msg.to_string())
}

// Runs ABI ui tests with specified `trybuild_wrapper`.
// See `tests/trybuild_wrapper` crate docs for motivation.
fn trybuild_wrapper_test(
    wrapper: &str,
    abi: &str,
    path: &str,
    expected: &str,
) -> Result<(), ExitStatus> {
    let mut cmd = Command::new(wrapper);
    cmd.args(["trybuild", "--path", path, "--expected", expected]);

    // Override `cargo` path while preserving initial path used by the test command.
    if let Ok(cargo) = env::var("CARGO") {
        cmd.env("TRYBUILD_WRAPPER_CARGO", cargo);
    }
    cmd.env("CARGO", wrapper);

    // Set ABI `cfg` flags that will be passed to `cargo` by the `trybuild` wrapper.
    let abi_cfg = format!(
        "--cfg\x1fink_abi=\"{abi}\"\x1f--check-cfg\x1fcfg(ink_abi,values(\"ink\",\"sol\",\"all\"))"
    );
    cmd.env("TRYBUILD_WRAPPER_ENCODED_FLAGS", abi_cfg);

    // Enable `std` and `unstable-hostfn` features (needed by events and metadata tests).
    cmd.env("TRYBUILD_WRAPPER_CARGO_ARGS", "--features std");

    let exit_status = cmd.status().unwrap();
    if !exit_status.success() {
        return Err(exit_status)
    }

    Ok(())
}
