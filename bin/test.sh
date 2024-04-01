#!/usr/bin/env bash

set -o errexit

export IPFS_RPC_API_URL=$(bin/ipfs.sh api-url)
make ipfs

cargo test
