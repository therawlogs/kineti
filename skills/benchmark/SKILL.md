---
name: /benchmark
description: Measures transaction speeds, resource footprints, and Web Vitals to assert zero performance regression under load.
---

# Skill: /benchmark (Stage 04 - Performance Vitals Benchmarking)

## When to Use
During the post-deployment release gate validation.

## Sequencing
- **Phase**: `04_verification_release`
- **Step**: 4

## Protocol & Actions
- **Instructions**:
  When this skill is run, you **MUST** prompt the user to configure these Stage 4 vitals parameters (Question 10):

  ### 10. Select performance vitals benchmark gate policy:
  *   **Option A: Core Web Vitals threshold (Block build)** (RECOMMENDED)
      ├─ Info: Measures LCP (Largest Contentful Paint) and INP (Interaction to Next Paint) speeds, failing the build if thresholds are breached.
      ├─ Why: Enforces world-class client loading speeds, preventing design bloat.
      └─ If Fails: Local server testing can report false positives due to network speeds.
  *   **Option B: Telemetry logging only**
      ├─ Info: Records transaction speeds to build logs without interrupting deployment steps.
      ├─ Why: High release velocity.
      └─ If Fails: Slow rendering scripts can creep into production.

  Obtain choices and request approval before executing performance vitals benchmarks.