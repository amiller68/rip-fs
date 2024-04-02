#!/usr/bin/env bash

set -o errexit

export OLLAMA_SERVER_URL=$(bin/ollama.sh server-url)
export OLLAMA_SUPERVISOR_MODEL=$(bin/ollama.sh supervisor-model)
export OLLAMA_CONVERSATIONAL_MODEL=$(bin/ollama.sh conversational-model)
export OLLAMA_IMAGE_MODEL=$(bin/ollama.sh image-model)
export CHROMA_DATABASE_URL=$(bin/chroma.sh database-url)
export SQLITE_DATABASE_URL=$(bin/sqlite.sh database-url)

cargo run
