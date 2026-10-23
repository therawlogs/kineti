#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../.."
export DOCKER_CONFIG="${DOCKER_CONFIG:-$PWD/.kineti/docker}"
context="$PWD/.kineti/harness-build"
mkdir -p "$context" "$DOCKER_CONFIG"
cp "$(command -v bun)" "$context/bun"
cp scripts/harness/{Dockerfile,check-spend.ts} "$context/"
proxy_args=()
if getent ahostsv4 proxy >/dev/null; then proxy_args=(--add-host "proxy:$(getent ahostsv4 proxy | awk 'NR==1 {print $1}')"); fi
docker build "${proxy_args[@]}" --network=host --build-arg HTTP_PROXY --build-arg HTTPS_PROXY --build-arg NO_PROXY -t kineti-harness-worker:acceptance "$context"
