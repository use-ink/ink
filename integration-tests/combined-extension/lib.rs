#![cfg_attr(not(feature = "std"), no_std, no_main)]

use ink::env::{
    DefaultEnvironment,
    Environment,
};
use psp22_extension::Psp22Extension;
use rand_extension::{
    FetchRandom,
    RandomReadErr,
};

ink::combine_extensions! {
    /// This extension combines the [`FetchRandom`] and [`Psp22Extension`] extensions.
    /// It is possible to combine any number of extensions in this way.
    ///
    /// This structure is an instance that is returned by the `self.env().extension()` call.
    pub struct CombinedChainExtension {
        /// The instance of the [`Psp22Extension`] chain extension.
        ///
        /// It provides you access to `PSP22` functionality.
        pub psp22: Psp22Extension,
        /// The instance of the [`FetchRandom`] chain extension.
        ///
        /// It provides you access to randomness functionality.
        pub rand: FetchRandom,
    }
}

/// An environment using default ink environment types, with PSP-22 extension included
#[derive(Debug, Clone, PartialEq, Eq)]
#[ink::scale_derive(TypeInfo)]
pub enum CustomEnvironment {}

/// We use the same types and values as for [`DefaultEnvironment`] except the
/// [`Environment::ChainExtension`] type.
impl Environment for CustomEnvironment {
    const MAX_EVENT_TOPICS: usize = <DefaultEnvironment as Environment>::MAX_EVENT_TOPICS;

    type AccountId = <DefaultEnvironment as Environment>::AccountId;
    type Balance = <DefaultEnvironment as Environment>::Balance;
    type Hash = <DefaultEnvironment as Environment>::Hash;
    type Timestamp = <DefaultEnvironment as Environment>::Timestamp;
    type BlockNumber = <DefaultEnvironment as Environment>::BlockNumber;

    /// Setting up the combined chain extension as a primary extension.
    ///
    /// The `self.env().extension()` call returns the instance that provides access to
    /// both chain extensions.
    type ChainExtension = CombinedChainExtension;
}

#[ink::contract(env = crate::CustomEnvironment)]
mod combined_extension {
    use super::*;
    use psp22_extension::Psp22Error;

    /// Defines the storage of our contract.
    ///
    /// The example shows how to call each extension and test it,
    /// so we don't need any state to save.
    #[ink(storage)]
    #[derive(Default)]
    pub struct CombinedExtensionContract;

    impl CombinedExtensionContract {
        /// Constructor that initializes empty storage.
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {}
        }

        /// Returns the random value from extension.
        #[ink(message)]
        pub fn get_rand(&self) -> Result<[u8; 32], RandomReadErr> {
            self.env().extension().rand.fetch_random([0; 32] /* seed */)
        }

        /// Returns the total supply from PSP22 extension.
        #[ink(message)]
        pub fn get_total_supply(&self) -> Result<Balance, Psp22Error> {
            self.env().extension().psp22.total_supply(0)
        }
    }

    /// Unit tests in Rust are normally defined within such a `#[cfg(test)]`
    #[cfg(test)]
    mod tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        const RANDOM_VALUE: [u8; 32] = [3; 32];

        /// Mocking the random extension to return results that we want in the tests.
        struct MockedRandExtension;
        impl ink::env::test::ChainExtension for MockedRandExtension {
            fn ext_id(&self) -> u16 {
                // It is identifier used by [`rand_extension::FetchRandom`] extension.
                666
            }

            fn call(
                &mut self,
                _func_id: u16,
                _input: &[u8],
                output: &mut Vec<u8>,
            ) -> u32 {
                ink::scale::Encode::encode_to(&RANDOM_VALUE, output);
                0
            }
        }

        #[ink::test]
        fn rand_chain_extension_works() {
            let contract = CombinedExtensionContract::new();

            // given
            let result = std::panic::catch_unwind(|| contract.get_rand());
            // The call to random extension should fail because it is not registered.
            assert!(result.is_err());

            // when
            ink::env::test::register_chain_extension(MockedRandExtension);

            // then
            assert_eq!(contract.get_rand(), Ok(RANDOM_VALUE));
        }

        const TOTAL_SUPPLY: u128 = 1377;

        /// Mocking the PSP22 extension to return results that we want in the tests.
        ///
        /// Because this extension has many methods, we want to implement only one of
        /// them:
        /// - `total_supply` with corresponding `func_id` - `0x162d`.
        struct MockedPSP22Extension;
        impl ink::env::test::ChainExtension for MockedPSP22Extension {
            fn ext_id(&self) -> u16 {
                // It is identifier used by [`psp22_extension::Psp22Extension`] extension.
                13
            }

            fn call(&mut self, func_id: u16, _input: &[u8], output: &mut Vec<u8>) -> u32 {
                match func_id {
                    0x162d /* `func_id` of the `total_supply` function */ => {
                        ink::scale::Encode::encode_to(&TOTAL_SUPPLY, output);
                        0
                    }
                    _ => {
                        1
                    }
                }
            }
        }

        #[ink::test]
        fn psp22_chain_extension_works() {
            let contract = CombinedExtensionContract::new();

            // given
            let result = std::panic::catch_unwind(|| contract.get_total_supply());
            // The call to PSP22 extension should fail because it is not registered.
            assert!(result.is_err());

            // when
            ink::env::test::register_chain_extension(MockedPSP22Extension);

            // then
            assert_eq!(contract.get_total_supply(), Ok(TOTAL_SUPPLY));
        }

        #[ink::test]
        fn both_chain_extensions_work() {
            let contract = CombinedExtensionContract::new();

            // given
            assert!(std::panic::catch_unwind(|| contract.get_rand()).is_err());
            assert!(std::panic::catch_unwind(|| { contract.get_total_supply() }).is_err());

            // when
            ink::env::test::register_chain_extension(MockedRandExtension);
            ink::env::test::register_chain_extension(MockedPSP22Extension);

            // then
            assert_eq!(contract.get_rand(), Ok(RANDOM_VALUE));
            assert_eq!(contract.get_total_supply(), Ok(TOTAL_SUPPLY));
        }
    }
}
