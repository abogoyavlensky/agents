---
name: fastplan
description: Fast planning skill for small-to-medium coding work — features, components, behavior changes — when you want a solid plan without the full brainstorming back-and-forth. Explores the codebase, makes the low-stakes design calls itself (optimizing for correctness, then tradeoffs, then UX), surfaces only genuinely pivotal decisions, writes the plan to docs/plans/ and has panel-review check it, then presents the whole design at once for a single round of approval and hands off to executing-plans. Use this whenever the user wants to "plan fast", "quick plan", "fast plan", "just plan it", or asks for a plan and signals they want speed and fewer questions. Prefer brainstorming instead when the work is large, exploratory, or the user wants to think it through together step by step.
---

# Fastplan: Ideas Into Plans, Fast

Turn an idea into an implementation-ready plan with minimal back-and-forth. The deal: you do the thinking, make the low-stakes design calls yourself, and bring the user in only for the decisions that genuinely need them. The goal is to be *faster*, not *dumber* — use the freedom to apply good judgment, not to skip it.

This is the express lane to the same destination as brainstorming: the same plan format, the same `docs/plans/` location, the same handoff to executing-plans. What's cut is the ceremony — questions one at a time, section-by-section approval gates, and the interactive plan-review loop (replaced by an advisory panel-review pass). What's kept is rigor in the design and the plan.

<HARD-GATE>
Do NOT write code, scaffold, or invoke any implementation skill until the user has approved the design at the single approval gate. Writing the plan *document* before that gate is expected — it is a document, not code. Speed comes from deciding well and presenting once — never from skipping the design or starting to build before it's agreed.
</HARD-GATE>

## The Process

1. **Explore project context** — read the relevant files, docs, and recent commits; learn the existing patterns before proposing anything. This step is not abbreviated: good autonomous decisions depend on actually knowing the codebase. If the request turns out to be large or spans several independent subsystems, say so and recommend brainstorming instead — fastplan's speed assumes a focused, small-to-medium task.

2. **Decide what's yours vs. what's theirs** — make the low-stakes calls yourself; batch only the pivotal ones for the user. When in doubt, decide it yourself and surface it in the design for veto. (details below)

3. **Write the plan** — without asking first, write the full detailed plan to `docs/plans/YYYY-MM-DD-HHMM-<topic>.md` following `plan-format-guide.md` (in this skill dir). Post a one-line progress note before you start ("Exploration done — writing the plan and running the plan review") so the user knows why the design hasn't appeared yet.

4. **Review the plan with panel-review** — the moment the plan file exists, run the panel-review skill in plan mode on it. Fold the gating findings into the plan; if the fold was substantial, re-run. Max 3 review rounds. Advisory, not a gate. (details below)

5. **Present the whole design at once** — one message, the complete shape of the solution and the key decisions behind it, plus one line on what the plan review flagged and what you folded, plus the plan path. Not a detailed task list. (details below)

6. **Approval gate** — the single checkpoint: ask whether the design looks right or any key decision needs adjusting, or whether to execute now. This is the main place the user steers. (details below)

7. **Hand off** — commit the plan and transition to executing-plans, or commit and stop.

## Decide What's Yours vs. What's Theirs

The core of fastplan is exercising judgment instead of deferring every fork to the user. In a small-to-medium task, most decisions are yours to make.

**Decide yourself** (and note the call in the design, don't ask): naming; where files go within an established structure; which library when one is clearly the better fit or already in the project; error-handling and logging conventions that match the codebase; the test framework already in use; which of two roughly-equivalent approaches to take. Resolve these by, in order: **correctness** (does it work and hold up), **tradeoffs** (simplicity, maintainability, performance), then **UX** (the experience for whoever uses the result). Pick the best and move on.

**Ask the user** (batch into a single `AskUserQuestion` prompt — never one at a time): decisions that reshape the whole design and are expensive to undo, or that hinge on product intent or preference you cannot infer from the codebase. Things like a fundamental architecture fork (CLI vs. long-running service), a data model that ripples through everything, a user-facing behavior with a real product tradeoff, or an ambiguous scope boundary. If several such decisions exist, ask them together in one batched prompt, collect every answer, then think. This batch is the only touchpoint before the plan is written — don't pad it to compensate; a surfaced-but-reversible decision costs the user one glance at the approval gate.

When you're unsure whether something is pivotal, lean toward deciding it yourself but make it prominent in the design's key decisions so the user can veto at the approval gate. A surfaced-but-reversible decision costs the user one glance; a premature question costs a round trip.

## Write the Plan

Once the pivotal questions (if any) are answered, write the full implementation plan to `docs/plans/YYYY-MM-DD-HHMM-<topic>.md` — do not ask for approval first; the design gets its single review after the plan exists and panel-review has read it. Get the prefix with `date +%Y-%m-%d-%H%M` — don't guess the time. The HHMM part keeps several same-day plans sorted in creation order, which a date alone can't do.

- Follow `plan-format-guide.md` (in this skill dir) for structure: the standard header, then `## Design`, `## File Structure`, and bite-sized `### Task N:` sections with checkbox (`- [ ]`) steps that executing-plans can mark off.
- The plan must be self-contained — fold the design into it so the executor has full context without the chat history.
- Map the file structure first, then break the work into small tasks (test, implement, verify, commit). Exact paths, exact commands, expected outputs. Describe what to build clearly; inline small code fragments only where tasks must agree exactly (shared signatures, data shapes) or a description would be ambiguous (tricky logic) — never full implementations.
- Use /writing-clearly if available. DRY, YAGNI, frequent commits.

There is no interactive plan-review *loop*. Instead, the panel-review pass (next) gives the plan an independent read before the user sees the design, and executing-plans reviews it again critically before it runs. Advisory passes, not a back-and-forth — that's what keeps it fast.

## Review the Plan with panel-review

An independent read of the finished plan catches gaps, contradictions, plan claims that don't match the repo, and over-engineering — cheap to fix now, expensive to hit mid-implementation. This is advisory, not a new approval loop.

The moment the plan file is written, run the **panel-review** skill in **plan** mode on it, as a calling skill (it returns a short gating list rather than asking the user anything). Its finders — a Claude subagent and codex, both carrying the plan and grounding lenses — run in the background, and a verifier re-checks each finding against the plan and the repo before it reaches you.

When it returns, fold the gating findings (critical / major) into the plan document, plus any minor that is clearly right and cheap. Skip the rest. If the fold changed the plan substantially, run another panel-review round on the updated plan. Stop after 3 rounds total, at panel-review's early-stop rule, or as soon as a round comes back with nothing gating. Only then present the design.

If codex is missing from the round, the Claude finder's review still counts — mention it in the review line. If every finder failed, tell the user at the approval gate and continue without the review — it's advisory, never a blocker.

## Present the Whole Design at Once

Present the complete design in a single message — the key points that shape the solution, scaled to the work's complexity. This mirrors brainstorming's design content, just delivered all at once instead of section by section. By this point the plan is written and reviewed, but the message is still the *design*, not the task list.

Cover, as relevant:
- **Approach** — the overall shape of the solution and why this one.
- **Key decisions** — the calls you made, each with a one-line rationale or tradeoff. Make the ones you'd most want a second opinion on easy to spot.
- **Components & structure** — the main pieces, their responsibilities, and how they fit. Favor small, well-bounded units with clear interfaces.
- **Data flow, error handling, testing** — only where they carry real weight for this task.
- **Plan review** — one line: what panel-review flagged (or "nothing significant"), what you folded in, what you skipped and why. If the review failed or is still running, say so.
- **Plan path** — where the plan lives in `docs/plans/`.

Keep it tight. This is the design, not the task list — enough for the user to recognize what's being built and catch anything they'd do differently, not a step-by-step.

## Approval Gate

End the design with a single, clear ask: *does this look right — want to adjust any of the key decisions, or should I commit the plan and start implementation with executing-plans?*

This is the one real checkpoint; don't gate every section, and don't add a second "ready to execute?" ask afterward.

- **If they want changes:** fold them into the plan document, run one more panel-review round on the updated plan (still within the 3-round max), then re-present only what changed — no need to repeat the whole design. Re-ask the same single question.
- **If proceed:** commit the plan document with a clear one-line message and no attribution, then invoke /executing-plans to implement it.
- **If not now:** commit the plan and stop, leaving execution for later.
- **If they reject the direction entirely:** delete the uncommitted plan file rather than leaving an orphan in `docs/plans/`, then either re-plan from the new direction or stop, as they prefer.

## Key Principles

- **Faster, not dumber** — use decision freedom to apply judgment, not skip it.
- **Decide low-stakes calls yourself** — correctness, then tradeoffs, then UX.
- **Batch the pivotal questions** — one prompt, all at once, never a drip of one-at-a-time questions.
- **Plan first, then present** — the plan is written and reviewed before the user sees the design, so approval and "go" are the same moment.
- **One design, presented whole** — the complete picture in a single message.
- **One real gate** — approve the design and execute in one answer; no section-by-section ceremony, no interactive review loop, no separate "commit and run?" ask. The panel-review pass is advisory, not a gate.
- **Same output as brainstorming** — same plan format, same `docs/plans/` location, same executing-plans handoff.
