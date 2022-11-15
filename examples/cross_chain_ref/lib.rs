#![cfg_attr(not(feature = "std"), no_std)]

#[ink::contract]
mod cross_chain_ref {
    use flipper::FlipperRef;

    #[ink(storage)]
    pub struct CrossChainRef {
        flipper: FlipperRef,
    }

    impl CrossChainRef {
        #[ink(constructor)]
        pub fn new(version: u32, flipper_code_hash: Hash) -> Self {
            let salt = version.to_le_bytes();
            let flipper = FlipperRef::default()
                .endowment(0)
                .code_hash(flipper_code_hash)
                .salt_bytes(salt)
                .instantiate()
                .unwrap_or_else(|error| {
                    panic!("failed at instantiating the Flipper contract: {:?}", error)
                });

            Self { flipper }
        }

        #[ink(message)]
        pub fn flip(&mut self) {
            self.flipper.flip();
        }

        #[ink(message)]
        pub fn flip_check(&mut self) {
            self.flipper.flip_checked().unwrap();
        }

        #[ink(message)]
        pub fn get(&mut self) -> bool {
            self.flipper.get()
        }

        #[ink(message)]
        pub fn get_check(&mut self) -> bool {
            self.flipper.get_checked().unwrap()
        }
    }
}
