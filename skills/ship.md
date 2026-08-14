# Skill: /ship

## Purpose
Collect deployment credentials, launch the live interactive preview, and execute production release.

## Execution Rules
1. Verify that `/qa` completed with zero failing tests.
2. Prompt the operator for required secrets:

```
Build verified. Enter any required environment variables/API keys below:
[ DATABASE_URL, API_KEYS ]
```

3. Spin up the live interactive preview environment.
4. **HARD PAUSE (Deploy Gate):**

```
# ============================================================
DEPLOY GATE: Application ready for launch.
Target Infrastructure: Edge Hosting / Containerized Node
Spend Circuit Breaker: Active ($50.00 Cap)

Type 'ship' to execute production release.
```

5. Upon confirmation, execute container build, deploy to target hosting, run live URL verification, and mark sprint COMPLETED in `state.json`.
