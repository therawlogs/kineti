# Kineti OS (v0.1) - Minimal Viable Reduction

Welcome to Kineti OS. This repository contains the core execution directives, agent tools, and configurations for a zero-decay software development framework.

## Single Source of Truth

**All operational logic, agent identity, and framework rules are defined in [ETHOS.md](ETHOS.md).** 

There are no secondary guides, persona documents, or overlapping master specifications. `ETHOS.md` governs every action taken by the agents within this OS.

## The 11 Core Skills

Kineti OS operates using a refined set of 11 core skills, found in the `skills/` directory:

### Core Pipeline
*   `/officehours`: Socratic discovery, domain validation, and 5-Whys root cause analysis.
*   `/spec`: Data quality audit, strict functional specifications, and API contracts (schema gate).
*   `/autoplan`: Enterprise-grade architecture planning (DB, auth, multi-tenancy, secrets, pipelines).
*   `/design`: Domain-specific visual audit, component library selection, and dynamic build execution.
*   `/ship`: Cost ceiling validation, deployment strategy, and release execution.

### Support Skills
*   `/design_consultation`: Standalone domain visual system generation.
*   `/browse`: Headless UI and layout verification.
*   `/qa`: Multi-viewport unit and integration testing with automated trace-and-patch loop.
*   `/review`: Production code linting, static checks, modularity, and error handling verification.
*   `/cso`: Dynamic spend bounds monitoring, PII tracking, and OWASP top 10 checks.
*   `/investigate`: Emergency trace/debugger loop to find and resolve regressions.

## Synchronization

If you modify these files locally, you can sync them to your global `~/.gemini/config/` directory by running:
```bash
python3 scripts/sync_kinetios.py
```
