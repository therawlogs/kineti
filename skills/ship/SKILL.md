---
name: ship
description: Deployment and release gate. Final skill in the pipeline.
---

**Step 1: Load State**
Read the `.northstar/sprint_state.json` file. Check that `last_completed_gate` is set to `design`. If not, stop and report the error.

**Step 2: Cost Architecture (Stage 6)**
Ask the user the following questions:
- Q1. Unit Cost Ceiling: What is the maximum cost per user action to maintain a >80% gross margin?
- Q2. Spend Circuit Breaker: What API cost cap triggers auto-stop?
- Q3. Outage Cost per Hour: $0 / Under $100 / $100-$10k / Over $10k
Generate and display 2-3 domain-specific cost probes based on the project type.

**Step 3: Deployment & Operations (Stage 7)**
Present 3 researched infrastructure options:
- Option A: Edge deployment (Vercel/Cloudflare) - for consumer apps, marketing sites
- Option B: Dedicated compute (Railway/Render/Fly.io) - for API-heavy, real-time apps
- Option C: Self-hosted/local - for regulated industries, data privacy

Ask the user:
- Q1. Target hosting (from options above)
- Q2. Rollback plan: How to undo a failed deployment automatically?
- Q3. Outage detection: Error rate tracking / Metric anomaly / User-reported
- Q4. Secret storage: Environment variables / Secrets Manager / Local .env
- Q5. Pass/fail criteria: What test proves the sprint is complete?

Present all options and STOP EXECUTION. Wait for user approval before deploying.

**Step 4: Execute Release**
After the user approves the deployment:
1. Execute the `/qa` command to run the full test suite across viewports.
2. Execute the `/cso` command to run the security audit.
3. Deploy the application to the chosen target hosting provider.
4. Execute the `/browse` command to verify the production URL loads and renders correctly.

**Step 5: Sprint Closure**
Update `.northstar/sprint_state.json` to mark the sprint as completed. Print a financial summary showing time spent, tokens used, total cost, and projected margin.