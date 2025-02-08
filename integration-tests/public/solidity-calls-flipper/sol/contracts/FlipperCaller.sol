// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

interface IFlipper {
    function flip() external;
}

contract FlipperCaller {
    // Address of the contract we want to call
    address private flipperContract;

    // Constructor to set the flipper contract address
    constructor(address _flipperContract) {
        flipperContract = _flipperContract;
    }

    // Function to call flip() on the target contract when
    // ink! sets the selector equivalent to the solidity
    // selector of the function flip()
    function callFlip() external {
        IFlipper(flipperContract).flip();
    }

    // Manually generate the selector for a message `flip_2`
    function callFlip2() external {
        bytes4 selector = bytes4(keccak256("flip_2"));
        (bool ok,) = flipperContract.call(abi.encodePacked(selector));
        require(ok, "call failed");
    }

    // TODO: currently fails as ink! returns data with SCALE
    function callGet() external returns (bool) {
        bytes4 selector = bytes4(keccak256("get"));
        (bool ok, bytes memory data) = flipperContract.call(abi.encodePacked(selector));
        require(ok, "call failed");
        return abi.decode(data, (bool));
    }

    // Function to update flipper contract address if needed
    function setFlipperContract(address _newFlipper) external {
        flipperContract = _newFlipper;
    }
}