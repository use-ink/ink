#!/bin/bash

set -ex

F="${1:-pass}"
CROSS_CONTRACT_ADDR=$2
FLIPPER_ADDR=$3

SELECTOR="0x633aa551"
BAD_SELECTOR="0x00000000"

args=()
args+=( --contract ${CROSS_CONTRACT_ADDR} )
args+=( --message call )
args+=( --suri //Alice )
args+=( --manifest-path ./examples/lang_err_integration_tests/Cargo.toml )
args+=( --verbose )
args+=( --skip-confirm )

case $F in
    '--fail')
        args+=( --args $FLIPPER_ADDR $BAD_SELECTOR ) ;;
    *)
        args+=( --args $FLIPPER_ADDR $SELECTOR ) ;;
 esac

cargo contract call "${args[@]}"
