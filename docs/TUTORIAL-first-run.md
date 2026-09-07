# First Run Tutorial

This tutorial walks you through using Kineti. It takes about 20 minutes.

You will need:
- Kineti installed (`./setup.sh`)
- An agent session open
- A small project or idea

You can start at any step, or run simple tasks directly (like bug fixes). If you are building a new project from scratch, follow these steps:

## Step 0: Check Installation

In your project folder, open your agent and ask:
> What kineti skills are available?

You should see skills starting with `kineti-`. If not, run `./setup.sh` from the Kineti folder.

## Step 1: Define the Goal (Office Hours)

Tell the agent your idea:
> Load kineti-officehours. My idea: <one sentence describing your idea>

The agent asks clarifying questions to find the simplest useful version. It creates `brief.md` and locks your main goal in `.kineti/state.json`.

Once locked, the goal cannot be changed by the agent. This prevents unintended drift.

## Step 2: Measure the Problem (Diagnose)

Ask the agent:
> Run kineti-diagnose against <the process from your brief>

The agent calculates where time or money is lost. It writes the results and calculations to `diagnostics.md`.

## Step 3: Choose the Visual Style (Design)

Ask the agent:
> Run kineti-design

Name three websites or apps whose look you like. The agent writes a design summary and creates HTML screen previews. Pick the layout you prefer.

## Step 4: Map Services and Failures (Architecture)

Ask the agent:
> Run kineti-architecture

The agent creates a diagram of services, lists error scenarios, and compares hosting costs.

## Step 5: Check Practical Limits (Feasibility)

Ask the agent:
> Run kineti-feasibility

The agent checks three areas:
1. Money: Expected cost and return.
2. Data: Quality of necessary input data.
3. Users: Key people who must approve the work.

If any check fails, the agent reports the issue and stops so you can fix it early.

## Step 6: Write the Specification (Spec)

Ask the agent:
> Run kineti-spec

The agent writes clear data types, pass/fail test rules, and a list of what will not be built.

The agent stops here and waits for your approval. No code is written in `src/` until you approve.

## Step 7: Build, Review, and Ship (Build → QA → Ship)

Once you approve the spec, tell the agent:
> Approve the spec. Run kineti-build.

1. **Build**: Code is written in small steps. Every file change registers an undo action in `.kineti/saga.jsonl`.
2. **Review**: The agent checks for logic errors and edge cases.
3. **QA**: The agent runs tests in a real browser at mobile, tablet, and desktop widths.
4. **Security**: The agent runs standard security checks.
5. **Ship**: The agent checks that all tests passed recently and creates the pull request or commit.

## Step 8: Monitor and Learn (Watch and Review)

After deployment:
- **Watch**: Check for errors or slow response times. Alerts go to `~/.kineti/alerts.log`.
- **Review**: At the end of each week, run `kineti-retro` to record lessons learned so future tasks use them.
