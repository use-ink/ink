// Copyright (C) Use Ink (UK) Ltd.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use sp_keyring::Sr25519Keyring;
use std::{
    ffi::{
        OsStr,
        OsString,
    },
    io::{
        BufRead,
        BufReader,
        Read,
    },
    process,
};
use subxt::{
    Config,
    OnlineClient,
    backend::rpc::RpcClient,
};

/// Spawn a local substrate node for testing.
pub struct TestNodeProcess<R: Config> {
    proc: process::Child,
    rpc: RpcClient,
    client: OnlineClient<R>,
    url: String,
}

impl<R> Drop for TestNodeProcess<R>
where
    R: Config,
{
    fn drop(&mut self) {
        let _ = self.kill();
    }
}

impl<R> TestNodeProcess<R>
where
    R: Config,
{
    /// Construct a builder for spawning a test node process.
    pub fn build<S>(program: S) -> TestNodeProcessBuilder<R>
    where
        S: AsRef<OsStr> + Clone,
    {
        TestNodeProcessBuilder::new(program)
    }

    /// Construct a builder for spawning a test node process, using the environment
    /// variable `CONTRACTS_NODE`, otherwise using the default contracts node.
    pub fn build_with_env_or_default() -> TestNodeProcessBuilder<R> {
        const DEFAULT_CONTRACTS_NODE: &str = "ink-node";

        // Use the user supplied `CONTRACTS_NODE` or default to `DEFAULT_CONTRACTS_NODE`.
        let contracts_node =
            std::env::var("CONTRACTS_NODE").unwrap_or(DEFAULT_CONTRACTS_NODE.to_owned());

        // Check the specified contracts node.
        if which::which(&contracts_node).is_err() {
            if contracts_node == DEFAULT_CONTRACTS_NODE {
                panic!(
                    "The '{DEFAULT_CONTRACTS_NODE}' executable was not found. Install '{DEFAULT_CONTRACTS_NODE}' on the PATH, \
                    or specify the `CONTRACTS_NODE` environment variable.",
                )
            } else {
                panic!("The contracts node executable '{contracts_node}' was not found.")
            }
        }
        Self::build(contracts_node)
    }

    /// Attempt to kill the running substrate process.
    pub fn kill(&mut self) -> Result<(), String> {
        tracing::info!("Killing node process {}", self.proc.id());
        if let Err(err) = self.proc.kill() {
            let err = format!("Error killing node process {}: {}", self.proc.id(), err);
            tracing::error!("{}", err);
            return Err(err)
        }
        Ok(())
    }

    /// Returns the `subxt` RPC client connected to the running node.
    pub fn rpc(&self) -> RpcClient {
        self.rpc.clone()
    }

    /// Returns the `subxt` client connected to the running node.
    pub fn client(&self) -> OnlineClient<R> {
        self.client.clone()
    }

    /// Returns the URL of the running node.
    pub fn url(&self) -> &str {
        &self.url
    }
}

/// Construct a test node process.
pub struct TestNodeProcessBuilder<R> {
    node_path: OsString,
    authority: Option<Sr25519Keyring>,
    marker: std::marker::PhantomData<R>,
}

impl<R> TestNodeProcessBuilder<R>
where
    R: Config,
{
    pub fn new<P>(node_path: P) -> TestNodeProcessBuilder<R>
    where
        P: AsRef<OsStr>,
    {
        Self {
            node_path: node_path.as_ref().into(),
            authority: None,
            marker: Default::default(),
        }
    }

    /// Set the authority development account for a node in validator mode e.g. --alice.
    pub fn with_authority(&mut self, account: Sr25519Keyring) -> &mut Self {
        self.authority = Some(account);
        self
    }

    /// Spawn the substrate node at the given path, and wait for RPC to be initialized.
    pub async fn spawn(&self) -> Result<TestNodeProcess<R>, String> {
        let mut cmd = process::Command::new(&self.node_path);
        cmd.env("RUST_LOG", "info")
            .arg("--dev")
            .stdout(process::Stdio::piped())
            .stderr(process::Stdio::piped())
            .arg("--port=0")
            .arg("--rpc-port=0")
            .arg("-lruntime::revive=debug");

        if let Some(authority) = self.authority {
            let authority = format!("{authority:?}");
            let arg = format!("--{}", authority.as_str().to_lowercase());
            cmd.arg(arg);
        }

        let mut proc = cmd.spawn().map_err(|e| {
            format!(
                "Error spawning substrate node '{}': {}",
                self.node_path.to_string_lossy(),
                e
            )
        })?;

        // Wait for RPC port to be logged (it's logged to stderr):
        let stderr = proc.stderr.take().unwrap();
        let port = find_substrate_port_from_output(stderr);
        let url = format!("ws://127.0.0.1:{port}");

        // Connect to the node with a `subxt` client:
        let rpc = RpcClient::from_url(url.clone())
            .await
            .map_err(|err| format!("Error initializing rpc client: {err}"))?;
        let client = OnlineClient::from_url(url.clone()).await;
        match client {
            Ok(client) => {
                Ok(TestNodeProcess {
                    proc,
                    rpc,
                    client,
                    url: url.clone(),
                })
            }
            Err(err) => {
                let err = format!("Failed to connect to node rpc at {url}: {err}");
                tracing::error!("{}", err);
                proc.kill().map_err(|e| {
                    format!("Error killing substrate process '{}': {}", proc.id(), e)
                })?;
                Err(err)
            }
        }
    }
}

// Consume a stderr reader from a spawned substrate command and
// locate the port number that is logged out to it.
fn find_substrate_port_from_output(r: impl Read + Send + 'static) -> u16 {
    let mut all_lines = String::new();
    BufReader::new(r)
        .lines()
        .find_map(|line| {
            let line =
                line.expect("failed to obtain next line from stdout for port discovery");
            all_lines.push_str(&format!("{line}\n"));

            // does the line contain our port (we expect this specific output from
            // substrate).
            let line_end = line
                .rsplit_once("Listening for new connections on 127.0.0.1:")
                .or_else(|| {
                    line.rsplit_once("Running JSON-RPC WS server: addr=127.0.0.1:")
                })
                .or_else(|| line.rsplit_once("Running JSON-RPC server: addr=127.0.0.1:"))
                .map(|(_, port_str)| port_str)?;

            // match the first group of digits
            let re = regex::Regex::new(r"^\d+").expect("regex creation failed");
            let port_capture = re
                .captures(line_end)
                .unwrap_or_else(|| panic!("unable to extract port from '{line_end}'"));
            assert!(
                port_capture.len() == 1,
                "captured more than one port from '{line_end}'"
            );
            let port_str = &port_capture[0];

            // expect to have a number here (the chars after '127.0.0.1:') and parse them
            // into a u16.
            let port_num = port_str.parse().unwrap_or_else(|_| {
                panic!("valid port expected for tracing line, got '{port_str}'")
            });

            Some(port_num)
        })
        .unwrap_or_else(|| {
            panic!(
                "Unable to extract port from spawned node, the reader ended.\n\
            These are the lines we saw up until here:\n{all_lines}"
            );
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use subxt::{
        PolkadotConfig as SubxtConfig,
        backend::legacy::LegacyRpcMethods,
    };

    #[tokio::test]
    #[allow(unused_assignments)]
    async fn spawning_and_killing_nodes_works() {
        let mut client1: Option<LegacyRpcMethods<SubxtConfig>> = None;
        let mut client2: Option<LegacyRpcMethods<SubxtConfig>> = None;

        {
            let node_proc1 = TestNodeProcess::<SubxtConfig>::build("ink-node")
                .spawn()
                .await
                .unwrap();
            client1 = Some(LegacyRpcMethods::new(node_proc1.rpc()));

            let node_proc2 = TestNodeProcess::<SubxtConfig>::build("ink-node")
                .spawn()
                .await
                .unwrap();
            client2 = Some(LegacyRpcMethods::new(node_proc2.rpc()));

            let res1 = client1.clone().unwrap().chain_get_block_hash(None).await;
            let res2 = client2.clone().unwrap().chain_get_block_hash(None).await;

            assert!(res1.is_ok(), "process 1 is not ok, but should be");
            assert!(res2.is_ok(), "process 2 is not ok, but should be");
        }

        // node processes should have been killed by `Drop` in the above block.
        let res1 = client1.unwrap().chain_get_block_hash(None).await;
        let res2 = client2.unwrap().chain_get_block_hash(None).await;

        assert!(
            res1.is_err(),
            "process 1: did not find err, but expected one"
        );
        assert!(
            res2.is_err(),
            "process 2: did not find err, but expected one"
        );
    }

    #[test]
    fn parse_port_from_node_output() {
        let log = "2024-12-04 10:57:03.893  INFO main sc_rpc_server: Running JSON-RPC server: addr=127.0.0.1:9944,[::1]:9944  ";
        let port = find_substrate_port_from_output(log.as_bytes());
        assert_eq!(port, 9944);

        let log = "2024-12-04 10:57:03.893  INFO main sc_rpc_server: Running JSON-RPC server: addr=127.0.0.1:9944  ";
        let port = find_substrate_port_from_output(log.as_bytes());
        assert_eq!(port, 9944);

        let log = r#"2024-12-04 11:02:24.637  INFO main sc_cli::runner: 👤 Role: AUTHORITY
2024-12-04 11:02:24.637  INFO main sc_cli::runner: 💾 Database: RocksDb at /var/folders/s5/5gcp8ck95k39z006fj059_0c0000gn/T/substrateHZoRbb/chains/dev/db/full
2024-12-04 11:02:25.324  WARN main sc_service::config: Using default protocol ID "sup" because none is configured in the chain specs
2024-12-04 11:02:25.327  INFO main sc_rpc_server: Running JSON-RPC server: addr=127.0.0.1:9944,[::1]:9944
2024-12-04 11:02:24.637  INFO main sc_cli::runner: 💾 Database: RocksDb at /var/folders/s5/5gcp8ck95k39z006fj059_0c0000gn/T/substrateHZoRbb/chains/dev/db/full
2024-12-04 11:02:24.637  INFO main sc_cli::runner: 💾 Database: RocksDb at /var/folders/s5/5gcp8ck95k39z006fj059_0c0000gn/T/substrateHZoRbb/chains/dev/db/full
"#;
        let port = find_substrate_port_from_output(log.as_bytes());
        assert_eq!(port, 9944);
    }
}
