# Kineti demo

## v0.2 — 30s receipt + gate (default product)

One terminal, no API key needed. Shows the meter and the stamp.

```sh
mkdir -p /tmp/kineti-demo && cd /tmp/kineti-demo
kineti init
echo 'echo hi' > greeting.sh && chmod +x greeting.sh

kineti evidence --cmd "./greeting.sh"   # bind proof to artifact fingerprint (any verify cmd)
kineti ship-check                       # 0 = fresh
kineti receipt                          # $ + head + gates
kineti verify --all                     # offline hash-chain

# generic: same gate works for docs / data / configs, not just code
#   kineti evidence --cmd "pytest"      # or "npm test" / "make check" / "./verify.sh"
#   kineti evidence --cmd "python -m verify_data"

# sabotage: edit artifact, then see gate refuse
echo 'echo hacked' > greeting.sh
kineti ship-check                       # ⛔ STALE proof (artifacts changed)

kineti evidence --cmd "./greeting.sh"   # re-bind after fix
kineti ship-check                       # ✔ FRESH

# swarm: one command fans out N agents (LLM prompts + shell commands) under one cap
cat > tasks.jsonl <<'EOF'
{"id":"a","prompt":"Write docs/notes.md with one insight about Kineti"}
{"id":"b","command":"./greeting.sh"}
EOF
kineti swarm --tasks tasks.jsonl --cap 5
```

With gateway (optional demo):

```sh
kineti gateway --port 8787 &
# point any OpenAI-compatible agent at http://localhost:8787/v1
# gateway does reserve → forward → settle, writes receipt (hashes only)
curl -s http://localhost:8787/v1/chat/completions -H "Content-Type: application/json" \
  -d '{"model":"gemini-3.6-flash","messages":[{"role":"user","content":"hi"}]}' | head
kineti receipt
```

CI: same gate via `.github/actions/kineti-receipt` — set `verify-command: "cargo test --all"` and the action runs `evidence --cmd` then `ship-check`+`verify --all` (no prior `.kineti/` needed).

---

## v0.1 — 90s 13-stage pipeline (frozen, `kineti run --legacy`)

> Frozen on tag `v0.1.0` (`docs/v0.1.md`). Use `kineti run --legacy --goal` — hidden, prints legacy warning (`kineti run --help` shows `--legacy`; without `--legacy` still works). Kept for reference; not the homepage.

Setup:
```sh
mkdir -p /tmp/kineti-demo && cd /tmp/kineti-demo
kineti init
export GEMINI_API_KEY=...  # or XAI_API_KEY with --provider grok
# proof command in kineti.toml so ship gate has something to bind:
#   [proof] command = "./greeting.sh"
# legacy alias still works: [limits] verify_command
```

Script (legacy pipeline):

| Time | Command | What the audience sees |
|---|---|---|
| 0:00 | `kineti --version && ls -lh $(which kineti)` | single small binary — no runtime |
| 0:10 | `kineti run --legacy --provider grok --cap 0.001 --goal "Create greeting.sh that prints KINETI SHIPS"` | **breaker trips** — halts, demands `touch .kineti/spend.reset` |
| 0:25 | `touch .kineti/spend.reset` | human gate only |
| 0:30 | `kineti resume` (to stage 6) | SPEC HARD STOP — type `n` once → refuses to write code |
| 0:55 | resume again, answer `y` | build → review → qa |
| 1:05 | edit `greeting.sh` by hand | sabotage |
| 1:10 | pipeline hits ship gate | `SHIP REFUSED — STALE proof` |
| 1:15 | `kineti evidence --cmd "./greeting.sh" && kineti resume` | fresh proof → `🚢 SHIPPED` |
| 1:20 | `./greeting.sh && kineti receipt && kineti verify` | artifact + receipt + chain head |

Talking points (legacy):
- "Every safeguard is control flow, not prompt text."
- "Memory is sha256-chained. One byte flip → `kineti verify` fails."
- "Breaker stops money unless a human moves it."

Recording notes: font ≥16pt, dark theme, 120×32, `asciinema rec` at 2× zoom. Reset between takes: `rm -rf .kineti greeting.sh`.
