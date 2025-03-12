// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

interface IFlipper {
    function flip() external;

    function get() external view returns (bool);
}

contract FlipperCaller {
    // Address of the contract we want to call
    address private flipperContract;
    uint16 public value;

    event ReturnValue(bool value);

    // Constructor to set the flipper contract address
    constructor(address _flipperContract) {
        flipperContract = _flipperContract;
        // arbitrary value
        value = 42;
    }

    // Function to call flip() on the target contract when
    // ink! sets the selector equivalent to the solidity
    // selector of the function flip()
    function callFlip() external {
        IFlipper(flipperContract).flip();
    }

    // Manually generate the selector for a message `flip_2`
    function callFlip2() external {
        bytes4 selector = bytes4(keccak256("flip_2()"));
        (bool ok,) = flipperContract.call(abi.encodePacked(selector));
        require(ok, "call failed");
    }

    function callSet(bool _value) external {
        bytes4 selector = bytes4(keccak256("set(bool)"));
        (bool ok,) = flipperContract.call(abi.encodeWithSelector(selector, _value));
        require(ok, "call failed");
    }

    function callGet() external {
        IFlipper(flipperContract).get();
        bytes4 selector = bytes4(keccak256("get()"));
        (bool ok, bytes memory data) = flipperContract.call(abi.encodePacked(selector));
        require(ok, "call failed");
        bool _value = abi.decode(data, (bool));
        emit ReturnValue(_value);
    }

    function callGet2() external {
        bool _value = IFlipper(flipperContract).get();
        emit ReturnValue(_value);
    }

    // get_value and set_value to be called by ink!

    function get_value() external view returns (uint16) {
        return value;
    }

    function set_value(uint16 _value) external {
        value = _value;
    }
}