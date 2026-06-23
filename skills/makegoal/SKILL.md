---
name: makegoal
description: Turn a rough feature or task idea into a ready-to-paste `/goal ` execution prompt — a goal *contract* (target state, fixed decisions, assumptions, non-goals, acceptance criteria, verification, review + escalation policy) that an autonomous implementation agent can run without hand-holding, saved to docs/goals/<YYYY>-<MM>-<DD>-<topic>.md. Use this whenever the user knows the target *end state* more than the exact steps and wants a goal/spec/contract instead of a step-by-step plan. Triggers include "makegoal", "make a goal", "turn this into a goal", "write/draft a goal", "goal prompt", "goal contract", "/goal prompt", or "I know what the end state should be, not the steps". Unlike fastplan and brainstorming (which produce detailed step-by-step plans in docs/plans/), makegoal deliberately does NOT plan the implementation — it pins down the destination and the guardrails and lets the executing agent choose the route. Prefer makegoal when handing work to an autonomous agent; prefer fastplan/brainstorming when you want the implementation steps worked out with you.
---

# makegoal — idea → ready-to-paste `/goal` contract

Turn a rough idea into a **goal contract**: a tight, self-contained `/goal ` prompt an
autonomous implementation session can execute end to end without checking back for
every fork. You specify the **destination and the guardrails**; the executing agent
chooses the route.

This is the tool for when you know the *end state* better than the steps. It is the
counterpart to fastplan/brainstorming, not a replacement: those work the implementation
*out with you* and emit a step-by-step plan in `docs/plans/`. makegoal emits a *contract*
in `docs/goals/` and hands you a prompt to paste into a fresh session.

**Announce at start:** "I'm using the makegoal skill to turn this into a `/goal` contract."

<HARD-GATE>
makegoal authors a contract. It does NOT implement, scaffold, or write code, and it does
NOT produce a low-level, step-by-step implementation plan — prescribing the route is the
executing agent's job and over-specifying it here defeats the purpose. Stay read-only:
explore to understand, then write the contract. If the user actually wants the steps
worked out, point them at fastplan or brainstorming instead.
</HARD-GATE>

## The process

1. **Explore the project** — enough to write a contract that's *true*, not generic.
2. **Decide: ask, decide, or assume** — resolve only the genuinely blocking forks; batch
   them into one question. Otherwise proceed and record assumptions.
3. **Draft the contract** — target state + the seven guardrail sections.
4. **Write `docs/goals/<date>-<topic>.md` and the `/goal` prompt** — present both in chat.
5. **Hand off** — stop. Offer to kick off execution; don't auto-implement.

## 1. Explore the project

A goal contract is only useful if its verification commands are real, its non-goals match
the actual seams of the codebase, and its "ask me if the public API is ambiguous" rule
points at a public API that actually exists. So explore first — this step is not optional
filler:

- **Verification surface.** How does this project prove things work? Look for a task runner
  or scripts (`Makefile`, `justfile`, `bb.edn`, `package.json` scripts, `Cargo.toml`),
  CI config, and any "verification"/"testing" section in `README`, `CLAUDE.md`, or
  `AGENTS.md`. Capture the exact commands (e.g. `bb check`, `bb e2e`, `npm test`,
  `pytest -q`). Note whether an **end-to-end / integration** harness exists — if one does,
  the goal should use it; if none does and the feature is user-visible, the contract should
  say so and suggest adding a smoke check (see Verification below).
- **The relevant surface.** Read the files the idea touches. What's the public API / config
  shape / user-visible behavior near it? Where are the module boundaries that define a clean
  non-goal? What existing behavior could this conflict with? You're gathering exactly enough
  to tell a *blocking* ambiguity from a detail the agent can settle itself.
- **Conventions.** Naming, error handling, test layout, where docs live — so the contract
  inherits the house style instead of imposing a new one.

Stay read-only. You're gathering the commands and constraints to put *in* the contract, not
running the build or implementing anything.

## 2. Decide: ask, decide, or assume

Default to momentum. Most forks in a focused task are yours to settle — settle them and
record the call. Reserve the user's attention for decisions that are expensive to reverse or
that you genuinely cannot infer.

**Decide yourself, silently or as a Fixed Decision** — anything reversible or inferable:
internal naming and structure, file placement within an established layout, which existing
library/util to reuse, error/logging conventions that match the codebase, the test framework
already in use, which of two roughly-equivalent internal approaches to take. Resolve these by,
in order: **correctness** → **tradeoffs** (simplicity, maintainability, performance) →
**UX**. Settled, load-bearing calls go in **Fixed Decisions**; reasonable-default calls you'd
revisit if reality differs go in **Assumptions**.

**Ask the user** — only when the answer would materially change one of: the **public API or
config shape**, **backwards compatibility**, **user-visible behavior/semantics**, a **scope
boundary**, or the **verification strategy**. Concretely, ask if: the public API/config shape
is ambiguous; a backwards-incompatible change seems necessary; the goal conflicts with
existing behavior; or two viable approaches differ in user-facing semantics. If several such
questions exist, batch them into **one** `AskUserQuestion` — never drip them one at a time —
collect the answers, then draft. Each answer becomes a Fixed Decision.

When unsure whether a fork is blocking, lean toward deciding it and surfacing it prominently in
the contract (a Fixed Decision or Assumption the user can veto at a glance) rather than spending
a question. A surfaced-but-reversible call costs one glance; a premature question costs a round
trip. If there are no blocking questions, say so and proceed — don't manufacture them.

## 3. Draft the contract

Seven sections around a target state. Keep each tight; this is a contract, not an essay.

- **Target state** — the end state in *observable* terms: what is true when this is done that
  isn't true now. Outcomes, not steps. 2–5 sentences. If you find yourself writing "first do
  X, then Y," stop — that's a plan, and it belongs to the executing agent.
- **Fixed decisions** — the settled, load-bearing calls the agent must honor and not
  relitigate (chosen API shape, config keys, severity levels, behavior). One line of rationale
  each, so the agent understands the *why* and can apply it to cases you didn't foresee.
- **Assumptions** — reasonable defaults taken in the absence of certainty. The agent may
  proceed on them but must **flag any that turn out false**. Keep these visibly separate from
  Fixed Decisions: the difference between "this is settled" and "this is my best guess, correct
  me" is the difference between confident autonomy and silent wrong turns.
- **Non-goals** — what's explicitly out of scope. This is the agent's anti-scope-creep fence
  and is often the most valuable section — name the adjacent things it should *not* build.
- **Acceptance criteria** — observable, checkable statements of done, as a `- [ ]` list.
  Prefer criteria a command or a concrete user-visible behavior can confirm. If a criterion
  isn't checkable, sharpen it until it is.
- **Verification** — see below.
- **Review policy** and **Escalation policy** — use the defaults below unless the user asked
  for something different.

### Verification defaults

The contract must make "done" provable, not asserted. Default to:

- **Deterministic commands first** — the project's own checks, by their exact names (e.g.
  `bb check`, then `bb e2e`). Reuse what exists; don't invent a new test story.
- **Prove the user-visible behavior** — at least one concrete example or end-to-end check that
  exercises the new behavior the way a user/editor/caller would, not just a unit test of an
  internal function. Prefer the existing e2e harness when there is one.
- **If no e2e/integration harness exists** and the feature is user-visible, say so in the
  contract and suggest adding a minimal smoke check; flag it to the user too, since a missing
  harness weakens every future goal.
- **Require evidence** — the contract instructs the agent to *show the passing output* of these
  commands and a **final diff summary** before claiming completion. "It should pass" is not
  evidence; pasted output is.

### Review and escalation defaults

Bake these into every contract unless the user overrides them:

- **Review:** after verification passes, run the **review-with-codex** skill on the diff for a
  second opinion. Address real correctness / API / edge-case / maintainability / test-coverage
  findings, and confirm the implementation is **minimal** (no unrequested scope). Ignore pure
  style nits unless they reveal a real problem. Allow up to **4** review/fix rounds. Per-step
  review only if the user explicitly asked for it.
- **Escalation:** decide local implementation details without asking; ask only on the
  public-API/compat/conflict/diverging-semantics forks from step 2; **stop and ask** when
  blocked (missing dependency, repeated verification failure, contradictory requirement) rather
  than guessing past a blocker.

## 4. Write the file and the `/goal` prompt

Get the date with `date +%F`. Write the contract to `docs/goals/<YYYY-MM-DD>-<topic>.md`,
where `<topic>` is a short kebab slug of the goal (the slug keeps same-day goals from
overwriting each other and makes the file self-describing). Create `docs/goals/` if it's
missing, and `git add -N` the new file so it shows up in status.

Use the templates in **Output templates** below. The doc ends with the `/goal` prompt — a
plain-text block that starts with `/goal ` and is self-sufficient (it inlines the operational
essentials) while naming the doc as the source of truth. Both belong in the chat reply too.

### Chat output format

Present the contract concisely in chat using these headings, then the prompt:

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
# Final `/goal` prompt   (the copy-paste block; also written to the doc)
```

Confirm the file path. Keep it scannable — the doc holds the full detail.

## 5. Hand off

makegoal stops at the contract. The whole point is a prompt you can paste into a *fresh*,
autonomous session, so don't slide into implementing it here. Close with the choice:

**"Contract written to `docs/goals/<date>-<topic>.md`. Paste the `/goal` block into a fresh
session to execute it — or want me to tweak the contract, or kick off execution here?"**

- **Tweak:** fold in the change, rewrite the file, re-present only what changed.
- **Execute now:** proceed in this session, treating the contract as the brief (it's fine to
  use executing-plans' discipline — verify, then review-with-codex — even though there's no
  task list).
- **Not now:** leave the file; you're done.

## Output templates

**Goal doc** — `docs/goals/<YYYY-MM-DD>-<topic>.md`:

```markdown
# Goal: <Title>

> Status: ready to execute · Created: <YYYY-MM-DD>
> Execute via the `/goal` prompt at the bottom — read this whole file first.

## Target state
<2–5 sentences, observable end state. No steps.>

## Fixed decisions
- <settled call> — <one-line why>

## Assumptions
- <default taken> — <why; what to do if it's wrong>

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

## `/goal` prompt
<the block below, verbatim>
```

**`/goal` prompt** — plain text, copy-paste ready (no nested code fences inside it):

```
/goal Implement <Title>.

Source of truth: docs/goals/<YYYY-MM-DD>-<topic>.md — read it first, then execute to the target state below.

TARGET STATE
<1–3 sentence summary>

ACCEPTANCE CRITERIA (all must hold)
- [ ] <…>

NON-GOALS (do not do)
- <…>

VERIFICATION — run these, paste the passing output, then give a final diff summary before claiming done:
- <command>
- <example proving the user-visible behavior>

DECIDE vs ASK
- Decide yourself: local implementation details — naming, layout, internal structure, anything reversible. Don't ask about these.
- Ask me only if: the public API/config shape is ambiguous; a backwards-incompatible change seems necessary; the goal conflicts with existing behavior; or two viable approaches differ in user-facing semantics.
- Stop and ask if blocked (missing dependency, repeated verification failure, contradictory requirement) — don't guess past a blocker.

REVIEW
- After verification passes, run the review-with-codex skill on the diff. Fix must-fix correctness/API/edge-case/coverage issues; keep the change minimal; ignore pure style nits. Up to 4 review/fix rounds.

Honor the Fixed Decisions; don't relitigate them. Proceed on the Assumptions, but flag any that prove false.
```

## Key principles

- **Destination, not directions.** Specify the end state and the guardrails; leave the route
  to the executing agent. The moment you're writing ordered steps, you've left makegoal for a
  plan.
- **Momentum over interrogation.** Decide the reversible forks yourself; spend a question only
  on public-API / compatibility / semantics / scope / verification forks — and batch them.
- **Fixed vs assumed, always separated.** "Settled, don't relitigate" and "my best guess,
  correct me" must be visibly different, or the agent can't tell confident autonomy from a
  silent wrong turn.
- **Done is proven, not asserted.** Real commands, an example of the user-visible behavior,
  shown output, a final diff. Suggest an e2e harness when one is missing.
- **Self-sufficient prompt.** The `/goal` block must stand on its own when pasted, while still
  naming the doc as the source of truth.
