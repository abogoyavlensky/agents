---
name: shape
description: Turn a batch of changes — several improvements, fixes or small features, often from different areas of an app — into one short "shape" doc in docs/shapes/ that pins what must be true for the user and the rules the work must follow, without planning the steps. A shape reads like one phase of a product roadmap — items as checkboxes, UX principles, settled decisions, non-goals, exit criteria — and is implemented in a single run and a single PR by the `impl` skill. Use this whenever the user says "shape", "shape this", "shape it up", "write a shape", "make a shape doc", "turn this discussion into a shape", or wants a batch of agreed changes written down with strong UX expectations but without a detailed implementation plan — especially right after a discuss session has converged. Prefer fastplan when the user wants step-by-step tasks, and makegoal when the work is a single end state for an autonomous /goal session.
---

# Shape: a Batch of Changes, Pinned Without Planning the Route

A shape is one page that says what will be true for the user when a batch of changes ships, and which rules the work must not break. It deliberately leaves the route to the implementer. It is modelled on a phase of a product roadmap — a short list of behaviours, each with the one rule that matters — because that format, backed by the project's own invariants, has proven enough for an agent to build a multi-area batch in one run.

**Announce at start:** "I'm using the shape skill to write this up as a shape."

<HARD-GATE>
Write the shape document and nothing else: no code, no scaffolding, no commit, no branch. The shape is read by the user before anything is built; `impl` commits it as the first commit of its branch. Do not write file-by-file steps — if you find yourself writing "first change X, then Y", that is a plan, and it belongs to the implementer.
</HARD-GATE>

## Why the format works

A shape stays short because the project already carries most of the weight: its agent instructions (AGENTS.md, CLAUDE.md), its product rules or roadmap, its design source, and its check commands. The shape only holds the **delta** — what this batch changes and the expectations that are not written anywhere else yet.

The one thing a shape must carry more of than a roadmap phase is **UX expectation**. Roadmap phases are usually backed by designs; ongoing batches mostly add interactions nobody has drawn. So for each new interaction, say the gesture, the feedback, who can see it, and what happens on failure or when empty. For anything that saves or loads asynchronously, also say what happens while it is in flight: can the user leave, dismiss or keep typing, and what becomes of their input. Leave visuals to "match the existing screens".

## The process

1. **Gather the intent.** A shape usually follows a discussion. Pull every agreed item, decision, rejected option and open question from the conversation. If the user hands you a fresh list instead, treat each line as an item to ground.

2. **Ground every item in the repo.** Read the project's instructions and product docs (and `docs/KNOWLEDGE.md` if it exists), then the code each item touches, enough to state facts rather than guesses: what exists today, what is missing (an endpoint, a column, a component), and which product rules apply (permissions, plans, limits). Note any claim from the discussion that the code contradicts. Use `git rev-parse --short HEAD` for the base commit.

3. **Find out how this project proves work is done.** This goes into Exit criteria, so `impl` never has to guess. Look in, roughly in this order of authority:
   - CI workflows (`.github/workflows/`, `.gitlab-ci.yml`, …) — what CI runs is the real gate;
   - agent instructions (AGENTS.md, CLAUDE.md) and the README;
   - the task runner (`rite`, `make`, `just`, `task`, `bb`, `mise` tasks, `package.json` scripts).

   Record: the **check command** (lint + test + build, whatever CI runs), the **end-to-end harness** if there is one (command, where specs live, where screenshots land, what to compare them with), the **test conventions** next to the code the items touch (unit tests beside the file? component tests? store tests against a real DB?), and the **checks only a human can do** (a real device, a third-party client, a live deploy, a payment). If there is no e2e harness and items are user-visible, say how the agent should drive the app instead (the `run` skill, a browser preview, a CLI invocation) and flag the missing harness.

4. **Decide what's yours and ask only what isn't.** Settle reversible, inferable calls yourself and record them under Decisions or Open. Ask the user only when the answer changes user-visible behaviour, scope, a public API or data shape, or how done is verified — and batch those into one `AskUserQuestion`. If the discussion already settled everything, ask nothing.

5. **Size it for one PR.** A shape ships as exactly one PR. If the batch would make a PR too large to review well, split it into two shapes along a natural seam and tell the user which items went where. Do not quietly drop items.

6. **Write the doc** at `docs/shapes/YYYY-MM-DD-HHMM-<topic>.md`, following `references/shape-template.md` (in this skill's directory). Get the prefix with `date +%Y-%m-%d-%H%M` — never guess the time. Create `docs/shapes/` if it is missing. Apply `/writing-clearly` if available.

7. **Hand back.** One short message: the path, the items in one line each, the decisions you made yourself that the user may want to veto, and anything still open. Then offer `/impl <path>` as the next step. Do not start it.

## Writing the items

Items are the heart of the shape. Each one is a numbered checkbox (`1. [ ] **…**`) that the implementer ticks and then rewrites to say what was built. The numbers are for tracking: `impl` names checkpoints and commits after them and keeps a count in the status line (`in progress · 3/7`). Number in a sensible build order where one exists, foundations first, but don't contort the list for it.

- **One user-visible behaviour per item**, in bold, then the rule that matters: who may do it, what it affects, what must hold. "**Reorder categories.** Long-press a tile on Categories and drag it; a tap still selects. Order is per budget and kind, admin only, and decides which tiles Add shows first."
- **At most about four lines.** An item that needs more is not decided yet: go back to the discussion or split it.
- **State facts the implementer needs**, not steps: "budgets have no colour or icon yet", "the `position` column already orders categories". Name a file or symbol only when a decision hinges on it.
- **Say who sees it.** Permission rules are the most common thing an implementer gets wrong when they are left implicit.

The whole shape should fit on about one page. That limit is what keeps it from turning back into a plan.

## The other sections, briefly

- **Why** — two or three sentences on what is wrong for the user today. Gives the implementer the intent to judge cases nobody foresaw.
- **UX principles** — cross-cutting expectations that apply to several items: feedback (haptics, toasts), failure behaviour, "a section before a new screen", accessibility, translations, consistency with an existing pattern.
- **Decisions** — settled calls, one line of *why* each, so the implementer does not relitigate them and can apply the reasoning to new cases.
- **Non-goals** — the fence. Put rejected options from the discussion here with their reason; they are the things an implementer is most tempted to "helpfully" add.
- **Open** — small choices the implementer makes and records (an icon's exact shape, a label). Anything that would change behaviour is not Open — decide it or ask.
- **Exit criteria** — from step 3: exact commands, the e2e expectations, which new flows need specs, and the owner-only checks.
- **Result** — left empty with a placeholder; `impl` fills it.

## Key principles

- **The destination, not the route.** Behaviours and rules, never file-by-file steps.
- **Strong UX expectations where nothing is drawn.** Gesture, feedback, permission, failure.
- **Grounded, not remembered.** Every fact in the shape was checked against the code.
- **One page, one PR.** Split rather than sprawl.
- **Verification is discovered, not assumed.** Each project proves work differently; write down how this one does.
