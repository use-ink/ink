#![cfg_attr(not(feature = "std"), no_std, no_main)]

// TODO:
// - need node port for starting eth rpc and hardhat
// - need to run eth-rpc, which docker needs binary for from polkadot-sdk
//   - example cmd: eth-rpc --dev --node-rpc-url ws://127.0.0.1:55615
// - need to update docker to use an earlier resolc version
// - returning data to solidity contract currently does not work

#[ink::contract(abi_encoding = "rlp")]
pub mod flipper {
    #[ink(storage)]
    pub struct Flipper {
        value: bool,
    }

    impl Flipper {
        /// Creates a new flipper smart contract initialized with the given value.
        #[ink(constructor)]
        pub fn new(init_value: bool) -> Self {
            Self { value: init_value }
        }

        /// Creates a new flipper smart contract initialized to `false`.
        #[ink(constructor)]
        pub fn new_default() -> Self {
            Self::new(Default::default())
        }

        // solidity compatible selector (`keccack256("flip()")`)
        #[ink(message, selector = 0xcde4efa9)]
        pub fn flip(&mut self) {
            self.value = !self.value;
        }

        #[ink(message)]
        pub fn flip_2(&mut self) {
            self.value = !self.value;
        }

        /// Returns the current value of the Flipper's boolean.
        #[ink(message)]
        pub fn get(&self) -> bool {
            self.value
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[ink::test]
        fn default_works() {
            let flipper = Flipper::new_default();
            assert!(!flipper.get());
        }

        #[ink::test]
        fn it_works() {
            let mut flipper = Flipper::new(false);
            assert!(!flipper.get());
            flipper.flip();
            assert!(flipper.get());
        }
    }

    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        use super::*;
        use ink_e2e::{
            subxt_signer,
            Weight,
        };
        use std::{
            env,
            process::Command,
        };

        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        #[ink_e2e::test]
        async fn it_works<Client: E2EBackend>(mut client: Client) -> E2EResult<()> {
            let constructor = FlipperRef::new(false);
            let params = constructor
                .endowment(0u32.into())
                .code_hash(ink::primitives::H256::zero())
                .salt_bytes(None)
                .params();
            let exec_input = params.exec_input();

            let alith = eth_account(subxt_signer::eth::dev::alith());

            client
                .api
                .try_transfer_balance(
                    &ink_e2e::alice(),
                    alith.0.into(),
                    1_000_000_000_000_000,
                )
                .await?;

            let signer = ink_e2e::alice();
            client.api.map_account(&signer).await;
            let res = client
                .exec_instantiate(
                    &signer,
                    client.contracts.load_code("flipper"),
                    ink::scale::Encode::encode(&exec_input),
                    0,
                    Weight::from_parts(100_000_000_000, 1024 * 1024).into(),
                    10_000_000_000_000,
                )
                .await?;
            println!("instantiated contract at {:?}", res.addr);

            // TODO: currently used to deploy solidity
            loop {}

            // Store the original working directory
            let original_dir = env::current_dir()?;

            // Change to the Solidity project directory
            println!("Changing to ./sol directory...");
            env::set_current_dir("./sol")?;

            println!("Starting Hardhat compilation...");

            // Execute 'npx hardhat compile'
            let compile_status =
                Command::new("npx").arg("hardhat").arg("compile").status()?;

            if !compile_status.success() {
                // Change back to original directory before returning error
                env::set_current_dir(original_dir)?;
                return Err("Hardhat compilation failed".into());
            }

            println!("Compilation successful. Starting deployment...");

            // Execute 'npx hardhat run deploy.js --network localhost --no-compile'
            let deploy_status = Command::new("npx")
                .arg("hardhat")
                .arg("run")
                .arg("deploy.js")
                .arg("--network")
                .arg("localhost")
                .arg("--no-compile")
                .status()?;

            // Change back to original directory
            env::set_current_dir(original_dir)?;

            if !deploy_status.success() {
                return Err("Deployment failed".into());
            }

            println!("Deployment completed successfully!");

            Ok(())
        }

        // borrowed from: https://github.com/paritytech/polkadot-sdk/blob/master/substrate/bin/node/cli/src/chain_spec.rs#L427
        fn eth_account(from: subxt_signer::eth::Keypair) -> AccountId {
            let mut account_id = crate::flipper::AccountId::from([0xEE; 32]);
            <AccountId as AsMut<[u8; 32]>>::as_mut(&mut account_id)[..20]
                .copy_from_slice(&from.public_key().to_account_id().as_ref());
            account_id
        }
    }
}
