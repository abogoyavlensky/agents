---
name: makegoal
description: Turn a rough feature or task idea into an autonomous `/goal` brief — a concise goal *contract* (target state, fixed decisions, assumptions, non-goals, acceptance criteria, verification, review + escalation policy) saved to docs/goals/<YYYY>-<MM>-<DD>-<topic>.md, plus a minimal `/goal` launcher condition that points at it. Use this whenever the user knows the target *end state* more than the exact steps and wants a goal/spec/contract instead of a step-by-step plan. Triggers include "makegoal", "make a goal", "turn this into a goal", "write/draft a goal", "goal prompt", "goal contract", "/goal", or "I know what the end state should be, not the steps". Built around Claude Code's `/goal` (a completion condition re-checked after every turn): makegoal writes the brief the main model reads and the condition the evaluator checks. Unlike fastplan and brainstorming (which produce step-by-step plans in docs/plans/), makegoal deliberately does NOT plan the implementation — it pins the destination and the guardrails and lets the executing agent choose the route. Prefer makegoal when handing work to an autonomous `/goal` session; prefer fastplan/brainstorming when you want the steps worked out with you.
---

# makegoal — idea → goal contract + `/goal` launcher

Turn a rough idea into two things: a concise **goal contract** in `docs/goals/`, and a
minimal **`/goal` launcher** that points at it. You specify the destination and the
guardrails; an autonomous `/goal` session chooses the route and works until the goal is met.

This is the tool for when you know the *end state* better than the steps. It's the
counterpart to fastplan/brainstorming, not a replacement: those work the implementation
*out with you* and emit a step-by-step plan in `docs/plans/`. makegoal emits a *contract* and
a launcher, and hands execution to a fresh session.

**Announce at start:** "I'm using the makegoal skill to turn this into a `/goal` contract."

<HARD-GATE>
makegoal authors a contract. It does NOT implement, scaffold, or write code, and it does NOT
produce a low-level, step-by-step implementation plan — prescribing the route is the executing
agent's job, and over-specifying it here defeats the purpose. Stay read-only: explore to
understand, then write the contract. If the user wants the steps worked out, point them at
fastplan or brainstorming instead.
</HARD-GATE>

## How this rides Claude Code's `/goal`

`/goal <condition>` (Claude Code v2.1.139+) sets a *completion condition* for the session and
starts working immediately — the condition is the directive, no separate prompt needed. After
each turn a small fast model checks the condition against **the conversation so far** — it does
not run commands or read files — and if it isn't met, another turn auto-starts. The goal clears
when the condition holds. It needs a trusted workspace with hooks enabled.

That "evaluator only sees the transcript" fact shapes the launcher:

- **State the check, not just the file.** The evaluator can't open the doc, so the condition
  must be demonstrable from what Claude surfaces: name the exact commands whose passing output
  will land in the conversation, and require each acceptance criterion to be confirmed there.
  The *main* model reads the doc at turn 1 for the full brief; the *evaluator* judges only the
  result.
- **Bound the loop.** Include a stop clause ("or stop after N turns") so it can't spin.
- **Make "blocked" terminal.** Phrase a genuine blocker — a question raised to the user — as a
  satisfying end state, so the loop hands control back instead of looping past it.

So makegoal splits the work: a tight, evaluator-facing **condition** (the launcher) and the
full **brief** the main model reads (the `docs/goals/…` file). The contract's sections are just
a structured way to write what the official docs call an effective condition — one measurable
end state (Target state + Acceptance criteria), a stated check (Verification), and constraints
that matter (Non-goals + Fixed decisions).

## The process

1. **Explore the project** — enough to write a contract that's *true*, not generic.
2. **Decide: ask, decide, or assume** — resolve only the genuinely blocking forks; batch them
   into one question. Otherwise proceed and record assumptions.
3. **Draft the contract** — target state + the guardrail sections, kept concise.
4. **Write `docs/goals/<date>-<topic>.md` and the launcher** — present both in chat.
5. **Hand off** — stop. Offer to run the launcher; don't auto-implement.

## 1. Explore the project

A goal contract is only useful if its verification commands are real, its non-goals match the
actual seams of the codebase, and its "ask me if the public API is ambiguous" rule points at an
API that exists. So explore first — this is not optional filler:

- **Verification surface.** How does this project prove things work? Look for a task runner or
  scripts (`Makefile`, `justfile`, `bb.edn`, `package.json` scripts, `Cargo.toml`, etc.), CI
  config, and any "verification"/"testing" section in `README`, `CLAUDE.md`, or `AGENTS.md`.
  Capture the exact commands (e.g. `bb check`, `bb e2e`, `npm test`, `pytest -q`). Note whether
  an **end-to-end / integration** harness exists — if one does, the goal should use it; if none
  does and the feature is user-visible, the contract should say so and suggest a smoke check.
- **The relevant surface.** Read the files the idea touches. What's the public API / config
  shape / user-visible behavior near it? Where are the module boundaries that define a clean
  non-goal? What existing behavior could this conflict with? Gather exactly enough to tell a
  *blocking* ambiguity from a detail the agent can settle itself.
- **Conventions.** Naming, error handling, test layout, where docs live — so the contract
  inherits the house style instead of imposing a new one.

Stay read-only. You're gathering the commands and constraints to put *in* the contract, not
running the build or implementing anything.

## 2. Decide: ask, decide, or assume

Default to momentum. Most forks in a focused task are yours to settle — settle them and record
the call. Reserve the user's attention for decisions expensive to reverse or that you genuinely
cannot infer.

**Decide yourself, silently or as a Fixed Decision** — anything reversible or inferable:
internal naming and structure, file placement within an established layout, which existing
library/util to reuse, conventions that match the codebase, the test framework already in use,
which of two roughly-equivalent internal approaches to take. Resolve by, in order:
**correctness** → **tradeoffs** (simplicity, maintainability, performance) → **UX**. Settled,
load-bearing calls go in **Fixed Decisions**; reasonable-default calls you'd revisit if reality
differs go in **Assumptions**.

**Ask the user** — only when the answer would materially change one of: the **public API or
config shape**, **backwards compatibility**, **user-visible behavior/semantics**, a **scope
boundary**, or the **verification strategy**. If several such questions exist, batch them into
**one** `AskUserQuestion` — never drip them one at a time — collect the answers, then draft.
Each answer becomes a Fixed Decision.

When unsure whether a fork is blocking, lean toward deciding it and surfacing it prominently in
the contract (a Fixed Decision or Assumption the user can veto at a glance) rather than spending
a question. A surfaced-but-reversible call costs one glance; a premature question costs a round
trip. If there are no blocking questions, say so and proceed — don't manufacture them.

## 3. Draft the contract

Sections around a target state. Keep each tight — this is a contract, not an essay; lean prose
helps the main model act and keeps the launcher honest. Apply the **/writing-clearly** skill if
available.

- **Target state** — the end state in *observable* terms: what is true when this is done that
  isn't now. Outcomes, not steps. 2–4 sentences. If you're writing "first do X, then Y," stop —
  that's a plan, and it belongs to the executing agent.
- **Fixed decisions** — settled, load-bearing calls the agent must honor and not relitigate
  (chosen API shape, config keys, severity levels, behavior). One line of rationale each, so
  the agent can apply the *why* to cases you didn't foresee.
- **Assumptions** — reasonable defaults taken without certainty. The agent may proceed on them
  but must **flag any that turn out false**. Keep these visibly separate from Fixed Decisions:
  "this is settled" vs. "best guess, correct me" is the difference between confident autonomy
  and a silent wrong turn.
- **Non-goals** — what's explicitly out of scope. The anti-scope-creep fence, and often the
  most valuable section — name the adjacent things the agent should *not* build.
- **Acceptance criteria** — observable, checkable statements of done, as a `- [ ]` list. Prefer
  criteria a command or a concrete user-visible behavior can confirm. If one isn't checkable,
  sharpen it until it is.
- **Verification** — see below.
- **Review policy** and **Escalation policy** — use the defaults below unless the user differs.
- **Result** — left as a placeholder the *executing* agent fills on completion (see Result
  write-back below). makegoal seeds it empty.

### Verification defaults

The contract must make "done" provable, not asserted. Default to:

- **Deterministic commands first** — the project's own checks, by exact name (e.g. `bb check`,
  then `bb e2e`). Reuse what exists; don't invent a new test story. Name them in the launcher
  too, so the `/goal` evaluator can confirm their passing output appears in the conversation.
- **Prove the user-visible behavior** — at least one example or end-to-end check that exercises
  the new behavior the way a user/editor/caller would, not just a unit test of an internal
  function. Prefer the existing e2e harness when there is one.
- **If no e2e/integration harness exists** and the feature is user-visible, say so and suggest
  a minimal smoke check; flag it to the user, since a missing harness weakens every future goal.
- **Require evidence** — the contract instructs the agent to *show the passing output* and a
  **final diff summary** before claiming completion. "It should pass" is not evidence.

### Review and escalation defaults

Bake these in unless the user overrides:

- **Review:** after verification passes, run the **review-with-codex** skill on the diff for a
  second opinion. Address real correctness / API / edge-case / maintainability / test-coverage
  findings, and confirm the implementation is **minimal** (no unrequested scope). Ignore pure
  style nits unless they reveal a real problem. Up to **4** review/fix rounds. Per-step review
  only if the user asked for it.
- **Escalation:** decide local implementation details without asking; ask only on the
  public-API / compat / behavior-conflict / diverging-semantics forks from step 2; **stop and
  ask** when blocked (missing dependency, repeated verification failure, contradictory
  requirement) rather than guessing past a blocker.

## 4. Write the file and the launcher

Get the date with `date +%F`. Write the contract to `docs/goals/<YYYY-MM-DD>-<topic>.md`, where
`<topic>` is a short kebab slug (keeps same-day goals from overwriting and makes the file
self-describing). Create `docs/goals/` if missing, and `git add -N` the new file. Keep it
concise; apply **/writing-clearly** if available. Use the templates below.

**Result write-back.** The doc carries a `Status:` line (starts `Ready to execute`) and an empty
`## Result` section. The contract instructs the executing agent that, on achieving the goal, it
must flip Status to `✅ Achieved <date>` and fill Result with a short report: what shipped, the
evidence (commands run + key output), and any assumption that proved false or deviation. The
launcher's completion condition requires this write-back, so the goal isn't "met" until the
record is updated.

**The launcher** is a single `/goal <condition>` line that points at the doc and states a
transcript-checkable completion (see template). It's minimal by design — the doc holds the
detail; the condition holds just enough for the evaluator.

### Chat output format

Present the contract concisely in chat using these headings, then the launcher:

```
# Goal summary
# Blocking questions   (only if any remain unresolved)
# Fixed decisions
# Assumptions
# Non-goals
# Acceptance criteria
# Verification
# Review policy
# Escalation policy
# Launcher   (the minimal /goal line; also written to the doc)
```

Confirm the file path. Keep it scannable — the doc holds the full detail.

## 5. Hand off

makegoal stops at the contract. The point is a `/goal` session you start fresh, so don't slide
into implementing it here. Close with the choice:

**"Contract written to `docs/goals/<date>-<topic>.md`. Run the launcher in a fresh session to
execute it — or want me to tweak the contract, or start the `/goal` here?"**

- **Tweak:** fold in the change, rewrite the file, re-present only what changed.
- **Run it:** the user starts the `/goal` (here or in a new session); the executing agent does
  the work, verifies, runs review-with-codex, and writes the Result + Status back to the doc.
- **Not now:** leave the file; you're done.

## Output templates

**Goal doc** — `docs/goals/<YYYY-MM-DD>-<topic>.md`:

````markdown
# Goal: <Title>

> Status: Ready to execute · Created: <YYYY-MM-DD>
> Brief for an autonomous `/goal` session — read it in full first. Launch with the condition at the bottom.

## Target state
<2–4 sentences, observable end state. No steps.>

## Fixed decisions
- <settled call> — <one-line why>

## Assumptions
- <default taken> — <what to do if it's wrong>

## Non-goals
- <explicitly out of scope>

## Acceptance criteria
- [ ] <observable, checkable statement of done>

## Verification
Run these, show the passing output, and end with a final diff summary before claiming done.
- `<command>` — <what it proves>
- <example/e2e check proving the user-visible behavior>

## Review policy
- After verification passes, run the **review-with-codex** skill on the diff.
- Fix real correctness / API / edge-case / coverage findings; keep the implementation minimal.
- Ignore pure style nits unless they reveal a real problem. Up to 4 review/fix rounds.

## Escalation policy
- Decide local implementation details yourself (naming, layout, internal structure — anything reversible).
- Ask me only if: the public API/config shape is ambiguous; a backwards-incompatible change seems necessary; the goal conflicts with existing behavior; or two viable approaches differ in user-facing semantics.
- Stop and ask when blocked (missing dependency, repeated verification failure, contradictory requirement) — don't guess past a blocker.

## Result
_Filled in on completion: flip Status above to `✅ Achieved <date>`; summarize what shipped, the evidence (commands + key output), and any assumption that proved false or deviation._

## `/goal` launcher
```
<the launcher line below, verbatim>
```
````

**Launcher** — one `/goal` line, copy-paste ready:

```
/goal Implement docs/goals/<YYYY-MM-DD>-<topic>.md to its Target state. Done when: every Acceptance criterion in that file is demonstrated in this conversation, `<check 1>` and `<check 2>` are shown passing here, the review-with-codex skill has run with must-fix items resolved, and the file's Status + Result are updated. If genuinely blocked, ask me — that counts as done. Or stop after <N> turns.
```

## Key principles

- **Destination, not directions.** Specify the end state and the guardrails; leave the route to
  the executing agent. The moment you're writing ordered steps, you've left makegoal for a plan.
- **Minimal launcher, full brief in the file.** The `/goal` condition stays tight and
  evaluator-facing; the contract carries the detail the main model reads at turn 1.
- **Write for the evaluator's blind spot.** It judges only the transcript — name the checks
  whose passing output will appear, bound the loop, and make "blocked → asked" a terminal state.
- **Momentum over interrogation.** Decide reversible forks yourself; spend a question only on
  public-API / compatibility / semantics / scope / verification forks — and batch them.
- **Fixed vs assumed, always separated.** "Settled, don't relitigate" and "best guess, correct
  me" must look different, or the agent can't tell confident autonomy from a silent wrong turn.
- **Done is proven, then recorded.** Real commands, an example of the behavior, shown output, a
  final diff — then Status flips to Achieved and the Result report lands in the doc.
- **Concise.** A contract, not an essay. Apply /writing-clearly if available.
