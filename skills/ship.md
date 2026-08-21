# Skill: /ship

## Purpose
Collect deployment credentials, launch the live interactive preview, and execute production release.

## Execution Rules
1. Verify that `/qa` completed with zero failing tests.
2. Prompt the operator for required secrets:

```
Build verified. Enter any required environment variables/API keys below:
- NEXT_PUBLIC_SUPABASE_URL
- NEXT_PUBLIC_SUPABASE_ANON_KEY
- SUPABASE_SERVICE_ROLE_KEY
- RESEND_API_KEY
- (or custom environment variables)
```

3. Spin up the live interactive preview environment.
4. **HARD PAUSE (Deploy Gate):**

```
# ============================================================
DEPLOY GATE: Application ready for launch.
Target Infrastructure: Vercel Edge Hosting
Spend Circuit Breaker: Active ($50.00 Cap)

Type 'ship' to execute production release.
```

5. Upon confirmation, execute build, deploy to target hosting (Vercel), run live URL verification, and mark sprint COMPLETED in `state.json`.
