#!/bin/bash

set -eu

MANIFEST_PATH=$1
EXAMPLE=$(dirname $MANIFEST_PATH | \
    sed "s/^integration-tests\///" | sed "s/^public\///" | sed "s/^internal\///" | \
    sed "s/\//_/g")

echo ""
echo "manifest path: " $MANIFEST_PATH
echo "size file: " $CONTRACT_SIZE_FILE
echo "example: " $EXAMPLE
echo ""

./scripts/build_and_determine_contract_size.sh $MANIFEST_PATH | \
    sed 's/^integration-tests\/\(public\/\|internal\/\)\?//' >> ${CONTRACT_SIZE_FILE}$EXAMPLE
