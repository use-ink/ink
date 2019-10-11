#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang::contract;
use scale::KeyedVec as _;

contract! {
    #![env = ink_core::env::DefaultSrmlTypes]

    /// This simple contract reads a value from runtime storage
    struct Runtime {
    }

    impl Deploy for Runtime {
        fn deploy(&mut self) {
        }
    }

    impl Runtime {
        /// Returns the account balance, read directly from runtime storage
        pub(external) fn get_balance(&self, account: AccountId) -> Balance {
            const BALANCE_OF: &[u8] = b"balance:";
            let key = account.to_keyed_vec(BALANCE_OF);
            env.runtime_get_storage::<Balance>(&key)
                .expect("account key should have a balance")
                .expect("should decode runtime storage balance")
        }
    }
}

#[cfg(all(test, feature = "test-env"))]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let mut contract = Runtime::deploy_mock();
        assert_eq!(contract.get(), false);
        contract.flip();
        assert_eq!(contract.get(), true);
    }
}
