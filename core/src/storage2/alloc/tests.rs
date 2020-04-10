// Copyright 2019-2020 Parity Technologies (UK) Ltd.
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

use super::{
    alloc,
    free,
    DynamicAllocation,
    set_contract_phase,
    ContractPhase,
};
use crate::env::{
    test,
    DefaultEnvTypes,
};

fn run_default_test<F>(f: F)
where
    F: FnOnce(),
{
    set_contract_phase(ContractPhase::Deploy);
    test::run_test::<DefaultEnvTypes, _>(|_| {
        f();
        Ok(())
    }).unwrap();
}

#[test]
fn alloc_works() {
    run_default_test(|| {
        assert_eq!(alloc(), DynamicAllocation(0));
    })
}

#[test]
fn free_works() {
    run_default_test(|| {
        // Check that this pattern does not panic.
        free(alloc());
    })
}
