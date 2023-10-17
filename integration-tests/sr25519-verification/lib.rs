#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
pub mod sr25519_verification {
    #[ink(storage)]
    #[derive(Default)]
    pub struct Sr25519Verification {}

    impl Sr25519Verification {
        /// Creates a new sr25519 verification smart contract initialized with the given
        /// value.
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {}
        }

        /// Dummy method to satisfy constaint of more than 0 messages per contract.
        #[ink(message)]
        pub fn foobar(&self) {
            ink::env::debug_println!("foobar - this method does nothing");
        }
    }

    #[cfg(test)]
    mod tests {

        #[ink::test]
        fn test_sr25519_verify_valid() {
            // "<Bytes>hello<Bytes>" as bytes
            let message: [u8; 49] = [
                60, 66, 121, 116, 101, 115, 62, 48, 120, 52, 54, 102, 98, 55, 52, 48, 56,
                100, 52, 102, 50, 56, 53, 50, 50, 56, 102, 52, 97, 102, 53, 49, 54, 101,
                97, 50, 53, 56, 53, 49, 98, 60, 47, 66, 121, 116, 101, 115, 62,
            ];
            // alice
            let public_key: [u8; 32] = [
                212, 53, 147, 199, 21, 253, 211, 28, 97, 20, 26, 189, 4, 169, 159, 214,
                130, 44, 133, 88, 133, 76, 205, 227, 154, 86, 132, 231, 165, 109, 162,
                125,
            ];
            // alice's signature of the message
            let signature: [u8; 64] = [
                10, 125, 162, 182, 49, 112, 76, 220, 254, 147, 199, 64, 228, 18, 23, 185,
                172, 102, 122, 12, 135, 85, 216, 218, 26, 130, 50, 219, 82, 127, 72, 124,
                135, 231, 128, 210, 237, 193, 137, 106, 235, 107, 27, 239, 11, 199, 195,
                141, 157, 242, 19, 91, 99, 62, 171, 139, 251, 23, 119, 232, 47, 173, 58,
                143,
            ];
            let result = ink::env::sr25519_verify(&signature, &message, &public_key);
            assert_eq!(result, Ok(()));
        }

        #[ink::test]
        fn test_sr25519_verify_invalid_public_key() {
            // "<Bytes>hello<Bytes>" as bytes
            let message: [u8; 49] = [
                60, 66, 121, 116, 101, 115, 62, 48, 120, 52, 54, 102, 98, 55, 52, 48, 56,
                100, 52, 102, 50, 56, 53, 50, 50, 56, 102, 52, 97, 102, 53, 49, 54, 101,
                97, 50, 53, 56, 53, 49, 98, 60, 47, 66, 121, 116, 101, 115, 62,
            ];
            // alice - off by 1 at start
            let public_key: [u8; 32] = [
                213, 53, 147, 199, 21, 253, 211, 28, 97, 20, 26, 189, 4, 169, 159, 214,
                130, 44, 133, 88, 133, 76, 205, 227, 154, 86, 132, 231, 165, 109, 162,
                125,
            ];
            // alice's signature of the message
            let signature: [u8; 64] = [
                10, 125, 162, 182, 49, 112, 76, 220, 254, 147, 199, 64, 228, 18, 23, 185,
                172, 102, 122, 12, 135, 85, 216, 218, 26, 130, 50, 219, 82, 127, 72, 124,
                135, 231, 128, 210, 237, 193, 137, 106, 235, 107, 27, 239, 11, 199, 195,
                141, 157, 242, 19, 91, 99, 62, 171, 139, 251, 23, 119, 232, 47, 173, 58,
                143,
            ];
            let result = ink::env::sr25519_verify(&signature, &message, &public_key);
            assert_eq!(result, Err(ink::env::Error::Sr25519VerifyFailed));
        }

        #[ink::test]
        fn test_sr25519_verify_invalid_message() {
            // "<Bytes>hello<Bytes>" as bytes - off by 1 at start
            let message: [u8; 49] = [
                61, 66, 121, 116, 101, 115, 62, 48, 120, 52, 54, 102, 98, 55, 52, 48, 56,
                100, 52, 102, 50, 56, 53, 50, 50, 56, 102, 52, 97, 102, 53, 49, 54, 101,
                97, 50, 53, 56, 53, 49, 98, 60, 47, 66, 121, 116, 101, 115, 62,
            ];
            // alice
            let public_key: [u8; 32] = [
                212, 53, 147, 199, 21, 253, 211, 28, 97, 20, 26, 189, 4, 169, 159, 214,
                130, 44, 133, 88, 133, 76, 205, 227, 154, 86, 132, 231, 165, 109, 162,
                125,
            ];
            // alice's signature of the message
            let signature: [u8; 64] = [
                10, 125, 162, 182, 49, 112, 76, 220, 254, 147, 199, 64, 228, 18, 23, 185,
                172, 102, 122, 12, 135, 85, 216, 218, 26, 130, 50, 219, 82, 127, 72, 124,
                135, 231, 128, 210, 237, 193, 137, 106, 235, 107, 27, 239, 11, 199, 195,
                141, 157, 242, 19, 91, 99, 62, 171, 139, 251, 23, 119, 232, 47, 173, 58,
                143,
            ];
            let result = ink::env::sr25519_verify(&signature, &message, &public_key);
            assert_eq!(result, Err(ink::env::Error::Sr25519VerifyFailed));
        }

        #[ink::test]
        fn test_sr25519_verify_invalid_signature() {
            // "<Bytes>hello<Bytes>" as bytes
            let message: [u8; 49] = [
                60, 66, 121, 116, 101, 115, 62, 48, 120, 52, 54, 102, 98, 55, 52, 48, 56,
                100, 52, 102, 50, 56, 53, 50, 50, 56, 102, 52, 97, 102, 53, 49, 54, 101,
                97, 50, 53, 56, 53, 49, 98, 60, 47, 66, 121, 116, 101, 115, 62,
            ];
            // alice
            let public_key: [u8; 32] = [
                212, 53, 147, 199, 21, 253, 211, 28, 97, 20, 26, 189, 4, 169, 159, 214,
                130, 44, 133, 88, 133, 76, 205, 227, 154, 86, 132, 231, 165, 109, 162,
                125,
            ];
            // alice's signature of the message - off by 1 at start
            let signature: [u8; 64] = [
                11, 125, 162, 182, 49, 112, 76, 220, 254, 147, 199, 64, 228, 18, 23, 185,
                172, 102, 122, 12, 135, 85, 216, 218, 26, 130, 50, 219, 82, 127, 72, 124,
                135, 231, 128, 210, 237, 193, 137, 106, 235, 107, 27, 239, 11, 199, 195,
                141, 157, 242, 19, 91, 99, 62, 171, 139, 251, 23, 119, 232, 47, 173, 58,
                143,
            ];
            let result = ink::env::sr25519_verify(&signature, &message, &public_key);
            assert_eq!(result, Err(ink::env::Error::Sr25519VerifyFailed));
        }
    }
}
