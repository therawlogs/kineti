---
name: officehours
description: Discovery and strategy gate for new projects and features.
---

# Office Hours Skill

This is the first skill to run when starting a new project or feature. It conducts a foundational discovery process to establish the context, strategy, and business constraints before writing any code.

## 1. 3-Layer Knowledge Search
Research the domain using these perspectives before asking questions:
- **Tried & True**: Proven industry standards and established patterns.
- **New & Popular**: Recent trends, modern tools, and contemporary approaches.
- **First Principles**: Core truths of the problem stripped of current solutions.
- **Eureka Moment**: Unconventional connections from outside the immediate domain.

## 2. Stage 0 Discovery
Present these questions to the user and wait for their answers:
- **Business Model**: Is this B2C, B2B, B2G, internal tool, or personal hobby?
- **Industry Domain**: What specific industry does this operate within?
- **Environment**: Is this a greenfield project (new) or brownfield (existing codebase)?
- **Risk Level**: What is the tolerance for failure or bugs?
- **Timeline Pressure**: What are the critical deadlines?

Generate follow-up, domain-specific questions based on the answers received.

## 3. Stage 1 Strategy
Present these questions to establish the project's strategic foundation:
- **5-Whys Root Cause**: What is the actual underlying problem we are solving?
- **Single Main Value Action**: What is the one core action the user must achieve?
- **Status Quo Annual Cost**: What does the current inefficient process cost in time or money?
- **Target Business Metric**: What specific metric determines this project's success?
- **Project Veto Holder**: Who has the authority to stop or reject this project?
- **Counter-Argument Refutation**: Why might this project fail, and how do we prevent it?

## 4. Memory Initialization
Create the private project memory structure.
1. Create the `.northstar/` directory at the project root.
2. Create subdirectories: `.northstar/dialogue/`, `.northstar/gaps/`, `.northstar/decisions/`, `.northstar/templates/`, `.northstar/mocks/`.
3. Create `.northstar/.gitignore` containing exactly `*` to ensure these files are never committed to the public repository.

## 5. Artifact Generation & Gap Detection
1. Save the transcript of the discovery conversation to `.northstar/dialogue/`.
2. Identify missing information or conflicting requirements and document them in `.northstar/gaps/`.
3. Generate any basic templates or text mockups needed based on the conversation and save them to `.northstar/templates/` and `.northstar/mocks/`.

## 6. Summary and Handoff
Do not automatically run downstream skills. End the interaction by presenting a summary of the decisions made, listing any remaining open questions, and stating the next available options.