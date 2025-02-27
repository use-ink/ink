const {ethers} = require("hardhat");

async function main() {
    try {
        const solAddress = process.env.SOL_ADDRESS;
        if (!solAddress) {
            console.error("requires SOL_ADDRESS being set.");
            process.exit(1);
        }

        const message = process.env.MESSAGE;
        if (!message) {
            console.error("Requires MESSAGE being set. Options: 'callFlip', 'callFlip2' or `callGet`");
            process.exit(1);
        }

        const value = process.env.VALUE === "true";

        const Contract = await ethers.getContractFactory("FlipperCaller");
        let flipper = Contract.attach(solAddress);

        let error = undefined;
        switch (message) {
            case "callFlip":
                let callFlipTx = await flipper.callFlip();
                let res = await callFlipTx.wait();
                error = res.error;
                break;
            case "callFlip2":
                let callFlip2Tx = await flipper.callFlip2();
                let res2 = await callFlip2Tx.wait();
                error = res2.error;
                break;
            case "callSet":
                let callSet = await flipper.callSet(value);
                let resSet = await callSet.wait();
                error = resSet.error;
                break;
            case "callGet":
                const resGet = await flipper.callGet();
                error = resGet.error;
                if (error) {
                    break;
                }

                const receipt = await resGet.wait();
                const logs = await receipt.logs;
                const currentValue = logs.find(event => event.fragment && event.fragment.name === "ReturnValue")?.args;
                console.log(currentValue.toString());

                break;
            case "callGet2":
                const resGet2 = await flipper.callGet2();
                error = resGet2.error;
                if (error) {
                    break;
                }

                const receipt2 = await resGet2.wait();
                const logs2 = await receipt2.logs;
                const currentValue2 = logs2.find(event => event.fragment && event.fragment.name === "ReturnValue")?.args;
                console.log(currentValue2.toString());

                break;
            default:
                console.error("Invalid message option. Options: 'callFlip', 'callFlip2' or `callGet`");
                process.exit(1);
        }
        if (error) {
            console.error("Calling failed:", error);
            process.exit(1);
        }
    } catch (error) {
        console.error("Calling failed:", error);
        process.exit(1);
    }
}

main()
    .then(() => process.exit(0))
    .catch((error) => {
        console.error(error);
        process.exit(1);
    });