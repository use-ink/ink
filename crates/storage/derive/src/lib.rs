// Copyright 2018-2021 Parity Technologies (UK) Ltd.
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

extern crate proc_macro;

mod packed_layout;
mod spread_layout;
mod storage_layout;

#[cfg(test)]
mod tests;

use self::{
    packed_layout::packed_layout_derive,
    spread_layout::spread_layout_derive,
    storage_layout::storage_layout_derive,
};
synstructure::decl_derive!([SpreadLayout] => spread_layout_derive);
synstructure::decl_derive!([PackedLayout] => packed_layout_derive);
synstructure::decl_derive!([StorageLayout] => storage_layout_derive);
