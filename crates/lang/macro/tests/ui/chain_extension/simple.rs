use ink_env::Environment;
use ink_lang as ink;

/// Custom chain extension to read to and write from the runtime.
#[ink::chain_extension]
pub trait RuntimeReadWrite {
    type ErrorCode = ReadWriteErrorCode;

    /// Reads from runtime storage.
    #[ink(extension = 1, expect_ok)]
    fn read(key: &[u8]) -> Result<Vec<u8>, ReadWriteErrorCode>;
    /// Writes into runtime storage.
    #[ink(extension = 2, expect_ok)]
    fn write(key: &[u8], value: &[u8]) -> Result<(), ReadWriteErrorCode>;
}

/// The shared error code for the read write chain extension.
#[derive(Debug, Copy, Clone, PartialEq, Eq, scale::Encode, scale::Decode, scale_info::TypeInfo)]
pub enum ReadWriteErrorCode {
    KeyDoesNotExist,
    CannotWriteToThatKey,
}

impl ink_env::chain_extension::FromStatusCode for ReadWriteErrorCode {
    fn from_status_code(status_code: u32) -> Result<(), Self> {
        match status_code {
            0 => Ok(()),
            1 => Err(Self::KeyDoesNotExist),
            2 => Err(Self::CannotWriteToThatKey),
            _ => panic!("encountered unknown status code")
        }
    }
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
    use super::ReadWriteErrorCode;

    #[ink(storage)]
    pub struct ReadWriter {}

    impl ReadWriter {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {}
        }

        #[ink(message)]
        pub fn read(&self, key: Vec<u8>) -> Result<Vec<u8>, ReadWriteErrorCode> {
            self.env()
                .extension()
                .read(&key)
                .expect("encountered error while reading from runtime storage")
        }

        #[ink(message)]
        pub fn write(&self, key: Vec<u8>, value: Vec<u8>) -> Result<(), ReadWriteErrorCode> {
            self.env()
                .extension()
                .write(&key, &value)
                .expect("encountered error while writing to runtime storage")
        }
    }
}

fn main() {}
