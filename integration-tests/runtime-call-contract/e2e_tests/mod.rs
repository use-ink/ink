mod runtime;

use super::*;
use ink_e2e::{
    subxt::dynamic::Value,
    ChainBackend,
    ContractsBackend,
};

type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// Just instantiate a contract using non-default runtime.
#[ink_e2e::test(backend(runtime_only(sandbox = runtime::ContractCallerSandbox)))]
async fn custom_runtime<Client: E2EBackend>(mut client: Client) -> E2EResult<()> {
    client
        .instantiate(
            "runtime-call-contract",
            &ink_e2e::alice(),
            &mut FlipperRef::new(false),
        )
        .submit()
        .await
        .expect("instantiate failed");

    Ok(())
}
