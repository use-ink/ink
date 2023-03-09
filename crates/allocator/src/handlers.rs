// Copyright 2018-2022 Parity Technologies (UK) Ltd.
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

#[alloc_error_handler]
fn oom(_: core::alloc::Layout) -> ! {
    #[cfg(target_arch = "wasm32")]
    core::arch::wasm32::unreachable();

    // For any other `no_std` architecture we just call an imaginary
    // `abort` function. No other architecture is supported. We
    // just have this to check if compilation does not break
    // for other true nostd targets.
    #[cfg(not(target_arch = "wasm32"))]
    unsafe {
        extern "C" {
            fn abort() -> !;
        }
        abort();
    }
}
