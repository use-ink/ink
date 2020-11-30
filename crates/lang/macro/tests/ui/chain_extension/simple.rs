use ink_env::Environment;
use ink_lang as ink;

/// Custom chain extension to read to and write from the runtime.
#[ink::chain_extension]
pub trait RuntimeReadWrite {
    /// Reads from runtime storage.
    #[ink(extension = 1)]
    fn read(key: &[u8]) -> Vec<u8>;
    /// Writes into runtime storage.
    #[ink(extension = 2)]
    fn write(key: &[u8], value: &[u8]);
}

pub enum CustomEnvironment {}

impl Environment for CustomEnvironment {
    const MAX_EVENT_TOPICS: usize =
        <ink_env::DefaultEnvironment as Environment>::MAX_EVENT_TOPICS;

    type AccountId = <ink_env::DefaultEnvironment as Environment>::AccountId;
    type Balance = <ink_env::DefaultEnvironment as Environment>::Balance;
    type Hash = <ink_env::DefaultEnvironment as Environment>::Hash;
    type BlockNumber =
        <ink_env::DefaultEnvironment as Environment>::BlockNumber;
    type Timestamp = <ink_env::DefaultEnvironment as Environment>::Timestamp;

    type ChainExtension = RuntimeReadWrite;
}

#[ink::contract(env_types = crate::CustomEnvironment)]
mod read_writer {
    #[ink(storage)]
    pub struct ReadWriter {}

    impl ReadWriter {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {}
        }

        #[ink(message)]
        pub fn read(&self, key: Vec<u8>) -> Vec<u8> {
            self.env()
                .extension()
                .read(&key)
                .expect("encountered error while reading from runtime storage")
        }

        #[ink(message)]
        pub fn write(&self, key: Vec<u8>, value: Vec<u8>) {
            self.env()
                .extension()
                .write(&key, &value)
                .expect("encountered error while writing to runtime storage")
        }
    }
}

fn main() {}
