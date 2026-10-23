#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../.."
export DOCKER_CONFIG="$PWD/.kineti/docker"
mkdir -p "$DOCKER_CONFIG"
cargo build --locked -p kineti-cli
mkdir -p .kineti/implementation/evaluator-context/.kineti/implementation .kineti/implementation/evaluator-context/scripts/evaluator
cp target/debug/kineti-cli .kineti/implementation/evaluator-context/.kineti/implementation/kineti-cli
cp scripts/evaluator/{Dockerfile,candidate,entrypoint} .kineti/implementation/evaluator-context/scripts/evaluator/
proxy_args=()
if getent ahostsv4 proxy >/dev/null; then proxy_args=(--add-host "proxy:$(getent ahostsv4 proxy | awk 'NR==1 {print $1}')"); fi
docker build "${proxy_args[@]}" --network=host --build-arg HTTP_PROXY --build-arg HTTPS_PROXY --build-arg NO_PROXY -t kineti-evaluator:acceptance -f .kineti/implementation/evaluator-context/scripts/evaluator/Dockerfile .kineti/implementation/evaluator-context
