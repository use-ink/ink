use jsonrpsee::core::async_trait;

/// Full E2E testing backend: combines general chain API and contract-specific operations.
#[async_trait]
pub trait E2EBackend: ChainBackend + ContractsBackend {}

/// General chain operations useful in contract testing.
#[async_trait]
pub trait ChainBackend {}

/// Contract-specific operations.
#[async_trait]
pub trait ContractsBackend {}
