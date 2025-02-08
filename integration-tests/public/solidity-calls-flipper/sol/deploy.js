const {ethers} = require("hardhat");

async function main() {
    try {
        console.log("Starting deployment process...");

        const Contract = await ethers.getContractFactory("FlipperCaller");

        // Get the deployer's address
        const [deployer] = await ethers.getSigners();
        console.log(`Deploying with account: ${deployer.address}`);

        const signers = await ethers.getSigners();

        for (const signer of signers.slice(1)) {
            const tx = {
                to: signer,
                value: ethers.parseEther("100.0"), // Amount to send (1.0 ETH in this example)
            };

            const transactionResponse = await signers[0].sendTransaction(tx);

            await transactionResponse.wait();
        }

        // Deploy the contract
        const contract = await Contract.deploy("0xad940b0a697c626c8b5d6e707124403bbcfa3bad");
        await contract.waitForDeployment();

        // Get the deployed contract address
        const contractAddress = await contract.getAddress();
        console.log(`Contract deployed to: ${contractAddress}`);

        // Verify the contract if not on a local network
        const network = await ethers.provider.getNetwork();
        const chainId = network.chainId;

        // Log deployment data
        const deploymentData = {
            contract: contractAddress,
            deployer: deployer.address,
            network: network.name,
            chainId: chainId,
            blockNumber: await ethers.provider.getBlockNumber(),
            timestamp: new Date().toISOString()
        };

        console.log("\nDeployment Summary:");
        console.log(JSON.stringify(deploymentData, (key, value) =>
            typeof value === "bigint" ? value.toString() : value, 2
        ));

        let flipper = contract.attach(contractAddress);
        // console.log("Initial value", await flipper.callGet());

        let callFlipTx = await flipper.callFlip();
        let res = await callFlipTx.wait();
        console.log(res);
        // console.log("After callFlip value", await flipper.callGet());
        //
        // let tx = await flipper.callFlip2();
        // res = await tx.wait();
        // console.log(res);
        // console.log("After callFlip2 value", await flipper.callGet());
    } catch (error) {
        console.error("Deployment failed:", error);
        process.exit(1);
    }
}

main()
    .then(() => process.exit(0))
    .catch((error) => {
        console.error(error);
        process.exit(1);
    });