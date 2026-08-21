# Tutorial — Your First Kineti Run

Follow this once and you will know the whole system. Time: about 20
minutes. You need: this repository installed (`./setup.sh`), an agent
session open in opencode, and any small idea you actually care about.

## 0. Confirm the ground

In a scratch project folder (not the Kineti repo), start an agent session
and ask:

> What kineti skills are available?

You should see the `kineti-*` list. If not, re-run `./setup.sh` from the
repository and reopen the session.

## 1. Office hours (stage 1)

> Load kineti-officehours. My idea: <one sentence about your idea>

The skill interviews you before it plans. Expect pushback: it asks for
real examples of pain, challenges your framing, and proposes the smallest
useful version. It ends by writing `brief.md` and locking your goal:

```sh
cat .kineti/state.json   # root_goal is now immutable
```

Try to change the goal and watch it refuse — that refusal is the system
working.

## 2. Diagnose (stage 2)

> Run kineti-diagnose against <the process from your brief>

Answer its questions with real numbers when you have them, estimates when
you do not (it marks which is which). You get `diagnostics.md`: a loss
table in dollars where every figure shows its math.

## 3. Design (stage 3)

> Run kineti-design

Name three products whose look you like. It writes a style brief for your
approval, then generates real HTML variants side by side. Pick, reject,
iterate. Your picks are remembered for next time.

## 4. Architecture (stage 4)

> Run kineti-architecture

It draws services and arrows, gives every arrow an error path, builds the
failure table, and shows stack math comparing Supabase/Vercel-style blocks
against alternatives — including self-hosting.

## 5. The gate (stage 5)

> Run kineti-feasibility

Three checks: money (margin, return vs hurdle), data (quality ≥ 0.8 on
required fields), people (who can kill this, and what artifact wins them).
A fail routes back to stage 2 with the exact number that failed. Try it on
a deliberately bad idea once — watching it die cheaply is the lesson.

## 6. Spec and the hard stop (stage 6)

> Run kineti-spec

Typed shapes, pass/fail acceptance criteria, out-of-scope list. Then the
run stops until you explicitly approve. Nothing under `src/` exists yet.

## 7. Build → Review → QA → Security → Ship (stages 7–11)

> Approve the spec. Run kineti-build.

Watch `.kineti/saga.jsonl` grow one undo step per change, and progress
commits carry decisions plus failed approaches. Then run review, qa
(real browser, three widths, regression test per bug), security, and ship.
Ship refuses stale proofs — edit a file after qa and see it block.

## 8. Watch and retro (stages 12–13)

After deploy: baseline, poll, alerts land in `~/.kineti/alerts.log`.
Weekly: `Run kineti-retro` turns the week into dated lessons that stages
1–5 quote automatically next time.

That is the whole loop. Everything else — memory jobs, audits, second
opinions — serves these thirteen stages.
