#!/usr/bin/env bash
set -euo pipefail

# Capture only local availability. This does not run either product or certify isolation.
repo_root="$(cd "$(dirname "$0")/.." && pwd)"
out="${1:-$repo_root/docs/containment-evaluation.local.json}"
mkdir -p "$(dirname "$out")"

has() { command -v "$1" >/dev/null 2>&1 && printf true || printf false; }
box_bin="$(command -v box || true)"
openshell_bin="$(command -v openshell || true)"
container_bin=""
for candidate in docker podman; do
  if command -v "$candidate" >/dev/null 2>&1; then container_bin="$candidate"; break; fi
done

export BOX_AVAILABLE="$(has box)"
export STRANDS_BOX_AVAILABLE="$(has strands-box)"
export OPENSHELL_AVAILABLE="$(has openshell)"
export CONTAINER_RUNTIME="${container_bin:-none}"
export BOX_PATH="${box_bin:-}"
export OPENSHELL_PATH="${openshell_bin:-}"
export HOST_OS="$(uname -s)"
export HOST_ARCH="$(uname -m)"
export OUT="$out"

bun -e '
const result = {
  schema_version: 1,
  evaluation: "strands-box-vs-openshell",
  generated_at: new Date().toISOString(),
  host: { os: process.env.HOST_OS, architecture: process.env.HOST_ARCH },
  availability: {
    box_cli: process.env.BOX_AVAILABLE === "true" || process.env.STRANDS_BOX_AVAILABLE === "true",
    box_path: process.env.BOX_PATH || null,
    openshell_cli: process.env.OPENSHELL_AVAILABLE === "true",
    openshell_path: process.env.OPENSHELL_PATH || null,
    container_runtime: process.env.CONTAINER_RUNTIME,
  },
  status: "unsupported",
  reason: "At least one candidate runtime is unavailable; no containment scenario was executed.",
  scenarios: [
    "declared-filesystem-read-write-boundary",
    "host-path-escape-and-symlink",
    "process-and-privilege-boundary",
    "network-default-deny-and-allowed-destination",
    "credential-non-disclosure",
    "cpu-memory-process-resource-limits",
    "agent-crash-and-runtime-recovery",
  ].map((id) => ({ id, strands_box: "unsupported", openshell: "unsupported", evidence: [] })),
  interpretation: "Availability checks are local observations only. Vendor descriptions below are not test results.",
};
await Bun.write(process.env.OUT, JSON.stringify(result, null, 2) + String.fromCharCode(10));
console.log(`containment evaluation: ${result.status}; report=${process.env.OUT}`);
'
