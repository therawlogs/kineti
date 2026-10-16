# Workflow contracts

This index defines the workflow contracts that connect the existing Kineti
skills. The numbered workflows are contract IDs; they do not change the
project's 13-stage numbering.

| Workflow | Inputs and order | Approval | Output and acceptance |
| --- | --- | --- | --- |
| 01 Discovery | Pain examples, system context, and user evidence; first workflow | Human approves the testable goal and framing | `brief.md` has primary/secondary personas, app model, user/client/server/AI responsibilities, and marked assumptions. Pass only with evidence-backed pain and at least one named primary persona. |
| 02 Design | Approved brief; precedes architecture and data intake | Human approves style brief, storyboard/mock-data shape, and selected design | `design/storyboard.md` covers the main path and relevant empty/error states; variants, comparison, and tokens trace to the approved design. |
| 03 Data intake | Approved storyboard and mock-data shape from 02; no real-data processing before separate approval | Human approves each real-data purpose and scope | `data-intake.md` records provenance, fields, purpose, sensitivity, retention/deletion, and real/mock status. If 02 approval is missing, stop and return to design. |
| 04 Feature discovery and architecture | Brief, storyboard, diagnostics, approved data scope, constraints | Human approves outcome map, features, architecture, and selected tools | `architecture.md` maps outcomes to features/tests and compares each selected tool with an alternative, including local/no-build options, costs, limits, failure behavior, and exit cost. |
| 06 Outcome contract and priorities | Approved brief, diagnostics, design, architecture, feasibility | Human approves spec, priority order, and out-of-scope list | `spec.md`, project-local `docs/`, roadmap and priorities. Each priority has acceptance criteria and dependencies; the spec gate remains pending until approval. |
| 07 Contract continuity | New request/change compared with locked root goal, approved brief, and spec | Human chooses among numbered, explained options when drift is detected | `contract-probe.md` records matching/differing statements, impact, and choice. No silent goal change; a new goal uses a new run. |
| 11 Product mode | Intended product capabilities, before setup/package selection | Human selects `harness`, `gate`, or `both` | Selection and included capabilities appear in the spec and setup result; editor/host selection does not imply product mode. |
| 12 Background adapters | Selected product mode and host capability contract | Human approves support claims; tests required before `supported` | Adapter runs alongside the foreground editor/agent and provides only declared capability. Untested integrations are `stub` or `unsupported`, never implicitly supported. |
| 13 Natural-language routing | Supported host, installed skills, routing examples | Human approves supported-host scope | Per-host request/expected-skill/negative-route matrix and inspectable results. No slash command is required for a passing natural-language case; untested hosts remain unsupported. |

## Dependency order

`01 → 02 → 03 → 04 → 06 → 07 → 11 → 12 → 13` is the default dependency
order. Workflow 03 may inspect only metadata until both the storyboard/mock
shape and real-data purpose are approved. A contract-continuity mismatch
may route back to an earlier workflow; it never silently changes the locked
goal.

## Support labels

Only integrations with acceptance evidence may be marked supported. Cursor
is the accepted adapter for the current protected-write gate. GitHub Action,
Codex, Claude Code, and any other host remain documented stubs or
unsupported for gate enforcement until their own acceptance tests pass.
Natural-language skill routing is host-specific and cannot be inferred
from a skill being installed.
