---
name: fastplan
description: Fast planning skill for small-to-medium coding work — features, components, behavior changes — when you want a solid plan without the full brainstorming back-and-forth. Explores the codebase, makes the low-stakes design calls itself (optimizing for correctness, then tradeoffs, then UX), surfaces only genuinely pivotal decisions, presents the whole design at once for a single round of approval, then writes a plan to docs/plans/ and hands off to executing-plans. Use this whenever the user wants to "plan fast", "quick plan", "fast plan", "just plan it", or asks for a plan and signals they want speed and fewer questions. Prefer brainstorming instead when the work is large, exploratory, or the user wants to think it through together step by step.
---

# Fastplan: Ideas Into Plans, Fast

Turn an idea into an implementation-ready plan with minimal back-and-forth. The deal: you do the thinking, make the low-stakes design calls yourself, and bring the user in only for the decisions that genuinely need them. The goal is to be *faster*, not *dumber* — use the freedom to apply good judgment, not to skip it.

This is the express lane to the same destination as brainstorming: the same plan format, the same `docs/plans/` location, the same handoff to executing-plans. What's cut is the ceremony — questions one at a time, section-by-section approval gates, and the plan review loop. What's kept is rigor in the design and the plan.

<HARD-GATE>
Do NOT write code, scaffold, or invoke any implementation skill until the user has approved the design AND the plan document is written. Speed comes from deciding well and presenting once — never from skipping the design or starting to build before it's agreed.
</HARD-GATE>

## The Process

1. **Explore project context** — read the relevant files, docs, and recent commits; learn the existing patterns before proposing anything. This step is not abbreviated: good autonomous decisions depend on actually knowing the codebase. If the request turns out to be large or spans several independent subsystems, say so and recommend brainstorming instead — fastplan's speed assumes a focused, small-to-medium task.

2. **Decide what's yours vs. what's theirs** — make the low-stakes calls yourself; batch only the pivotal ones for the user. When in doubt, decide it yourself and surface it in the design for veto. (details below)

3. **Present the whole design at once** — one message, the complete shape of the solution and the key decisions behind it. Not a detailed task list yet. (details below)

4. **Approval gate** — ask whether the design looks right or any key decision needs adjusting. Iterate quickly if so. This is the main place the user steers.

5. **Write the plan** — once the design is agreed, write the full detailed plan to `docs/plans/YYYY-MM-DD-<topic>.md` following `../brainstorming/plan-format-guide.md`.

6. **Hand off** — offer a final light edit pass, then commit the plan and transition to executing-plans.

## Decide What's Yours vs. What's Theirs

The core of fastplan is exercising judgment instead of deferring every fork to the user. In a small-to-medium task, most decisions are yours to make.

**Decide yourself** (and note the call in the design, don't ask): naming; where files go within an established structure; which library when one is clearly the better fit or already in the project; error-handling and logging conventions that match the codebase; the test framework already in use; which of two roughly-equivalent approaches to take. Resolve these by, in order: **correctness** (does it work and hold up), **tradeoffs** (simplicity, maintainability, performance), then **UX** (the experience for whoever uses the result). Pick the best and move on.

**Ask the user** (batch into a single `AskUserQuestion` prompt — never one at a time): decisions that reshape the whole design and are expensive to undo, or that hinge on product intent or preference you cannot infer from the codebase. Things like a fundamental architecture fork (CLI vs. long-running service), a data model that ripples through everything, a user-facing behavior with a real product tradeoff, or an ambiguous scope boundary. If several such decisions exist, ask them together in one batched prompt, collect every answer, then think.

When you're unsure whether something is pivotal, lean toward deciding it yourself but make it prominent in the design's key decisions so the user can veto at the approval gate. A surfaced-but-reversible decision costs the user one glance; a premature question costs a round trip.

## Present the Whole Design at Once

Present the complete design in a single message — the key points that shape the solution, scaled to the work's complexity. This mirrors brainstorming's design content, just delivered all at once instead of section by section.

Cover, as relevant:
- **Approach** — the overall shape of the solution and why this one.
- **Key decisions** — the calls you made, each with a one-line rationale or tradeoff. Make the ones you'd most want a second opinion on easy to spot.
- **Components & structure** — the main pieces, their responsibilities, and how they fit. Favor small, well-bounded units with clear interfaces.
- **Data flow, error handling, testing** — only where they carry real weight for this task.

Keep it tight. This is the design, not the task list — enough for the user to recognize what's being built and catch anything they'd do differently, not a step-by-step.

## Approval Gate

End the design with a single, clear ask: *does this look right, or do you want to adjust any of the key decisions before I write the plan?*

If the user wants changes, fold them in and re-present only what changed — no need to repeat the whole design. Once they're on board, move to writing the plan. This is the one real checkpoint; don't gate every section.

## Write the Plan

After the design is agreed, write the full implementation plan to `docs/plans/YYYY-MM-DD-<topic>.md`.

- Follow `../brainstorming/plan-format-guide.md` for structure: the standard header, then `## Design`, `## File Structure`, and bite-sized `### Task N:` sections with checkbox (`- [ ]`) steps that executing-plans can mark off.
- The plan must be self-contained — fold the approved design into it so the executor has full context without the chat history.
- Map the file structure first, then break the work into small tasks (test, implement, verify, commit). Exact paths, exact commands, expected outputs. Describe what to build clearly; don't inline full code.
- Use /writing-clearly if available. DRY, YAGNI, frequent commits.

There is no plan review loop — the design was already agreed, and executing-plans reviews the plan critically before it runs. Skipping the loop is a deliberate part of being fast.

## Hand Off

Once the plan is written:

**"Plan written to `docs/plans/...`. Want to tweak anything in it, or should I commit it and start implementation with executing-plans?"**

- **If they want edits:** apply them, then re-offer.
- **If proceed:** commit the plan document with a clear one-line message and no attribution, then invoke /executing-plans to implement it.
- **If not now:** commit the plan and stop, leaving execution for later.

## Key Principles

- **Faster, not dumber** — use decision freedom to apply judgment, not skip it.
- **Decide low-stakes calls yourself** — correctness, then tradeoffs, then UX.
- **Batch the pivotal questions** — one prompt, all at once, never a drip of one-at-a-time questions.
- **One design, presented whole** — the complete picture in a single message.
- **One real gate** — approve the design, then go; no section-by-section ceremony, no review loop.
- **Same output as brainstorming** — same plan format, same `docs/plans/` location, same executing-plans handoff.
