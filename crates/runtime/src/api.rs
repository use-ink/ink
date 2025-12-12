pub mod assets_api;
pub mod balance_api;
pub mod nfts_api;
pub mod revive_api;
pub mod system_api;
pub mod timestamp_api;

pub mod prelude {
    pub use super::{
        assets_api::AssetsAPI,
        balance_api::BalanceAPI,
        nfts_api::NftsAPI,
        revive_api::ContractAPI,
        system_api::SystemAPI,
        timestamp_api::TimestampAPI,
    };
}
