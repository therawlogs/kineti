# Workflow contracts

These workflow IDs remain stable for saved projects and records. This guide
defines a suggested path for new projects; direct fixes, reviews, and cleanup
can enter at the relevant step. Written guidance does not enforce host
behavior by itself.

At new-project activation, ask the user to choose `harness`, `gate`, or
`both`. Do not preselect a mode. A canceled choice leaves the project
inactive; installing Kineti does not activate a project. Legacy projects
remain readable with mode selection pending.

| Workflow | Inputs and order | Approval | Output and acceptance |
| --- | --- | --- | --- |
| 01 Discovery | Request, available evidence, constraints, and intended users | User approves the goal and framing for a new run | `brief.md` records the goal, personas and needs, initial feature ideas, app model, responsibilities, constraints, assumptions, and unanswered questions. Noncommercial goals and incomplete evidence are valid. |
| 02 Design | Approved brief; before data intake and architecture | User approves storyboard and mock-data shape | `design/storyboard.md` covers the main journey and relevant empty/error states. Design options and references are used when they help; fixed counts are not required. |
| 03 Data intake | Approved storyboard/mock shape, or an explicit no-data path | Separate approval for each real-data purpose | `data-intake.md` records provenance, purpose, sensitivity, retention, and real/mock status for inputs used. No-data projects use synthetic fixtures where needed. |
| 04 Feature and architecture choices | Approved brief, design, data scope or no-data choice, constraints, and relevant diagnosis | User approves accepted scope, tool choices, and architecture | Features link to user needs, outcomes, and tests. Relevant alternatives, costs, limits, failure behavior, and exit cost are recorded. Paid services are never selected silently. |
| 06 Outcome contract and priorities | Accepted scope, design, architecture, feasibility, and unresolved decisions | User approves artifact versions and confirmed priorities | Local documents link goals, personas, design, data, features, architecture, tools, tests, roadmap, priorities, and dependencies. Missing approvals block affected implementation. |
| 07 Contract continuity | Every new request or change, compared with the locked goal and approved choices | User chooses numbered options when a conflict affects work | Preserve earlier decisions; record matching and differing statements, evidence, and affected work. Only conflicting work pauses. A new goal starts a separate run. |
| 11 Product mode | At new-project activation, before workflow setup | User explicitly selects `harness`, `gate`, or `both` | Persist mode and capabilities. Ordinary chat and package installation do not create project state or scaffolding. |
| 12 Host capabilities | Versioned host capability declaration and isolated install | Each capability claim needs its own acceptance evidence | Declare installation, workflow guidance, background behavior, natural-language routing, and protected actions separately. Guidance text is not enforcement. |
| 13 Natural-language routing | Claimed host support, installed skills, and routing cases | User approves the supported-host scope | Use isolated live-model runs for routing claims. Static text checks verify instructions only. Slash commands may be optional. |

## Order and direct tasks

For a new project, use: explicit mode selection → 01 discovery → 02 design
and mock-data approval → 03 approved data intake or no-data path → 04 refined
features, architecture, tool choices, and feasibility → 06 approved outcome
contract → build and verification. Historical stage identifiers remain
readable; save explicit workflow-step and approval records instead of
renumbering stages.

The existing diagnosis skill (historical stage 2) is optional supporting
analysis. Use it when relevant after design approval and before processing
any real data. It may use synthetic examples or approved inputs. It does not
require business-loss calculations. Report measured evidence and assumptions;
time order or dependency alone does not establish cause.

Workflow 07 continuity checks apply throughout. Bug fixes, reviews, and
cleanup can start at the relevant point. Small tasks do not need to traverse
every stage. Setup and routing are checked only for hosts and capabilities
that the project claims to support.

## Support declarations

Each host advertises only capabilities with passing evidence in the
versioned `hosts/capabilities.v1.json` declaration. Keep distinct claims for
installation, guidance, background behavior, natural-language routing, and
protected actions. An integration may support some capabilities and leave
others unsupported. A second-host stub must return unsupported; it does not
count as enforcement. Protected-action claims need real allow and deny
evidence for the declared action.

The installation path and user guidance describe available behavior; they do
not imply transparent interception of arbitrary tools or workspace writes.
Natural-language routing is host-specific and must be tested with the
declared live runner before being marked supported.
