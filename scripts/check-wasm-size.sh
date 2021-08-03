#!/usr/bin/env bash

# Write the non-optimized and optimized sizes of the given
# example contracts into a CSV file.
#
# Format is: `contract_name, unoptimized_size, optimized_size`.
#
# Usage: `./check-wasm-size.sh <file_name>`

set -eux

declare -a size_output

OUTPUT_FILE=$1
if test -f "$OUTPUT_FILE.csv"; then
    echo "$OUTPUT_FILE.csv exists, removing."
    rm $OUTPUT_FILE.csv
fi


EXAMPLES="erc20 trait-erc20 erc1155" # erc721 dns multisig_plain"

CC_OUT=""
SZ_OUT=""
for crate in ${EXAMPLES}; do
   # cargo +nightly clean --manifest-path examples/${crate}/Cargo.toml
   CC_OUT=$(cargo +nightly contract build --manifest-path examples/${crate}/Cargo.toml \
   --generate code-only)
   SIZE_OUT=$(echo "${CC_OUT}" | rg "Original" | cut -d " " -f 4,6)
   size_output+=( "${crate}, ${SIZE_OUT}" )
done

for out in "${size_output[@]}"; do
    echo "$out" >> $OUTPUT_FILE.csv
done

# awk '{print $1","$2","$3"}' > wasm-size.csv
