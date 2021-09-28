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

#[test]
fn contract() {
    let t = trybuild::TestCases::new();

    // t.pass("tests/ui/contract/pass/no-implicit-prelude.rs");
    t.pass("tests/ui/contract/pass/minimal-contract.rs");
    t.pass("tests/ui/contract/pass/constructor-many-inputs.rs");
    t.pass("tests/ui/contract/pass/constructor-selector.rs");
    t.pass("tests/ui/contract/pass/message-many-inputs.rs");
    t.pass("tests/ui/contract/pass/message-many-outputs.rs");
    t.pass("tests/ui/contract/pass/message-payable.rs");
    t.pass("tests/ui/contract/pass/message-selector.rs");
    t.pass("tests/ui/contract/pass/storage-single-field.rs");
    t.pass("tests/ui/contract/pass/storage-many-fields.rs");
    t.pass("tests/ui/contract/pass/storage-packed-fields.rs");
    t.pass("tests/ui/contract/pass/storage-with-derives.rs");
    t.pass("tests/ui/contract/pass/event-single-definition.rs");
    t.pass("tests/ui/contract/pass/event-many-definitions.rs");
    t.pass("tests/ui/contract/pass/event-topics.rs");
    t.pass("tests/ui/contract/pass/event-anonymous.rs");
    t.pass("tests/ui/contract/pass/event-config-more-topics.rs");
    t.pass("tests/ui/contract/pass/impl-alias-storage.rs");
    t.pass("tests/ui/contract/pass/impl-with-property.rs");
    t.pass("tests/ui/contract/pass/config-compile-as-dependency-true.rs");
    t.pass("tests/ui/contract/pass/config-compile-as-dependency-false.rs");
    t.pass("tests/ui/contract/pass/config-dynamic-storage-allocator-true.rs");
    t.pass("tests/ui/contract/pass/config-dynamic-storage-allocator-false.rs");
    t.pass("tests/ui/contract/pass/config-custom-env.rs");
    t.pass("tests/ui/contract/pass/env-access.rs");
    t.pass("tests/ui/contract/pass/module-non-ink-items.rs");
    t.pass("tests/ui/contract/pass/module-env-types.rs");
    t.pass("tests/ui/contract/pass/example-flipper-works.rs");
    t.pass("tests/ui/contract/pass/example-incrementer-works.rs");
    t.pass("tests/ui/contract/pass/example-trait-flipper-works.rs");
    t.pass("tests/ui/contract/pass/example-trait-incrementer-works.rs");
    t.pass("tests/ui/contract/pass/example-erc20-works.rs");
    t.pass("tests/ui/contract/pass/example-erc721-works.rs");

    t.compile_fail("tests/ui/contract/fail/C-00-constructor-self-ref.rs");
    t.compile_fail("tests/ui/contract/fail/C-01-constructor-self-mut.rs");
    t.compile_fail("tests/ui/contract/fail/C-02-constructor-self-val.rs");
    t.compile_fail("tests/ui/contract/fail/C-03-constructor-missing-return.rs");
    t.compile_fail("tests/ui/contract/fail/C-04-missing-constructor.rs");
    t.compile_fail("tests/ui/contract/fail/C-10-async-constructor.rs");
    t.compile_fail("tests/ui/contract/fail/C-11-unsafe-constructor.rs");
    t.compile_fail("tests/ui/contract/fail/C-12-const-constructor.rs");
    t.compile_fail("tests/ui/contract/fail/C-13-abi-constructor.rs");
    t.compile_fail("tests/ui/contract/fail/C-14-payable-constructor.rs");
    t.compile_fail("tests/ui/contract/fail/C-15-non-codec-input.rs");
    t.compile_fail("tests/ui/contract/fail/C-16-function-arg-struct-destructuring.rs");

    t.compile_fail("tests/ui/contract/fail/H-01-invalid-dyn-alloc.rs");
    t.compile_fail("tests/ui/contract/fail/H-02-invalid-as-dependency.rs");
    t.compile_fail("tests/ui/contract/fail/H-03-use-forbidden-idents.rs");

    t.compile_fail("tests/ui/contract/fail/M-01-missing-message.rs");
    t.compile_fail("tests/ui/contract/fail/M-02-message-missing-self-arg.rs");
    t.compile_fail("tests/ui/contract/fail/M-03-message-returns-self.rs");
    t.compile_fail("tests/ui/contract/fail/M-04-message-returns-non-codec.rs");
    t.compile_fail("tests/ui/contract/fail/M-05-message-invalid-selector.rs");
    t.compile_fail("tests/ui/contract/fail/M-06-message-invalid-selector-type.rs");
    t.compile_fail("tests/ui/contract/fail/M-07-message-input-non-codec.rs");
    t.compile_fail("tests/ui/contract/fail/M-10-method-unknown-ink-marker.rs");

    t.compile_fail("tests/ui/contract/fail/S-01-missing-storage-struct.rs");
    t.compile_fail("tests/ui/contract/fail/S-02-multiple-storage-structs.rs");
    t.compile_fail("tests/ui/contract/fail/S-03-struct-unknown-ink-marker.rs");
    t.compile_fail("tests/ui/contract/fail/S-04-non-storage-ink-impls.rs");
    t.compile_fail("tests/ui/contract/fail/S-05-storage-as-event.rs");
    t.compile_fail("tests/ui/contract/fail/S-06-event-as-storage.rs");

    t.compile_fail("tests/ui/contract/fail/N-01-namespace-invalid-identifier.rs");
    t.compile_fail("tests/ui/contract/fail/N-02-namespace-invalid-type.rs");
    t.compile_fail("tests/ui/contract/fail/N-03-namespace-missing-argument.rs");
}

#[test]
fn chain_extension() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/chain_extension/E-01-simple.rs");
}

#[test]
fn trait_definition() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/trait_def/pass/simple_definition.rs");
    // t.pass("tests/ui/trait_def/pass/no-implicit-prelude.rs");
    t.pass("tests/ui/trait_def/pass/many_inputs.rs");
    t.pass("tests/ui/trait_def/pass/many_outputs.rs");
    t.pass("tests/ui/trait_def/pass/payable_message.rs");
    t.pass("tests/ui/trait_def/pass/custom_selector.rs");
    t.pass("tests/ui/trait_def/pass/with_namespace.rs");
    t.compile_fail("tests/ui/trait_def/fail/definition_empty.rs");
    t.compile_fail("tests/ui/trait_def/fail/definition_constructor.rs");
    t.compile_fail("tests/ui/trait_def/fail/definition_rust_method.rs");
    t.compile_fail("tests/ui/trait_def/fail/definition_assoc_type.rs");
    t.compile_fail("tests/ui/trait_def/fail/definition_assoc_const.rs");
    t.compile_fail("tests/ui/trait_def/fail/definition_super_trait_invalid_1.rs");
    t.compile_fail("tests/ui/trait_def/fail/definition_super_trait_invalid_2.rs");
    t.compile_fail("tests/ui/trait_def/fail/definition_non_pub.rs");
    t.compile_fail("tests/ui/trait_def/fail/definition_unsafe.rs");
    t.compile_fail("tests/ui/trait_def/fail/definition_generic.rs");
    t.compile_fail("tests/ui/trait_def/fail/config_namespace_invalid_1.rs");
    t.compile_fail("tests/ui/trait_def/fail/config_namespace_invalid_2.rs");
    t.compile_fail("tests/ui/trait_def/fail/config_namespace_invalid_3.rs");
    t.compile_fail("tests/ui/trait_def/fail/config_namespace_invalid_4.rs");
    t.compile_fail("tests/ui/trait_def/fail/message_selector_overlap.rs");
    t.compile_fail("tests/ui/trait_def/fail/message_selector_invalid_1.rs");
    t.compile_fail("tests/ui/trait_def/fail/message_selector_invalid_2.rs");
    t.compile_fail("tests/ui/trait_def/fail/message_payable_invalid_1.rs");
    t.compile_fail("tests/ui/trait_def/fail/message_payable_invalid_2.rs");
    t.compile_fail("tests/ui/trait_def/fail/message_receiver_missing.rs");
    t.compile_fail("tests/ui/trait_def/fail/message_receiver_invalid_1.rs");
    t.compile_fail("tests/ui/trait_def/fail/message_receiver_invalid_2.rs");
    t.compile_fail("tests/ui/trait_def/fail/message_receiver_invalid_3.rs");
    t.compile_fail("tests/ui/trait_def/fail/message_async_invalid.rs");
    t.compile_fail("tests/ui/trait_def/fail/message_const_invalid.rs");
    t.compile_fail("tests/ui/trait_def/fail/message_unsafe_invalid.rs");
    t.compile_fail("tests/ui/trait_def/fail/message_abi_invalid.rs");
    t.compile_fail("tests/ui/trait_def/fail/message_generic_invalid.rs");
    t.compile_fail("tests/ui/trait_def/fail/message_input_pattern_invalid.rs");
    t.compile_fail("tests/ui/trait_def/fail/message_default_impl.rs");
    t.compile_fail("tests/ui/trait_def/fail/message_constructor_conflict.rs");
}
