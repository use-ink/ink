#!/bin/bash

set -eu

# This file creates a CSV file containing the result
# of building a contract with different ABIs.
# Additionally, a comparison to the contract's size in
# ink! v5 is included.

master=$1
default_abi=$2
sol_abi=$3
all_abi=$4
v5=$5
contract=$6

master=$(egrep "^$contract\s+" $master | head -n1 | cut -d " " -f2)
def=$(egrep "^$contract\s+" $default_abi | head -n1 | cut -d " " -f2)
sol=$(egrep "^$contract\s+" $sol_abi | head -n1 | cut -d " " -f2)
all=$(egrep "^$contract\s+" $all_abi | head -n1 | cut -d " " -f2)
v5=$(egrep "^$contract\s+" $v5 | head -n1 | cut -d ";" -f2)

if [ -z "$master" ]; then
    master="0";
fi

if [ -z "$def" ]; then
    def="0";
fi

if [ -z "$sol" ]; then
    sol="0";
fi

if [ -z "$all" ]; then
    all="0";
fi

if [ -z "$v5" ]; then
    v5="0";
fi

echo "$contract;$master;$def;$sol;$all;$v5"
