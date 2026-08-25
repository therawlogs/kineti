# The 90-second Kineti demo

One terminal, one clean project directory, one API key in the environment.
Every step below is a real gate firing mechanically — nothing is staged.

## Setup (before recording)

```sh
mkdir -p /tmp/kineti-demo && cd /tmp/kineti-demo
kineti init
# set verify command so the ship gate has something to bind:
#   [limits] verify_command = "./greeting.sh"
export GEMINI_API_KEY=...   # or XAI_API_KEY with --provider grok
```

## Script

| Time | Command | What the audience sees |
|---|---|---|
| 0:00 | `kineti --version && ls -lh $(which kineti)` | single small binary — no runtime, no node_modules |
| 0:10 | `kineti run --provider grok --cap 0.001 --goal "Create greeting.sh that prints KINETI SHIPS"` | **breaker trips mid-run** — halts between stages, demands a human flag file |
| 0:25 | `touch .kineti/spend.reset` | the only way past §3: a human decision |
| 0:30 | `kineti resume` (let it run to stage 6) | stage banners stream; artifacts accumulate |
| 0:45 | SPEC HARD STOP appears — type `n` once | **the machine refuses to write code**; regenerates spec |
| 0:55 | resume again, answer `y` | code tools unlock; build → review → qa run |
| 1:05 | edit greeting.sh by hand after qa | sabotage for science |
| 1:10 | pipeline hits ship gate | `SHIP REFUSED — STALE proof` — it *knows* you touched the file |
| 1:15 | `kineti evidence --cmd "./greeting.sh" && kineti resume` | fresh proof bound → `🚢 SHIPPED.` |
| 1:20 | `./greeting.sh && kineti receipt && kineti verify` | working artifact + hash-chained receipt + chain head |

## Talking points while it runs

- "Every safeguard you just saw is control flow, not prompt text. The model
  cannot talk its way past the spec stop."
- "Memory is sha256-chained. I can flip one byte and `kineti verify` fails."
- "The breaker is how enterprises will sleep at night: money stops moving
  unless a human moves it."
- "This whole harness is about three megabytes. No framework, no runtime."

## Recording notes

- Terminal font ≥16pt, dark theme, 120×32 window.
- Record with `asciinema rec` or screen capture at 2× zoom on gates.
- Reset state between takes: `rm -rf .kineti greeting.sh`.
