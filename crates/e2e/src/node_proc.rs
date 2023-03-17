// Copyright 2018-2022 Parity Technologies (UK) Ltd.
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

use sp_keyring::AccountKeyring;
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
};

/// Spawn a local substrate node for testing.
pub struct TestNodeProcess<R: Config> {
    proc: process::Child,
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

    /// Attempt to kill the running substrate process.
    pub fn kill(&mut self) -> Result<(), String> {
        log::info!("Killing node process {}", self.proc.id());
        if let Err(err) = self.proc.kill() {
            let err = format!("Error killing node process {}: {}", self.proc.id(), err);
            log::error!("{}", err);
            return Err(err)
        }
        Ok(())
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
    authority: Option<AccountKeyring>,
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
    pub fn with_authority(&mut self, account: AccountKeyring) -> &mut Self {
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
            .arg("--ws-port=0");

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
        let ws_port = find_substrate_port_from_output(stderr);
        let ws_url = format!("ws://127.0.0.1:{ws_port}");

        // Connect to the node with a `subxt` client:
        let client = OnlineClient::from_url(ws_url.clone()).await;
        match client {
            Ok(client) => {
                Ok(TestNodeProcess {
                    proc,
                    client,
                    url: ws_url.clone(),
                })
            }
            Err(err) => {
                let err = format!("Failed to connect to node rpc at {ws_url}: {err}");
                log::error!("{}", err);
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
    BufReader::new(r)
        .lines()
        .find_map(|line| {
            let line =
                line.expect("failed to obtain next line from stdout for port discovery");

            // does the line contain our port (we expect this specific output from substrate).
            let line_end = line
                .rsplit_once("Listening for new connections on 127.0.0.1:")
                .or_else(|| {
                    line.rsplit_once("Running JSON-RPC WS server: addr=127.0.0.1:")
                })
                .map(|(_, port_str)| port_str)?;

            // trim non-numeric chars from the end of the port part of the line.
            let port_str = line_end.trim_end_matches(|b: char| !b.is_ascii_digit());

            // expect to have a number here (the chars after '127.0.0.1:') and parse them into a u16.
            let port_num = port_str.parse().unwrap_or_else(|_| {
                panic!("valid port expected for log line, got '{port_str}'")
            });

            Some(port_num)
        })
        .expect("We should find a port before the reader ends")
}

#[cfg(test)]
mod tests {
    use super::*;
    use subxt::PolkadotConfig as SubxtConfig;

    #[tokio::test]
    #[allow(unused_assignments)]
    async fn spawning_and_killing_nodes_works() {
        let mut client1: Option<OnlineClient<SubxtConfig>> = None;
        let mut client2: Option<OnlineClient<SubxtConfig>> = None;

        {
            let node_proc1 =
                TestNodeProcess::<SubxtConfig>::build("substrate-contracts-node")
                    .spawn()
                    .await
                    .unwrap();
            client1 = Some(node_proc1.client());

            let node_proc2 =
                TestNodeProcess::<SubxtConfig>::build("substrate-contracts-node")
                    .spawn()
                    .await
                    .unwrap();
            client2 = Some(node_proc2.client());

            let res1 = node_proc1.client().rpc().block_hash(None).await;
            let res2 = node_proc1.client().rpc().block_hash(None).await;

            assert!(res1.is_ok());
            assert!(res2.is_ok());
        }

        // node processes should have been killed by `Drop` in the above block.
        let res1 = client1.unwrap().rpc().block_hash(None).await;
        let res2 = client2.unwrap().rpc().block_hash(None).await;

        assert!(res1.is_err());
        assert!(res2.is_err());
    }
}
