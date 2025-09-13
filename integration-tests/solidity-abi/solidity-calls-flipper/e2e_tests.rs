use crate::flipper::FlipperRef;
use ink::{
    Address,
    SolDecode,
    SolEncode,
    env::{
        Balance,
        DefaultEnvironment,
    },
    primitives::DepositLimit,
};
use ink_e2e::{
    BuilderClient,
    ChainBackend,
    PolkadotConfig,
    Weight,
    subxt_signer,
};
use std::{
    error::Error,
    process::{
        Command,
        Stdio,
    },
};

const DEFAULT_GAS: Weight = Weight::from_parts(100_000_000_000, 1024 * 1024);
type E2EResult<T> = Result<T, Box<dyn Error>>;

// TODO: (@davidsemakula) Re-enable when "no space left" CI issue is fixed
// See https://github.com/use-ink/ink/issues/2458 for details.
// This test consistently triggers the issue in CI when running `npm install` for hardhat
// scripts.
#[ignore]
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
    let alith = subxt_signer::eth::dev::alith();
    let acc_id = alith.public_key().to_account_id();
    let mut acc_bytes = [0xEE; 32];
    acc_bytes[..20].copy_from_slice(acc_id.as_ref());

    client
        .transfer_allow_death(
            &ink_e2e::alice(),
            <ink::env::DefaultEnvironment as ink::env::Environment>::AccountId::from(
                acc_bytes,
            ),
            1_000_000_000_000_000,
        )
        .await?;

    let signer = ink_e2e::alice();

    // deploy ink! flipper (Sol encoded)
    let _ = client.map_account(&signer).await;
    let input = exec_input.encode();
    let storage_deposit_limit: Balance = 10_00_000_000_000_000_000;
    let ink_addr = client
        .exec_instantiate(
            &signer,
            "flipper",
            input,
            0,
            DEFAULT_GAS,
            storage_deposit_limit,
        )
        .await?
        .addr;

    let get_selector = keccak_selector(b"get()");
    let value: bool = call_ink(&mut client, ink_addr, get_selector.clone()).await;
    assert!(!value);

    let mut sol_handler = SolidityHandler::new("./sol".into(), client.url().to_string());
    sol_handler.start_eth_rpc()?;
    sol_handler.setup()?;
    let sol_addr = sol_handler.deploy(ink_addr)?;

    let _ = sol_handler.call(&sol_addr, "callFlip")?;
    let value: bool = call_ink(&mut client, ink_addr, get_selector.clone()).await;
    assert!(value);

    let output = sol_handler.call(&sol_addr, "callGet")?;
    assert_eq!(output, Some("true".to_string()));

    let _ = sol_handler.call(&sol_addr, "callFlip2")?;
    let value: bool = call_ink(&mut client, ink_addr, get_selector.clone()).await;
    assert!(!value);

    let _ = sol_handler.call_with_value(&sol_addr, "callSet", true)?;

    let output = sol_handler.call(&sol_addr, "callGet")?;
    assert_eq!(output, Some("true".to_string()));

    let output = sol_handler.call(&sol_addr, "callGet2")?;
    assert_eq!(output, Some("true".to_string()));

    let _ = sol_handler.call_with_value(&sol_addr, "callSet", false)?;

    let output = sol_handler.call(&sol_addr, "callGet")?;
    assert_eq!(output, Some("false".to_string()));

    // test ink! can call solidity
    let sol_addr_h160 = <Address as std::str::FromStr>::from_str(&sol_addr)
        .expect("Expected 20 bytes hex string");
    let sol_addr_encoded = SolEncode::encode(&sol_addr_h160);

    let encoded = encode_ink_call("call_solidity_set(address)", sol_addr_encoded.clone());
    let encoded_get = encode_ink_call("call_solidity_get(address)", sol_addr_encoded);
    let ret: u16 = call_ink(&mut client, ink_addr, encoded_get.clone()).await;
    assert_eq!(ret, 42);
    call_ink_no_return(&mut client, ink_addr, encoded).await;
    // set_value uses hardcoded 77 for simplicity.
    let ret: u16 = call_ink(&mut client, ink_addr, encoded_get.clone()).await;
    assert_eq!(ret, 77);

    Ok(())
}

async fn call_ink<Ret>(
    client: &mut ink_e2e::Client<PolkadotConfig, DefaultEnvironment>,
    ink_addr: Address,
    data_sol: Vec<u8>,
) -> Ret
where
    Ret: SolDecode,
{
    let signer = ink_e2e::alice();
    let exec_result = client
        .raw_call_dry_run::<Vec<u8>, ink::abi::Sol>(
            ink_addr,
            data_sol,
            0u32.into(),
            ink::primitives::DepositLimit::UnsafeOnlyForDryRun,
            &signer,
        )
        .await
        .expect("dry run failed");
    <Ret>::decode(&exec_result.exec_result.result.unwrap().data[..])
        .expect("decode failed")
}

async fn call_ink_no_return(
    client: &mut ink_e2e::Client<PolkadotConfig, DefaultEnvironment>,
    ink_addr: Address,
    data_sol: Vec<u8>,
) {
    let signer = ink_e2e::alice();
    let _ = client
        .raw_call(
            ink_addr,
            data_sol,
            Balance::from(0u128),
            DEFAULT_GAS.into(),
            DepositLimit::Balance(Balance::MAX),
            &signer,
        )
        .await;
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
            .arg(self.eth_rpc_process_id.unwrap().to_string())
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
        env_vars: &[(&str, &str)],
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
            "{script} execution failed with env: {env_vars:?}"
        );

        Ok(output.stdout)
    }

    fn deploy(&self, ink_addr: Address) -> Result<String, Box<dyn Error>> {
        let output = self.run_hardhat_script(
            "01-deploy.js",
            &[("INK_ADDRESS", &format!("{ink_addr:?}"))],
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
        sol_addr: &str,
        message: &str,
    ) -> Result<Option<String>, Box<dyn Error>> {
        let output = self.run_hardhat_script(
            "02-call-flip.js",
            &[("SOL_ADDRESS", sol_addr), ("MESSAGE", message)],
            Stdio::piped(),
        )?;
        Ok(String::from_utf8(output)
            .ok()
            .and_then(|s| Some(s.lines().last()?.to_string()))
            .map(|s| s.trim().to_string()))
    }

    fn call_with_value(
        &self,
        sol_addr: &str,
        message: &str,
        value: bool,
    ) -> Result<Option<String>, Box<dyn Error>> {
        let output = self.run_hardhat_script(
            "02-call-flip.js",
            &[
                ("SOL_ADDRESS", sol_addr),
                ("MESSAGE", message),
                ("VALUE", &value.to_string()),
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

fn keccak_selector(input: &[u8]) -> Vec<u8> {
    let mut output = [0; 32];
    use sha3::{
        Digest as _,
        digest::generic_array::GenericArray,
    };
    let mut hasher = sha3::Keccak256::new();
    hasher.update(input);
    hasher.finalize_into(<&mut GenericArray<u8, _>>::from(&mut output[..]));

    vec![output[0], output[1], output[2], output[3]]
}

fn encode_ink_call(selector: &str, args: Vec<u8>) -> Vec<u8> {
    let mut encoded = Vec::new();
    encoded.extend(keccak_selector(selector.as_bytes()));
    encoded.extend(args);
    encoded
}
