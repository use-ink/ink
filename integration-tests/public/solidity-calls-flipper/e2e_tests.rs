use crate::flipper::FlipperRef;
use ink::{
    alloy_sol_types::{
        SolType,
        SolValue,
    },
    env::DefaultEnvironment,
    primitives::AccountId,
    H160,
};
use ink_e2e::{
    subxt::tx::Signer,
    subxt_signer,
    PolkadotConfig,
    Weight,
};
use std::{
    error::Error,
    process::{
        Command,
        Stdio,
    },
};

const DEFAULT_GAS: Weight = Weight::from_parts(100_000_000_000, 1024 * 1024);
const DEFAULT_STORAGE_DEPOSIT_LIMIT: u128 = 10_000_000_000_000;
type E2EResult<T> = Result<T, Box<dyn Error>>;

#[ink_e2e::test]
async fn solidity_calls_ink_works<Client: E2EBackend>(
    mut client: Client,
) -> E2EResult<()> {
    let constructor = FlipperRef::new(false);
    let params = constructor
        .endowment(0u32.into())
        .code_hash(ink::primitives::H256::zero())
        .salt_bytes(None)
        .params();
    let exec_input = params.exec_input();

    // fund alith
    let alith = eth_account(subxt_signer::eth::dev::alith());
    client
        .api
        .try_transfer_balance(&ink_e2e::alice(), alith.0.into(), 1_000_000_000_000_000)
        .await?;

    let signer = ink_e2e::alice();

    // deploy ink! flipper (Sol encoded)
    client.api.map_account(&signer).await;
    let ink_addr = client
        .exec_instantiate(
            &signer,
            client.contracts.load_code("flipper"),
            ink::scale::Encode::encode(&exec_input),
            0,
            DEFAULT_GAS.into(),
            DEFAULT_STORAGE_DEPOSIT_LIMIT,
        )
        .await?
        .addr;

    let get_selector = keccak_selector(b"get");
    let value: bool = call_ink(&mut client, ink_addr, get_selector.clone()).await;
    assert_eq!(value, false);

    let mut sol_handler = SolidityHandler::new("./sol".into(), client.url().to_string());
    sol_handler.start_eth_rpc()?;
    sol_handler.setup()?;
    let sol_addr = sol_handler.deploy(ink_addr)?;

    let _ = sol_handler.call(sol_addr.clone(), "callFlip")?;
    let value: bool = call_ink(&mut client, ink_addr, get_selector.clone()).await;
    assert_eq!(value, true);

    let output = sol_handler.call(sol_addr.clone(), "callGet")?;
    assert_eq!(output, Some("true".to_string()));

    let _ = sol_handler.call(sol_addr.clone(), "callFlip2")?;
    let value: bool = call_ink(&mut client, ink_addr, get_selector.clone()).await;
    assert_eq!(value, false);

    let _ = sol_handler.call_with_value(sol_addr.clone(), "callSet", true)?;

    let output = sol_handler.call(sol_addr.clone(), "callGet")?;
    assert_eq!(output, Some("true".to_string()));

    let output = sol_handler.call(sol_addr.clone(), "callGet2")?;
    assert_eq!(output, Some("true".to_string()));

    let _ = sol_handler.call_with_value(sol_addr.clone(), "callSet", false)?;

    let output = sol_handler.call(sol_addr, "callGet")?;
    assert_eq!(output, Some("false".to_string()));

    Ok(())
}

async fn call_ink<Ret>(
    client: &mut ink_e2e::Client<PolkadotConfig, DefaultEnvironment>,
    ink_addr: H160,
    data_sol: Vec<u8>,
) -> Ret
where
    Ret: SolValue + From<<<Ret as SolValue>::SolType as SolType>::RustType>,
{
    let signer = ink_e2e::alice();
    let exec_result = client
        .api
        .call_dry_run(
            <ink_e2e::Keypair as Signer<PolkadotConfig>>::account_id(&signer),
            ink_addr,
            data_sol,
            0,
            0,
        )
        .await;

    <Ret>::abi_decode(&mut &exec_result.result.unwrap().data[..], true)
        .expect("decode failed")
}

struct SolidityHandler {
    sol_dir: String,
    node_url: String,
    eth_rpc_process_id: Option<u32>,
}

impl SolidityHandler {
    fn new(sol_dir: String, node_url: String) -> Self {
        Self {
            sol_dir,
            node_url,
            eth_rpc_process_id: None,
        }
    }

    fn start_eth_rpc(&mut self) -> Result<(), Box<dyn Error>> {
        let eth_rpc = Command::new("eth-rpc")
            .arg("--dev")
            .arg("--node-rpc-url")
            .arg(&self.node_url)
            .spawn()?;
        self.eth_rpc_process_id = Some(eth_rpc.id());
        Ok(())
    }

    fn stop_eth_rpc(&mut self) -> Result<(), Box<dyn Error>> {
        if self.eth_rpc_process_id.is_none() {
            return Ok(());
        }
        Command::new("kill")
            .arg("-9")
            .arg(&self.eth_rpc_process_id.unwrap().to_string())
            .spawn()?;
        Ok(())
    }

    fn setup(&mut self) -> Result<(), Box<dyn Error>> {
        let install_status = Command::new("npm")
            .current_dir(&self.sol_dir)
            .arg("install")
            .arg("--save-dev")
            .status()?;
        assert!(install_status.success(), "npm install failed");

        let compile_status = Command::new("npx")
            .current_dir(&self.sol_dir)
            .arg("hardhat")
            .arg("compile")
            .arg("--config")
            .arg("hardhat.config.js")
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()?;
        assert!(compile_status.success(), "hardhat compilation failed");

        Ok(())
    }

    fn run_hardhat_script(
        &self,
        script: &str,
        env_vars: &[(&str, String)],
        is_piped: Stdio,
    ) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut command = Command::new("npx");
        command
            .current_dir(&self.sol_dir)
            .arg("hardhat")
            .arg("run")
            .arg(script)
            .arg("--network")
            .arg("localhost")
            .arg("--no-compile")
            .arg("--config")
            .arg("hardhat.config.js")
            .stdout(is_piped) // Capture stdout
            .stderr(Stdio::inherit()); // Print stderr

        // Add environment variables
        for (key, value) in env_vars {
            command.env(key, value);
        }

        let output = command.spawn()?.wait_with_output()?;
        assert!(
            output.status.success(),
            "{} execution failed with env: {:?}",
            script,
            env_vars
        );

        Ok(output.stdout)
    }

    fn deploy(&self, ink_addr: H160) -> Result<String, Box<dyn Error>> {
        let output = self.run_hardhat_script(
            "01-deploy.js",
            &[("INK_ADDRESS", format!("{:?}", ink_addr))],
            Stdio::piped(),
        )?;
        Ok(String::from_utf8(output)?
            .lines()
            .last()
            .ok_or("Failed to retrieve contract address")?
            .trim()
            .to_string())
    }

    fn call(
        &self,
        sol_addr: String,
        message: &str,
    ) -> Result<Option<String>, Box<dyn Error>> {
        let output = self.run_hardhat_script(
            "02-call-flip.js",
            &[("SOL_ADDRESS", sol_addr), ("MESSAGE", message.to_string())],
            Stdio::piped(),
        )?;
        Ok(String::from_utf8(output)
            .ok()
            .and_then(|s| Some(s.lines().last()?.to_string()))
            .map(|s| s.trim().to_string()))
    }

    fn call_with_value(
        &self,
        sol_addr: String,
        message: &str,
        value: bool,
    ) -> Result<Option<String>, Box<dyn Error>> {
        let output = self.run_hardhat_script(
            "02-call-flip.js",
            &[
                ("SOL_ADDRESS", sol_addr),
                ("MESSAGE", message.to_string()),
                ("VALUE", value.to_string()),
            ],
            Stdio::piped(),
        )?;
        Ok(String::from_utf8(output)
            .ok()
            .and_then(|s| Some(s.lines().last()?.to_string()))
            .map(|s| s.trim().to_string()))
    }
}

impl Drop for SolidityHandler {
    fn drop(&mut self) {
        self.stop_eth_rpc().unwrap();
    }
}

// borrowed from: https://github.com/paritytech/polkadot-sdk/blob/master/substrate/bin/node/cli/src/chain_spec.rs#L427
fn eth_account(from: subxt_signer::eth::Keypair) -> AccountId {
    let mut account_id = AccountId::from([0xEE; 32]);
    <AccountId as AsMut<[u8; 32]>>::as_mut(&mut account_id)[..20]
        .copy_from_slice(&from.public_key().to_account_id().as_ref());
    account_id
}

fn keccak_selector(input: &[u8]) -> Vec<u8> {
    let mut output = [0; 32];
    use sha3::{
        digest::generic_array::GenericArray,
        Digest as _,
    };
    let mut hasher = sha3::Keccak256::new();
    hasher.update(input);
    hasher.finalize_into(<&mut GenericArray<u8, _>>::from(&mut output[..]));

    vec![output[0], output[1], output[2], output[3]]
}
