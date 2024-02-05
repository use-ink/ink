#![cfg_attr(not(feature = "std"), no_std, no_main)]

use ink::env::Environment;

/// This is an example of how an ink! contract may call the Substrate
/// runtime function `RandomnessCollectiveFlip::random_seed`. See the
/// file `runtime/chain-extension-example.rs` for that implementation.
///
/// Here we define the operations to interact with the Substrate runtime.
#[ink::chain_extension(extension = 666)]
pub trait FetchRandom {
    type ErrorCode = RandomReadErr;

    /// Note: this gives the operation a corresponding `func_id` (1101 in this case),
    /// and the chain-side chain extension will get the `func_id` to do further
    /// operations.
    #[ink(function = 1101)]
    fn fetch_random(subject: [u8; 32]) -> [u8; 32];
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub enum RandomReadErr {
    FailGetRandomSource,
}

impl ink::env::chain_extension::FromStatusCode for RandomReadErr {
    fn from_status_code(status_code: u32) -> Result<(), Self> {
        match status_code {
            0 => Ok(()),
            1 => Err(Self::FailGetRandomSource),
            _ => panic!("encountered unknown status code"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[ink::scale_derive(TypeInfo)]
pub enum CustomEnvironment {}

impl Environment for CustomEnvironment {
    const MAX_EVENT_TOPICS: usize =
        <ink::env::DefaultEnvironment as Environment>::MAX_EVENT_TOPICS;

    type AccountId = <ink::env::DefaultEnvironment as Environment>::AccountId;
    type Balance = <ink::env::DefaultEnvironment as Environment>::Balance;
    type Hash = <ink::env::DefaultEnvironment as Environment>::Hash;
    type BlockNumber = <ink::env::DefaultEnvironment as Environment>::BlockNumber;
    type Timestamp = <ink::env::DefaultEnvironment as Environment>::Timestamp;

    type ChainExtension = FetchRandom;
}

#[ink::contract(env = crate::CustomEnvironment)]
mod rand_extension {
    use super::RandomReadErr;

    /// Defines the storage of our contract.
    ///
    /// Here we store the random seed fetched from the chain.
    #[ink(storage)]
    pub struct RandExtension {
        /// Stores a single `bool` value on the storage.
        value: [u8; 32],
    }

    #[ink(event)]
    pub struct RandomUpdated {
        #[ink(topic)]
        new: [u8; 32],
    }

    impl RandExtension {
        /// Constructor that initializes the `bool` value to the given `init_value`.
        #[ink(constructor)]
        pub fn new(init_value: [u8; 32]) -> Self {
            Self { value: init_value }
        }

        /// Constructor that initializes the `bool` value to `false`.
        ///
        /// Constructors may delegate to other constructors.
        #[ink(constructor)]
        pub fn new_default() -> Self {
            Self::new(Default::default())
        }

        /// Seed a random value by passing some known argument `subject` to the runtime's
        /// random source. Then, update the current `value` stored in this contract with
        /// the new random value.
        #[ink(message)]
        pub fn update(&mut self, subject: [u8; 32]) -> Result<(), RandomReadErr> {
            // Get the on-chain random seed
            let new_random = self.env().extension().fetch_random(subject)?;
            self.value = new_random;
            // Emit the `RandomUpdated` event when the random seed
            // is successfully fetched.
            self.env().emit_event(RandomUpdated { new: new_random });
            Ok(())
        }

        /// Simply returns the current value.
        #[ink(message)]
        pub fn get(&self) -> [u8; 32] {
            self.value
        }
    }

    /// Unit tests in Rust are normally defined within such a `#[cfg(test)]`
    #[cfg(test)]
    mod tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        /// We test if the default constructor does its job.
        #[ink::test]
        fn default_works() {
            let rand_extension = RandExtension::new_default();
            assert_eq!(rand_extension.get(), [0; 32]);
        }

        #[ink::test]
        fn chain_extension_works() {
            // given
            struct MockedRandExtension;
            impl ink::env::test::ChainExtension for MockedRandExtension {
                /// The static function id of the chain extension.
                fn ext_id(&self) -> u16 {
                    666
                }

                /// The chain extension is called with the given input.
                ///
                /// Returns an error code and may fill the `output` buffer with a
                /// SCALE encoded result. The error code is taken from the
                /// `ink::env::chain_extension::FromStatusCode` implementation for
                /// `RandomReadErr`.
                fn call(
                    &mut self,
                    _func_id: u16,
                    _input: &[u8],
                    output: &mut Vec<u8>,
                ) -> u32 {
                    let ret: [u8; 32] = [1; 32];
                    ink::scale::Encode::encode_to(&ret, output);
                    0
                }
            }
            ink::env::test::register_chain_extension(MockedRandExtension);
            let mut rand_extension = RandExtension::new_default();
            assert_eq!(rand_extension.get(), [0; 32]);

            // when
            rand_extension.update([0_u8; 32]).expect("update must work");

            // then
            assert_eq!(rand_extension.get(), [1; 32]);
        }
    }
}
