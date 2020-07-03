// Copyright 2018-2020 Parity Technologies (UK) Ltd.
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

mod contract;
mod cross_calling;
mod env;
mod storage;
mod traits;

/// Module privately re-exporting all code generators through this namespace.
mod codegen {
    pub use super::{
        cross_calling::CrossCallingConflictCfg,
        contract::Contract,
        env::Env,
        storage::Storage,
    };
}

use self::traits::{
    GenerateCode,
    GenerateCodeUsing,
};

use proc_macro2::TokenStream as TokenStream2;

/// Generates the entire code for the given ink! contract.
pub fn generate_code(contract: &ir::Contract) -> TokenStream2 {
    codegen::Contract::from(contract).generate_code()
}
