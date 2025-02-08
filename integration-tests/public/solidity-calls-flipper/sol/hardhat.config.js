require("hardhat-resolc")
require("hardhat-revive-node")
require("@nomicfoundation/hardhat-ethers");

const config = {
    solidity: "0.8.28",
    networks: {
        localhost: {
            accounts: [
                // alith private key
                '0x5fb92d6e98884f76de468fa3f6278f8807c48bebc13595d45af5bdc4da702133',
            ]
        },
        hardhat: {
            polkavm: true,
            nodeConfig: {
                rpcPort: 55615,
                nodeBinaryPath: "/Users/peter/dev/dot-repos/polkadot-sdk/target/release/substrate-node",
                dev: true,
            },
            adapterConfig: {
                adapterBinaryPath: '/Users/peter/dev/dot-repos/polkadot-sdk/target/release/eth-rpc',
                dev: true,
            },

        },
    },
    resolc: {
        compilerSource: 'binary',
        settings: {
            // optimizer: {
            //     enabled: true,
            // },
            evmVersion: "cancun",
            compilerPath: "/Users/peter/.cargo/bin/resolc-v.7",
            standardJson: true,
        },
    },
    paths: {
        cache: "./cache-pvm",
        artifacts: "./artifacts-pvm"
    },
};

module.exports = config;
