const {ethers} = require("hardhat");

async function main() {
    try {
        const inkAddress = process.env.INK_ADDRESS;
        if (!inkAddress) {
            console.error("requires INK_ADDRESS being set.");
            process.exit(1);
        }

        const Contract = await ethers.getContractFactory("FlipperCaller");
        const contract = await Contract.deploy(inkAddress);
        await contract.waitForDeployment();

        const contractAddress = await contract.getAddress();
        // output for e2e test to read
        console.log(contractAddress);
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