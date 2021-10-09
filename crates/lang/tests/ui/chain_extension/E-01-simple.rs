use ink_env::Environment;
use ink_lang as ink;

/// Custom chain extension to read to and write from the runtime.
#[ink::chain_extension]
pub trait RuntimeReadWrite {
    type ErrorCode = ReadWriteErrorCode;

    /// Reads from runtime storage.
    #[ink(extension = 1, returns_result = false)]
    fn read(key: &[u8]) -> Vec<u8>;

    /// Reads from runtime storage.
    ///
    /// Returns the number of bytes read and up to 32 bytes of the
    /// read value. Unused bytes in the output are set to 0.
    ///
    /// # Errors
    ///
    /// If the runtime storage cell stores a value that requires more than
    /// 32 bytes.
    #[ink(extension = 2)]
    fn read_small(key: &[u8]) -> Result<(u32, [u8; 32]), ReadWriteError>;

    /// Writes into runtime storage.
    #[ink(extension = 3, returns_result = false)]
    fn write(key: &[u8], value: &[u8]);

    /// Returns the access allowed for the key for the caller.
    #[ink(extension = 4, returns_result = false, handle_status = false)]
    fn access(key: &[u8]) -> Option<Access>;

    /// Unlocks previously aquired permission to access key.
    ///
    /// # Errors
    ///
    /// If the permission was not granted.
    #[ink(extension = 5, handle_status = false)]
    fn unlock_access(key: &[u8], access: Access) -> Result<(), UnlockAccessError>;
}

/// The shared error code for the read write chain extension.
#[derive(
    Debug, Copy, Clone, PartialEq, Eq, scale::Encode, scale::Decode, scale_info::TypeInfo,
)]
pub enum ReadWriteErrorCode {
    InvalidKey,
    CannotWriteToKey,
    CannotReadFromKey,
}

/// Returned by `read_small` in case there were too few bytes available in the buffer.
///
/// Provides the number of bytes required to read the storage cell.
#[derive(
    Debug, Copy, Clone, PartialEq, Eq, scale::Encode, scale::Decode, scale_info::TypeInfo,
)]
pub enum ReadWriteError {
    ErrorCode(ReadWriteErrorCode),
    BufferTooSmall { required_bytes: u32 },
}

impl From<ReadWriteErrorCode> for ReadWriteError {
    fn from(error_code: ReadWriteErrorCode) -> Self {
        Self::ErrorCode(error_code)
    }
}

impl From<scale::Error> for ReadWriteError {
    fn from(_: scale::Error) -> Self {
        panic!("encountered unexpected invalid SCALE encoding")
    }
}

/// Returned by `unlock_access` if permission to access key was not granted with reason.
#[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode, scale_info::TypeInfo)]
pub struct UnlockAccessError {
    reason: String,
}

impl From<scale::Error> for UnlockAccessError {
    fn from(_: scale::Error) -> Self {
        panic!("encountered unexpected invalid SCALE encoding")
    }
}

/// The kind of access allows for a storage cell.
#[derive(
    Debug, Copy, Clone, PartialEq, Eq, scale::Encode, scale::Decode, scale_info::TypeInfo,
)]
pub enum Access {
    ReadWrite,
    ReadOnly,
    WriteOnly,
}

impl ink_env::chain_extension::FromStatusCode for ReadWriteErrorCode {
    fn from_status_code(status_code: u32) -> Result<(), Self> {
        match status_code {
            0 => Ok(()),
            1 => Err(Self::InvalidKey),
            2 => Err(Self::CannotWriteToKey),
            3 => Err(Self::CannotReadFromKey),
            _ => panic!("encountered unknown status code"),
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
    type BlockNumber = <ink_env::DefaultEnvironment as Environment>::BlockNumber;
    type Timestamp = <ink_env::DefaultEnvironment as Environment>::Timestamp;
    type RentFraction = <ink_env::DefaultEnvironment as Environment>::RentFraction;

    type ChainExtension = RuntimeReadWrite;
}

#[ink::contract(env = crate::CustomEnvironment)]
mod read_writer {
    use super::{Access, ReadWriteErrorCode, ReadWriteError, UnlockAccessError};

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
        }

        #[ink(message)]
        pub fn read_small(&self, key: Vec<u8>) -> Result<(u32, [u8; 32]), ReadWriteError> {
            self.env()
                .extension()
                .read_small(&key)
        }

        #[ink(message)]
        pub fn write(
            &self,
            key: Vec<u8>,
            value: Vec<u8>,
        ) -> Result<(), ReadWriteErrorCode> {
            self.env()
                .extension()
                .write(&key, &value)
        }

        #[ink(message)]
        pub fn access(&self, key: Vec<u8>) -> Option<Access> {
            self.env()
                .extension()
                .access(&key)
        }

        #[ink(message)]
        pub fn unlock_access(&self, key: Vec<u8>, access: Access) -> Result<(), UnlockAccessError> {
            self.env()
                .extension()
                .unlock_access(&key, access)
        }
    }
}

fn main() {}
