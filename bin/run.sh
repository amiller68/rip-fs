#!/usr/bin/env bash

set -o errexit

export IPFS_RPC_API_URL=$(bin/ipfs.sh api-url)
export SQLITE_DATABASE_URL=$(bin/sqlite.sh database-url)
make ipfs
make sqlite

cargo run
