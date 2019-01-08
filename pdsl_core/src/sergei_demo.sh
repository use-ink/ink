#!/bin/sh

curl -XPOST -N 'Content-Type: application/json' -d '{"jsonrpc": "2.0", "method": "chain_getBlockHash", "params": [0], "id": 1}' -vv http://localhost:9933 | jq -r '.result'

# On Success: Returns the generis block hash in the last line.
# => GENESIS_HASH

curl -XPOST -N 'Content-Type: application/json' -d '{"jsonrpc": "2.0", "method": "author_submitEntrinsic", "params": ["$GENESIS_HASH"], "id": 1}' -vv http://localhost:9933 | jq -r '.result'

# 