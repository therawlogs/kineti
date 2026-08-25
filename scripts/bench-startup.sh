#!/bin/sh
# Startup benchmark (Tier 2): spawn→exit wall time vs the /bin/true floor.
# Usage: scripts/bench-startup.sh [path-to-kineti]
#   KINETI_BENCH_HARD=1  → exit 1 when kineti p50 exceeds floor + 500 µs
set -eu

BIN="${1:-target/release/kineti}"
[ -x "$BIN" ] || { echo "bench: $BIN not found/executable" >&2; exit 2; }

python3 - "$BIN" <<'EOF'
import subprocess, sys, time, statistics, os

exe = sys.argv[1]
hard = os.environ.get("KINETI_BENCH_HARD") == "1"

def bench(cmd, n=300):
    subprocess.run(cmd, capture_output=True)  # warm page cache
    xs = []
    for _ in range(n):
        t0 = time.perf_counter()
        subprocess.run(cmd, capture_output=True)
        xs.append((time.perf_counter() - t0) * 1e6)
    xs.sort()
    return statistics.median(xs), xs[int(0.95 * len(xs))]

floor_p50, _ = bench(["/usr/bin/true"] if os.path.exists("/usr/bin/true") else ["/bin/true"])
kin_p50, kin_p95 = bench([exe, "--version"])

over = kin_p50 - floor_p50
print(f"floor (/bin/true)      p50 = {floor_p50:8.0f} µs")
print(f"kineti --version       p50 = {kin_p50:8.0f} µs   over-floor = {over:.0f} µs")
print(f"kineti --version       p95 = {kin_p95:8.0f} µs")

if over > 500:
    msg = f"startup budget: p50 is {over:.0f} µs over the process-creation floor (>500 µs)"
    if hard:
        print("⛔ " + msg); sys.exit(1)
    print("⚠ WARN: " + msg + " (soft gate; set KINETI_BENCH_HARD=1 to enforce)")
else:
    print("✔ within startup budget (≤ floor + 500 µs)")
EOF
