use super::{
    Flipper,
    FlipperRef,
};
use ink_e2e::{
    ChainBackend,
    ContractsBackend,
};

type E2EResult<T> = Result<T, Box<dyn std::error::Error>>;

/// Just instantiate a contract using non-default runtime.
#[ink_e2e::test(backend(runtime_only(sandbox = sandbox_runtime::ContractCallerSandbox)))]
async fn instantiate_and_get<Client: E2EBackend>(mut client: Client) -> E2EResult<()> {
    use flipper_traits::Flip;

    let initial_value = false;
    let mut constructor = FlipperRef::new(initial_value);

    let contract = client
        .instantiate("runtime-call-contract", &ink_e2e::alice(), &mut constructor)
        .submit()
        .await
        .expect("instantiate failed");

    let mut call_builder = contract.call_builder::<Flipper>();
    let flip_dry_run = client
        .call(&ink_e2e::bob(), &call_builder.flip())
        .dry_run()
        .await?;
    let gas_required = flip_dry_run.exec_result.gas_required;
    // todo do same thing as above with storage_deposit_limit

    // call pallet dispatchable
    client
        .runtime_call(
            &ink_e2e::alice(),
            "ContractCaller",
            "contract_call_flip",
            vec![
                ink_e2e::subxt::dynamic::Value::from_bytes(contract.addr),
                ink_e2e::subxt::ext::scale_value::serde::to_value(
                    frame_support::weights::Weight::from_parts(
                        gas_required.ref_time(),
                        gas_required.proof_size(),
                    ),
                )
                .unwrap(),
            ],
        )
        .await
        .expect("runtime call failed");

    // now check that the flip was executed via the pallet
    let get_result = client
        .call(&ink_e2e::alice(), &call_builder.get())
        .dry_run()
        .await?;

    assert_eq!(get_result.return_value(), !initial_value);

    Ok(())
}
