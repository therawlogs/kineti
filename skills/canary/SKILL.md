---
name: /canary
description: SRE post-deployment telemetry analyzer. Hooks into production log streams to capture latency changes and trace active faults.
---

# Skill: /canary (Stage 04 - Post-Deployment Canary Telemetry)

## When to Use
Immediately following production deployment.

## Sequencing
- **Phase**: `04_verification_release`
- **Step**: 3

## Protocol & Actions
- **Instructions**:
  When this skill is run, you **MUST** prompt the user to configure these Stage 4 canary parameters (Question 9):

  ### 9. Select post-deployment canary telemetry focus:
  *   **Option A: Log Streams Error Metrics** (RECOMMENDED)
      ├─ Info: Hooks directly into application stdout/stderr streams to count warning and error logs.
      ├─ Why: Detects internal execution faults and database database timeout anomalies instantly.
      └─ If Fails: High volume logging logs can cause buffer latency under spike loads.
  *   **Option B: Active HTTP pinging checks**
      ├─ Info: Automatically runs periodic curl/fetch queries against the public url endpoints.
      ├─ Why: Direct verification of app server response and availability status.
      └─ If Fails: Misses internal memory leaks or background cron job errors.

  Obtain choices and request approval before initiating canary telemetry tracking.