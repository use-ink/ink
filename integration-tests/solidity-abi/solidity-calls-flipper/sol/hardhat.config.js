require("@nomicfoundation/hardhat-ethers");
require("@parity/hardhat-polkadot");

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
        },
    },
    resolc: {
        compilerSource: 'npm',
    },
    paths: {
        cache: "./cache-pvm",
        artifacts: "./artifacts-pvm"
    },
};

module.exports = config;
