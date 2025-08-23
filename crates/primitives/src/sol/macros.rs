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

/// Calls a macro up to 12 times with an increasing number of identifiers for each call.
///
/// # Note
///
/// The callee is a typical macro that implements a trait for tuples.
///
/// We follow the Rust standard library's convention of implementing traits for tuples up
/// to twelve items long.
///
/// Ref: <https://doc.rust-lang.org/std/primitive.tuple.html#trait-implementations>
macro_rules! impl_all_tuples {
    // Call direct to omit unit `()`.
    (@nonempty $macro: path) => {
        $macro!(T1);
        $macro!(T1, T2);
        $macro!(T1,T2,T3);
        $macro!(T1,T2,T3,T4);
        $macro!(T1,T2,T3,T4,T5);
        $macro!(T1,T2,T3,T4,T5,T6);
        $macro!(T1,T2,T3,T4,T5,T6,T7);
        $macro!(T1,T2,T3,T4,T5,T6,T7,T8);
        $macro!(T1,T2,T3,T4,T5,T6,T7,T8,T9);
        $macro!(T1,T2,T3,T4,T5,T6,T7,T8,T9,T10);
        $macro!(T1,T2,T3,T4,T5,T6,T7,T8,T9,T10,T11);
        $macro!(T1,T2,T3,T4,T5,T6,T7,T8,T9,T10,T11,T12);
    };
    // Default, include `()`.
    ($macro: path) => {
        $macro!();
        impl_all_tuples!(@nonempty $macro);
    };
}
