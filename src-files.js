var srcIndex = JSON.parse('{\
"ink":["",[["codegen",[["dispatch",[],["execution.rs","info.rs","mod.rs","type_check.rs"]],["trait_def",[],["call_builder.rs","mod.rs","trait_message.rs"]],["utils",[],["identity_type.rs","mod.rs","same_type.rs"]]],["env.rs","implies_return.rs","mod.rs"]],["reflect",[["trait_def",[],["info.rs","mod.rs","registry.rs"]]],["contract.rs","dispatch.rs","mod.rs"]]],["chain_extension.rs","contract_ref.rs","env_access.rs","lib.rs","message_builder.rs","option_info.rs","result_info.rs"]],\
"ink_allocator":["",[],["bump.rs","lib.rs"]],\
"ink_codegen":["",[["generator",[["as_dependency",[],["call_builder.rs","contract_ref.rs","mod.rs"]],["trait_def",[],["call_builder.rs","call_forwarder.rs","definition.rs","message_builder.rs","mod.rs","trait_registry.rs"]]],["arg_list.rs","blake2b.rs","chain_extension.rs","contract.rs","dispatch.rs","env.rs","event.rs","ink_test.rs","item_impls.rs","metadata.rs","mod.rs","selector.rs","storage.rs","storage_item.rs"]]],["enforced_error.rs","lib.rs","traits.rs"]],\
"ink_e2e":["",[],["backend.rs","backend_calls.rs","builders.rs","client_utils.rs","contract_build.rs","contract_results.rs","error.rs","events.rs","lib.rs","node_proc.rs","sandbox_client.rs","subxt_client.rs","xts.rs"]],\
"ink_e2e_macro":["",[],["codegen.rs","config.rs","ir.rs","lib.rs"]],\
"ink_engine":["",[],["chain_extension.rs","database.rs","exec_context.rs","ext.rs","hashing.rs","lib.rs","test_api.rs","types.rs"]],\
"ink_env":["",[["call",[],["call_builder.rs","common.rs","create_builder.rs","execution.rs","mod.rs","selector.rs"]],["engine",[["off_chain",[],["call_data.rs","impls.rs","mod.rs","test_api.rs","types.rs"]]],["mod.rs"]]],["api.rs","arithmetic.rs","backend.rs","chain_extension.rs","contract.rs","error.rs","event.rs","hash.rs","lib.rs","types.rs"]],\
"ink_ir":["",[["ast",[],["attr_args.rs","meta.rs","mod.rs"]],["ir",[["event",[],["config.rs","mod.rs","signature_topic.rs"]],["item",[],["mod.rs","storage.rs"]],["item_impl",[],["callable.rs","constructor.rs","impl_item.rs","iter.rs","message.rs","mod.rs"]],["storage_item",[],["config.rs","mod.rs"]],["trait_def",[["item",[],["iter.rs","mod.rs","trait_item.rs"]]],["config.rs","mod.rs"]]],["attrs.rs","blake2.rs","chain_extension.rs","config.rs","contract.rs","idents_lint.rs","ink_test.rs","item_mod.rs","mod.rs","selector.rs","utils.rs"]]],["error.rs","lib.rs","literal.rs"]],\
"ink_macro":["",[["event",[],["metadata.rs","mod.rs"]],["storage",[],["mod.rs","storable.rs","storable_hint.rs","storage_key.rs","storage_layout.rs"]]],["blake2b.rs","chain_extension.rs","contract.rs","ink_test.rs","lib.rs","scale.rs","selector.rs","storage_item.rs","trait_def.rs"]],\
"ink_metadata":["",[["layout",[],["mod.rs","validate.rs"]]],["lib.rs","specs.rs","utils.rs"]],\
"ink_prelude":["",[],["lib.rs"]],\
"ink_primitives":["",[],["key.rs","lib.rs","types.rs"]],\
"ink_sandbox":["",[["api",[],["balance_api.rs","contracts_api.rs","system_api.rs","timestamp_api.rs"]]],["api.rs","lib.rs","macros.rs"]],\
"ink_storage":["",[["lazy",[],["mapping.rs","mod.rs","vec.rs"]]],["lib.rs"]],\
"ink_storage_traits":["",[["impls",[],["mod.rs"]],["layout",[],["impls.rs","mod.rs"]]],["lib.rs","storage.rs"]]\
}');
createSrcSidebar();