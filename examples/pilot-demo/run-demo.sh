#!/usr/bin/env bash
# Kineti 1.0 — Self-Contained Offline AI-PR Gate Demo
# Runs 4 scenes in a disposable temporary Git repository with zero network access:
#   Scene 1 (Blocked, exit 1): PR weakens base-branch .kineti/gate.json
#   Scene 2 (Blocked, exit 1): PR adds @pytest.mark.skip / it.skip / #[ignore] and deletes a test
#   Scene 3 (Blocked, exit 4): 1-byte edit in src/legacy/auth.py invalidates fresh evidence
#   Scene 4 (Passed,  exit 0): Clean fix passes supervised tests + emits signed kineti-pr-audit-v1

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

resolve_bin() {
  if [[ -n "${KINETI_BIN:-}" ]]; then
    if [[ "$KINETI_BIN" == */* ]]; then
      local dir base
      dir="$(cd "$(dirname "$KINETI_BIN")" && pwd)"
      base="$(basename "$KINETI_BIN")"
      printf '%s/%s\n' "$dir" "$base"
    else
      command -v "$KINETI_BIN"
    fi
    return
  fi
  if [[ -x "$REPO_ROOT/core-native/target/release/kineti" ]]; then
    printf '%s\n' "$REPO_ROOT/core-native/target/release/kineti"
    return
  fi
  if [[ -x "$REPO_ROOT/core-native/target/debug/kineti" ]]; then
    printf '%s\n' "$REPO_ROOT/core-native/target/debug/kineti"
    return
  fi
  if command -v kineti >/dev/null 2>&1; then
    command -v kineti
    return
  fi
  return 1
}

banner() {
  printf '\n========================================================================\n'
  printf '  %s\n' "$1"
  printf '========================================================================\n'
}

require_tool() {
  if ! command -v "$1" >/dev/null 2>&1; then
    printf 'ERROR: required tool not found on PATH: %s\n' "$1" >&2
    exit 2
  fi
}

require_tool git
require_tool python3

if ! BIN_PATH="$(resolve_bin)"; then
  cat >&2 <<'EOF'
ERROR: Could not locate the native `kineti` binary.
Build from source first:
  cargo build --release --manifest-path core-native/Cargo.toml
Or pass an explicit path:
  KINETI_BIN=/path/to/kineti ./examples/pilot-demo/run-demo.sh
EOF
  exit 2
fi

if [[ ! -x "$BIN_PATH" ]]; then
  printf 'ERROR: KINETI_BIN is not executable: %s\n' "$BIN_PATH" >&2
  exit 2
fi

TMP_DIR="$(mktemp -d "${TMPDIR:-/tmp}/kineti-pilot-demo.XXXXXX")"
WORK_DIR="$TMP_DIR/repo"
DEMO_HOME="$TMP_DIR/home"

cleanup() {
  rm -rf "$TMP_DIR"
}
trap cleanup EXIT INT TERM

mkdir -p "$WORK_DIR/.kineti" "$WORK_DIR/src/legacy" "$WORK_DIR/tests" "$DEMO_HOME"

banner "Kineti 1.0 Offline AI-PR Gate Demo (Zero Network Calls)"
printf 'Executable : %s\n' "$BIN_PATH"
printf 'Workspace  : disposable temp repo (auto-cleaned on exit)\n'

cd "$WORK_DIR"
git init -q
git checkout -q -b main
git config user.name "Kineti Pilot Demo"
git config user.email "demo@getkineti.example"

cat > .gitignore <<'EOF'
.kineti/*
!.kineti/gate.json
__pycache__/
.pytest_cache/
EOF

: > src/__init__.py
: > src/legacy/__init__.py

cat > src/legacy/auth.py <<'EOF'
def verify_token(token: str) -> bool:
    """Reject empty tokens and require the expected prefix."""
    if not isinstance(token, str):
        return False
    return token.startswith("knt_") and len(token) >= 12


def is_session_active(age_seconds: int, max_age_seconds: int = 3600) -> bool:
    """Return True only when session age is non-negative and within max_age."""
    return 0 <= age_seconds <= max_age_seconds
EOF

cat > tests/test_auth.py <<'EOF'
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
from src.legacy.auth import is_session_active, verify_token


class AuthTests(unittest.TestCase):
    def test_token_validation(self) -> None:
        self.assertTrue(verify_token("knt_prod_987654"))
        self.assertFalse(verify_token("bad"))
        self.assertFalse(verify_token(""))

    def test_session_expiry(self) -> None:
        self.assertTrue(is_session_active(120))
        self.assertFalse(is_session_active(7200))
        self.assertFalse(is_session_active(-1))


if __name__ == "__main__":
    unittest.main()
EOF

cat > tests/billing.test.js <<'EOF'
function calculateTotal(cents, taxBps) {
  return cents + Math.round((cents * taxBps) / 10000);
}

it("calculates invoice total with tax", () => {
  if (calculateTotal(10000, 825) !== 10825) {
    throw new Error("unexpected total");
  }
});

test("handles zero tax", () => {
  if (calculateTotal(5000, 0) !== 5000) {
    throw new Error("unexpected zero-tax total");
  }
});
EOF

cat > tests/core_test.rs <<'EOF'
#[test]
fn test_gate_enforces_exact_policy() {
    assert_eq!(2 + 2, 4);
}

#[test]
fn test_rejects_empty_label() {
    assert!(!"".ends_with("unit"));
}
EOF

cat > .kineti/gate.json <<'EOF'
{
  "version": 1,
  "required_checks": ["unit"],
  "commands": {
    "unit": ["python3", "-B", "tests/test_auth.py"]
  },
  "parsers": {
    "unit": "exit-code"
  },
  "required_tests": [
    "tests/test_auth.py",
    "tests/billing.test.js",
    "tests/core_test.rs"
  ],
  "protected_paths": [
    ".kineti/gate.json",
    ".github/workflows/",
    "pytest.ini"
  ],
  "max_deleted_tests": 0,
  "max_age_minutes": 240,
  "exclude": [".pytest_cache", "__pycache__"]
}
EOF

git add .gitignore .kineti/gate.json src/ tests/
git commit -q -m "chore: establish protected base branch and kineti gate policy"

# Initialize local runtime state using an isolated HOME so host configs are untouched.
env -u CODEX_HOME HOME="$DEMO_HOME" \
  "$BIN_PATH" init --goal "Kineti 1.0 offline AI-PR gate demo" --apply >/dev/null

# ------------------------------------------------------------------------------
# SCENE 1 (Blocked): AI PR weakens base-branch .kineti/gate.json
# ------------------------------------------------------------------------------
banner "SCENE 1 [BLOCKED]: AI PR Weakens Protected .kineti/gate.json"
git checkout -q -b pr/scene-1-policy-tamper main

cat > .kineti/gate.json <<'EOF'
{
  "version": 1,
  "required_checks": [],
  "commands": {
    "unit": ["/bin/sh", "-c", "true"]
  },
  "required_tests": [],
  "protected_paths": [],
  "max_deleted_tests": 99
}
EOF

git add .kineti/gate.json
git commit -q -m "ai: simplify gate policy"

set +e
S1_OUT="$("$BIN_PATH" pr-check --base-ref main --head-ref HEAD 2>&1)"
S1_RC=$?
set -e

printf '%s\n' "$S1_OUT"
printf '\nObserved exit code: %d (expected: 1)\n' "$S1_RC"
if [[ "$S1_RC" -ne 1 ]]; then
  printf 'FAIL: Scene 1 expected exit 1, got %d\n' "$S1_RC" >&2
  exit 1
fi
if ! printf '%s\n' "$S1_OUT" | grep -q "policy_tamper_detected"; then
  printf 'FAIL: Scene 1 expected policy_tamper_detected in output\n' >&2
  exit 1
fi
printf 'RESULT: BLOCKED as expected (policy_tamper_detected)\n'

git checkout -q main

# ------------------------------------------------------------------------------
# SCENE 2 (Blocked): AI PR injects skip/ignore markers & deletes a test
# ------------------------------------------------------------------------------
banner "SCENE 2 [BLOCKED]: AI PR Adds Skip/Ignore Directives & Deletes a Test"
git checkout -q -b pr/scene-2-test-tamper main

cat > tests/test_auth.py <<'EOF'
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
from src.legacy.auth import verify_token


class AuthTests(unittest.TestCase):
    @pytest.mark.skip(reason="temporarily skipped by agent")
    def test_token_validation(self) -> None:
        self.assertTrue(verify_token("knt_prod_987654"))


if __name__ == "__main__":
    unittest.main()
EOF

cat > tests/billing.test.js <<'EOF'
function calculateTotal(cents, taxBps) {
  return cents + Math.round((cents * taxBps) / 10000);
}

it.skip("calculates invoice total with tax", () => {
  if (calculateTotal(10000, 825) !== 10825) {
    throw new Error("unexpected total");
  }
});

test("handles zero tax", () => {
  if (calculateTotal(5000, 0) !== 5000) {
    throw new Error("unexpected zero-tax total");
  }
});
EOF

cat > tests/core_test.rs <<'EOF'
#[test]
fn test_gate_enforces_exact_policy() {
    assert_eq!(2 + 2, 4);
}

#[test]
#[ignore]
fn test_rejects_empty_label() {
    assert!(!"".ends_with("unit"));
}
EOF

git add tests/
git commit -q -m "ai: stabilize flaky tests"

set +e
S2_OUT="$("$BIN_PATH" pr-check --base-ref main --head-ref HEAD --audit-out .kineti/scene2-audit.json 2>&1)"
S2_RC=$?
set -e

printf '%s\n' "$S2_OUT"
python3 -c '
import json
a = json.load(open(".kineti/scene2-audit.json"))
for f in a["test_tamper_findings"]:
    print("  - [" + str(f["rule"]) + "] " + str(f["file"]) + ":" + str(f["line"]) + " -> " + str(f["snippet"]))
'
printf '\nObserved exit code: %d (expected: 1)\n' "$S2_RC"
if [[ "$S2_RC" -ne 1 ]]; then
  printf 'FAIL: Scene 2 expected exit 1, got %d\n' "$S2_RC" >&2
  exit 1
fi
printf 'RESULT: BLOCKED as expected (polyglot test tampering detected)\n'

git checkout -q main

# ------------------------------------------------------------------------------
# SCENE 3 (Blocked): 1-byte edit to src/legacy/auth.py triggers stale check
# ------------------------------------------------------------------------------
banner "SCENE 3 [BLOCKED]: 1-Byte Edit in src/legacy/auth.py Invalidates Evidence"
git checkout -q -b pr/scene-3-stale-legacy main

printf 'Step 3a: Running supervised test suite (--parser exit-code)...\n'
"$BIN_PATH" test run \
  --label unit \
  --required-test tests/test_auth.py \
  --parser exit-code \
  -- python3 -B tests/test_auth.py

printf '\nStep 3b: Checking fresh receipt before edit...\n'
"$BIN_PATH" test check --label unit

printf '\nStep 3c: Applying 1-byte edit to src/legacy/auth.py (3600 -> 3601)...\n'
python3 -c '
from pathlib import Path
p = Path("src/legacy/auth.py")
before = p.read_text(encoding="utf-8")
after = before.replace("3600", "3601", 1)
assert len(after.encode("utf-8")) == len(before.encode("utf-8")) and after != before
p.write_text(after, encoding="utf-8")
'

printf 'Step 3d: Re-checking evidence after 1-byte edit...\n'
set +e
S3_OUT="$("$BIN_PATH" test check --label unit 2>&1)"
S3_RC=$?
set -e

printf '%s\n' "$S3_OUT"
printf '\nObserved exit code: %d (expected: 4)\n' "$S3_RC"
if [[ "$S3_RC" -ne 4 ]]; then
  printf 'FAIL: Scene 3 expected exit 4 (stale), got %d\n' "$S3_RC" >&2
  exit 1
fi
printf 'RESULT: BLOCKED as expected (exit 4 — src/legacy/ is hashed, never silently skipped)\n'

git checkout -q -- src/legacy/auth.py
git checkout -q main

# ------------------------------------------------------------------------------
# SCENE 4 (Passed): Clean Fix PR -> Signed kineti-pr-audit-v1 + Markdown Summary
# ------------------------------------------------------------------------------
banner "SCENE 4 [PASSED]: Clean Fix PR Emits Signed kineti-pr-audit-v1 & Summary"
git checkout -q -b pr/scene-4-clean-fix main

cat > src/legacy/auth.py <<'EOF'
def verify_token(token: str) -> bool:
    """Reject empty tokens, whitespace, and require the expected prefix."""
    if not isinstance(token, str) or any(ch.isspace() for ch in token):
        return False
    return token.startswith("knt_") and len(token) >= 12


def is_session_active(age_seconds: int, max_age_seconds: int = 3600) -> bool:
    """Return True only when session age is non-negative and within max_age."""
    return 0 <= age_seconds <= max_age_seconds
EOF

cat > tests/test_auth.py <<'EOF'
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
from src.legacy.auth import is_session_active, verify_token


class AuthTests(unittest.TestCase):
    def test_token_validation(self) -> None:
        self.assertTrue(verify_token("knt_prod_987654"))
        self.assertFalse(verify_token("bad"))
        self.assertFalse(verify_token(""))

    def test_session_expiry(self) -> None:
        self.assertTrue(is_session_active(120))
        self.assertFalse(is_session_active(7200))
        self.assertFalse(is_session_active(-1))

    def test_rejects_whitespace_token(self) -> None:
        self.assertFalse(verify_token("knt_prod_ 987654"))


if __name__ == "__main__":
    unittest.main()
EOF

git add src/legacy/auth.py tests/test_auth.py
git commit -q -m "fix(auth): reject whitespace inside bearer tokens"

printf 'Step 4a: Running supervised tests on clean fix PR...\n'
"$BIN_PATH" test run \
  --label unit \
  --required-test tests/test_auth.py \
  --parser exit-code \
  -- python3 -B tests/test_auth.py

printf '\nStep 4b: Running kineti pr-check with signed audit & summary output...\n'
"$BIN_PATH" pr-check \
  --base-ref main \
  --head-ref HEAD \
  --require-evidence unit \
  --audit-out .kineti/pr-audit.json \
  --summary-out .kineti/pr-summary.md

if [[ ! -s .kineti/pr-audit.json || ! -s .kineti/pr-summary.md ]]; then
  printf 'FAIL: Scene 4 expected non-empty .kineti/pr-audit.json and .kineti/pr-summary.md\n' >&2
  exit 1
fi

printf '\n--- Signed kineti-pr-audit-v1 Bundle ---\n'
python3 -m json.tool .kineti/pr-audit.json

printf '\n--- Markdown PR Audit Table (.kineti/pr-summary.md) ---\n'
cat .kineti/pr-summary.md

banner "DEMO COMPLETE — ALL 4 SCENES VERIFIED"
printf '  Scene 1 (Policy tamper blocked)   : exit %d (expected 1)\n' "$S1_RC"
printf '  Scene 2 (Test tamper blocked)     : exit %d (expected 1)\n' "$S2_RC"
printf '  Scene 3 (src/legacy drift blocked): exit %d (expected 4)\n' "$S3_RC"
printf '  Scene 4 (Clean PR signed pass)    : exit 0 (expected 0)\n'
printf '\nNote: Signatures prove record integrity relative to the signing key, not\n'
printf 'assertion quality. --parser exit-code records count_observed: false.\n'
