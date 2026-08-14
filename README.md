# Kineti OS (v2.0.0) — Autonomous Software Builder

> Low-cognitive-load software generation pipeline with deterministic background safety, modeled after modular skill architecture.

---

## 🏗️ Project Structure

```text
kineti_os/
├── kineti.config.json
├── ETHOS.md
├── WORKFLOWS.md
├── README.md
└── skills/
    ├── brainstorm.md
    ├── design.md
    ├── architecture.md
    ├── spec.md
    ├── build.md
    ├── qa.md
    └── ship.md
```

---

## ⚡ The Greenfield Pipeline

```text
/brainstorm ──> /design ──> /architecture ──> /spec ──> /build ──> /qa ──> /ship
 (Intake)        (UI/UX)       (System)     (Contract)   (Code)    (Test)   (Launch)
```

1. **`/brainstorm`** — Ingests raw concept, prompts for target audience (Internal, B2C, B2B), runs sanitized market research, and buckets features into P1, P2, and P3.
2. **`/design`** — Selects 1 of 3 Visual Archetypes, enforces the 12 UI standard components, and outputs high-fidelity screen preview layouts into `design/screens/`.
3. **`/architecture`** — Generates relational database schema (PostgreSQL), API contracts, and selects 1 of 3 tradeoff dials (Balanced, Zero Cost, Enterprise Fortress).
4. **`/spec`** — Assembles strict, typed data contracts into `spec.md` with a mandatory human approval gate.
5. **`/build`** — Spawns isolated sub-agents with immutable root goals and Saga LIFO rollback handlers to assemble production code in `src/`.
6. **`/qa`** — Runs automated Playwright multi-viewport verification (Desktop, Tablet, Mobile) with a 5-attempt self-healing loop and security scan.
7. **`/ship`** — Ingests environment secrets, launches live interactive preview, and manages production deployment under a strict $50 spend circuit breaker.

---

## 📜 Master Directives & Standards

All operational directives, background safety guardrails (Dual-LLM Sanitization, Spend Circuit Breaker, Saga LIFO Rollbacks), and Design System Standards are defined in [ETHOS.md](ETHOS.md).
