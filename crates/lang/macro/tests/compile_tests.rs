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
    // t.pass("tests/ui/contract/pass/minimal-contract.rs");
    // t.pass("tests/ui/contract/pass/constructor-many-inputs.rs");
    // t.pass("tests/ui/contract/pass/constructor-selector.rs");
    // t.pass("tests/ui/contract/pass/message-many-inputs.rs");
    // t.pass("tests/ui/contract/pass/message-many-outputs.rs");
    // t.pass("tests/ui/contract/pass/message-payable.rs");
    // t.pass("tests/ui/contract/pass/message-selector.rs");
    // t.pass("tests/ui/contract/pass/storage-single-field.rs");
    // t.pass("tests/ui/contract/pass/storage-many-fields.rs");
    // t.pass("tests/ui/contract/pass/storage-packed-fields.rs");
    // t.pass("tests/ui/contract/pass/storage-with-derives.rs");
    // t.pass("tests/ui/contract/pass/event-single-definition.rs");
    // t.pass("tests/ui/contract/pass/event-many-definitions.rs");
    // t.pass("tests/ui/contract/pass/event-topics.rs");
    // t.pass("tests/ui/contract/pass/event-anonymous.rs");
    // t.pass("tests/ui/contract/pass/event-config-more-topics.rs");
    // t.pass("tests/ui/contract/pass/impl-alias-storage.rs");
    // t.pass("tests/ui/contract/pass/impl-with-property.rs");
    // t.pass("tests/ui/contract/pass/impl-block-namespace.rs");
    // t.pass("tests/ui/contract/pass/config-compile-as-dependency-true.rs");
    // t.pass("tests/ui/contract/pass/config-compile-as-dependency-false.rs");
    // t.pass("tests/ui/contract/pass/config-dynamic-storage-allocator-true.rs");
    // t.pass("tests/ui/contract/pass/config-dynamic-storage-allocator-false.rs");
    // t.pass("tests/ui/contract/pass/config-custom-env.rs");
    // t.pass("tests/ui/contract/pass/env-access.rs");
    // t.pass("tests/ui/contract/pass/module-non-ink-items.rs");
    // t.pass("tests/ui/contract/pass/module-env-types.rs");
    // t.pass("tests/ui/contract/pass/trait-message-payable-guard.rs");
    // t.pass("tests/ui/contract/pass/trait-message-selector-guard.rs");
    // t.pass("tests/ui/contract/pass/example-flipper-works.rs");
    // t.pass("tests/ui/contract/pass/example-incrementer-works.rs");
    // t.pass("tests/ui/contract/pass/example-trait-flipper-works.rs");
    // t.pass("tests/ui/contract/pass/example-trait-incrementer-works.rs");
    // t.pass("tests/ui/contract/pass/example-erc20-works.rs");
    // t.pass("tests/ui/contract/pass/example-erc721-works.rs");

    t.compile_fail(
        "tests/ui/contract/fail/config-dynamic-storage-allocator-invalid-type-01.rs",
    );
    t.compile_fail(
        "tests/ui/contract/fail/config-dynamic-storage-allocator-invalid-type-02.rs",
    );
    t.compile_fail(
        "tests/ui/contract/fail/config-dynamic-storage-allocator-missing-arg.rs",
    );
    t.compile_fail(
        "tests/ui/contract/fail/config-compile-as-dependency-invalid-type-01.rs",
    );
    t.compile_fail(
        "tests/ui/contract/fail/config-compile-as-dependency-invalid-type-02.rs",
    );
    t.compile_fail("tests/ui/contract/fail/config-compile-as-dependency-missing-arg.rs");

    t.compile_fail("tests/ui/contract/fail/module-use-forbidden-idents.rs");
    t.compile_fail("tests/ui/contract/fail/module-missing-constructor.rs");
    t.compile_fail("tests/ui/contract/fail/module-missing-message.rs");
    t.compile_fail("tests/ui/contract/fail/module-missing-storage.rs");
    t.compile_fail("tests/ui/contract/fail/module-multiple-storages.rs");

    t.compile_fail("tests/ui/contract/fail/constructor-self-receiver-01.rs");
    t.compile_fail("tests/ui/contract/fail/constructor-self-receiver-02.rs");
    t.compile_fail("tests/ui/contract/fail/constructor-self-receiver-03.rs");
    t.compile_fail("tests/ui/contract/fail/constructor-self-receiver-04.rs");
    t.compile_fail("tests/ui/contract/fail/constructor-missing-return.rs");
    t.compile_fail("tests/ui/contract/fail/constructor-async.rs");
    t.compile_fail("tests/ui/contract/fail/constructor-unsafe.rs");
    t.compile_fail("tests/ui/contract/fail/constructor-const.rs");
    t.compile_fail("tests/ui/contract/fail/constructor-abi.rs");
    t.compile_fail("tests/ui/contract/fail/constructor-payable.rs");
    t.compile_fail("tests/ui/contract/fail/constructor-input-non-codec.rs");
    t.compile_fail("tests/ui/contract/fail/constructor-input-pattern.rs");

    t.compile_fail("tests/ui/contract/fail/message-input-pattern.rs");
    t.compile_fail("tests/ui/contract/fail/message-self-receiver-missing.rs");
    t.compile_fail("tests/ui/contract/fail/message-self-receiver-invalid-01.rs");
    t.compile_fail("tests/ui/contract/fail/message-self-receiver-invalid-02.rs");
    t.compile_fail("tests/ui/contract/fail/message-self-receiver-invalid-03.rs");
    t.compile_fail("tests/ui/contract/fail/message-returns-self.rs");
    t.compile_fail("tests/ui/contract/fail/message-returns-non-codec.rs");
    t.compile_fail("tests/ui/contract/fail/message-selector-invalid-type-01.rs");
    t.compile_fail("tests/ui/contract/fail/message-selector-invalid-type-02.rs");
    t.compile_fail("tests/ui/contract/fail/message-selector-missing-arg.rs");
    t.compile_fail("tests/ui/contract/fail/message-input-non-codec.rs");
    t.compile_fail("tests/ui/contract/fail/message-unknown-property.rs");

    t.compile_fail("tests/ui/contract/fail/impl-block-namespace-invalid-identifier.rs");
    t.compile_fail("tests/ui/contract/fail/impl-block-namespace-invalid-type.rs");
    t.compile_fail("tests/ui/contract/fail/impl-block-namespace-missing-argument.rs");
    t.compile_fail("tests/ui/contract/fail/impl-block-for-non-storage-01.rs");
    t.compile_fail("tests/ui/contract/fail/impl-block-for-non-storage-02.rs");

    t.compile_fail("tests/ui/contract/fail/trait-message-selector-mismatch.rs");
    t.compile_fail("tests/ui/contract/fail/trait-message-payable-mismatch.rs");
    t.compile_fail("tests/ui/contract/fail/trait-impl-namespace-invalid.rs");

    t.compile_fail("tests/ui/contract/fail/storage-unknown-marker.rs");
    t.compile_fail("tests/ui/contract/fail/storage-conflicting-event.rs");

    t.compile_fail("tests/ui/contract/fail/event-conflicting-storage.rs");
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
    t.compile_fail("tests/ui/trait_def/fail/message_input_non_codec.rs");
    t.compile_fail("tests/ui/trait_def/fail/message_output_non_codec.rs");
    t.compile_fail("tests/ui/trait_def/fail/message_default_impl.rs");
    t.compile_fail("tests/ui/trait_def/fail/message_constructor_conflict.rs");
}
