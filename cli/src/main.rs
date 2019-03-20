// Copyright 2018-2019 Parity Technologies (UK) Ltd.
// This file is part of pDSL.
//
// pDSL is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// pDSL is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with pDSL.  If not, see <http://www.gnu.org/licenses/>.

mod cmd;

use structopt::{
    clap::AppSettings,
    StructOpt,
};

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

use std::result::Result as StdResult;

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
        /// Deploy on a local development chain.
        #[structopt(name = "dev", short, long)]
        on_dev: bool,
    },
}

fn main() -> cmd::Result<()> {
    let Opts::Contract(args) = Opts::from_args();
    use crate::cmd::{
        CommandError,
        CommandErrorKind,
    };
    match &args.cmd {
        Command::New { layer, name } => cmd::execute_new(layer, name),
        Command::Build {} => {
            Err(CommandError::new(CommandErrorKind::UnimplementedCommand))
        }
        Command::Test {} => {
            Err(CommandError::new(CommandErrorKind::UnimplementedCommand))
        }
        Command::Deploy { .. } => {
            Err(CommandError::new(CommandErrorKind::UnimplementedCommand))
        }
    }
}
