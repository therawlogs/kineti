# No-key demo clip

**Clip status: pending.** The no-key demo script is ready, but this environment does not permit opening the system Terminal app for screen capture.

When recording becomes available:

1. Record the terminal at a readable size, with the repository at the project root.
2. Run `DEMO_PAUSE_SECONDS=4 ./demo-no-key.sh` to hold each of the eight lifecycle steps on screen. The pauses add 32 seconds; test execution brings the clip to roughly 35–45 seconds.
3. Keep the full sequence visible: tests and proof, fresh check, source edit, stale check, restore, new proof, fresh check.
4. Save a 30–60 second clip under `docs/media/no-key-demo.mp4`, review its duration and readability, then add the README star line and link to the clip.
5. Do not record API keys, private environment values, or unrelated desktop content.

The script accepts `DEMO_PAUSE_SECONDS=0` by default, so normal runs stay fast. It restores `bin/lib.ts` from a byte-for-byte backup on normal exit and shell interruption.
