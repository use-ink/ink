// Copyright 2018-2019 Parity Technologies (UK) Ltd.
// This file is part of ink!.
//
// ink! is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// ink! is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with ink!.  If not, see <http://www.gnu.org/licenses/>.

mod cmd;

use structopt::{
    clap::AppSettings,
    StructOpt,
};
use url::Url;

#[derive(Debug, StructOpt)]
#[structopt(bin_name = "cargo")]
pub(crate) enum Opts {
    #[structopt(
        name = "contract",
        raw(
            setting = "AppSettings::UnifiedHelpMessage",
            setting = "AppSettings::DeriveDisplayOrder",
            setting = "AppSettings::DontCollapseArgsInUsage"
        )
    )]
    /// Utilities to develop Wasm smart contracts.
    Contract(ContractArgs),
}

#[derive(Debug, StructOpt)]
pub(crate) struct ContractArgs {
    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(crate) enum AbstractionLayer {
    Core,
    Model,
    Lang,
}

use std::{
    path::PathBuf,
    result::Result as StdResult,
};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(crate) struct InvalidAbstractionLayer;

impl std::fmt::Display for InvalidAbstractionLayer {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "expected `core`, `model` or `lang`")
    }
}

impl std::str::FromStr for AbstractionLayer {
    type Err = InvalidAbstractionLayer;

    fn from_str(input: &str) -> StdResult<Self, Self::Err> {
        match input {
            "core" => Ok(AbstractionLayer::Core),
            "model" => Ok(AbstractionLayer::Model),
            "lang" => Ok(AbstractionLayer::Lang),
            _ => Err(InvalidAbstractionLayer),
        }
    }
}

#[derive(Debug, StructOpt)]
enum Command {
    /// Setup and create a new smart contract.
    #[structopt(name = "new")]
    New {
        /// The abstraction layer to use: `core`, `model` or `lang`
        #[structopt(short = "l", long = "layer", default_value = "lang")]
        layer: AbstractionLayer,
        /// The name of the newly created smart contract.
        name: String,
        /// The optional target directory for the contract project
        #[structopt(short, long, parse(from_os_str))]
        target_dir: Option<PathBuf>,
    },
    /// Builds the smart contract.
    #[structopt(name = "build")]
    Build {},
    /// Test the smart contract off-chain.
    #[structopt(name = "test")]
    Test {},
    /// Deploy the smart contract on-chain. (Also for testing purposes.)
    #[structopt(name = "deploy")]
    Deploy {
        /// Websockets url of a substrate node
        #[structopt(
            name = "url",
            long,
            parse(try_from_str),
            default_value = "ws://localhost:9944"
        )]
        url: Url,
        /// Secret key URI for the account deploying the contract.
        #[structopt(name = "suri", long, short)]
        suri: String,
        /// Password for the secret key
        #[structopt(name = "password", long, short)]
        password: Option<String>,
        #[structopt(name = "gas", long, default_value = "500000")]
        /// Maximum amount of gas to be used in this deployment
        gas: u64,
        /// Path to wasm contract code, defaults to ./target/<name>-pruned.wasm
        #[structopt(parse(from_os_str))]
        wasm_path: Option<std::path::PathBuf>,
    },
}

fn main() {
    env_logger::init();

    let Opts::Contract(args) = Opts::from_args();
    match exec(args.cmd) {
        Ok(msg) => println!("\t{}", msg),
        Err(err) => eprintln!("error: {}", err),
    }
}

fn exec(cmd: Command) -> cmd::Result<String> {
    use crate::cmd::CommandError;
    match &cmd {
        Command::New {
            layer,
            name,
            target_dir,
        } => cmd::execute_new(*layer, name, target_dir.as_ref()),
        Command::Build {} => cmd::execute_build(None),
        Command::Test {} => Err(CommandError::UnimplementedCommand),
        Command::Deploy {
            url,
            suri,
            password,
            gas,
            wasm_path,
        } => {
            cmd::execute_deploy(
                url.clone(),
                suri,
                password.as_ref().map(String::as_ref),
                *gas,
                wasm_path.as_ref(),
            )
        }
    }
}
