#!/usr/bin/env bash

set -o errexit
set -o nounset

IPFS_CONTAINER_NAME="smeeg-ipfs"
IPFS_STAGING_VOLUME_NAME="smeeg-ipfs-staging"
IPFS_DATA_VOLUME_NAME="smeeg-ipfs-data"

IPFS_PEERING_PORT=4001
IPFS_API_PORT=5001
IPFS_GATEWAY_PORT=8080

CONTAINER_RUNTIME="podman"
if which docker &>/dev/null; then
	CONTAINER_RUNTIME="docker"
fi

function api_url {
	echo "http://localhost:${IPFS_API_PORT}"
}

function gateway_url {
	echo "http://localhost:${IPFS_GATEWAY_PORT}"
}

function run {
	start-ipfs-container
}

# Helpers:

function start-ipfs-container {
	ensure-ipfs-container-exists
	${CONTAINER_RUNTIME} start ${IPFS_CONTAINER_NAME}
}

function ensure-ipfs-container-exists {
	docker pull ipfs/kubo:latest
	create-ipfs-container
}

function create-ipfs-container {
	if ${CONTAINER_RUNTIME} ps -a | grep ${IPFS_CONTAINER_NAME} &>/dev/null; then
		return
	fi

	${CONTAINER_RUNTIME} volume create ${IPFS_STAGING_VOLUME_NAME} || true
	${CONTAINER_RUNTIME} volume create ${IPFS_DATA_VOLUME_NAME} || true

	${CONTAINER_RUNTIME} run \
		--name ${IPFS_CONTAINER_NAME} \
		--volume ${IPFS_STAGING_VOLUME_NAME}:/export \
		--volume ${IPFS_DATA_VOLUME_NAME}:/data/ipfs \
		--publish 4001:4001 \
		--publish 4001:4001/udp \
		--publish 8080:8080 \
		--publish 5001:5001 \
		--detach \
		ipfs/kubo:latest
}

function clean() {
	docker stop ${ipfs_CONTAINER_NAME} || true
	${CONTAINER_RUNTIME} rm -fv ${IPFS_CONTAINER_NAME} || true
	${CONTAINER_RUNTIME} volume rm -f ${IPFS_STAGING_VOLUME_NAME} || true
	${CONTAINER_RUNTIME} volume rm -f ${IPFS_DATA_VOLUME_NAME} || true
}

$1
