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

//! The minimal test framework for the ink! core libraries.

/// The set-up procedure of the entire crate under test.
fn setup() {}

/// The tear-down procedure of the entire crate under test.
fn teardown() {}

/// Runs the given test.
///
/// This executes general setup routines before executing
/// the test and general tear-down procedures after executing.
pub(crate) fn run_test<F>(test: F) -> ()
where
    F: FnOnce() -> () + std::panic::UnwindSafe,
{
    setup();
    let result = std::panic::catch_unwind(|| test());
    teardown();
    assert!(result.is_ok())
}
