---
name: executing-plans
description: Use when you have a written implementation plan to execute in a separate session with review checkpoints. Triggers are "execute plan", "let's implement the plan", "implement plan".
---

# Executing Plans

## Overview

Load plan, review critically, execute all tasks with review checkpoints, verify end-to-end, report when complete.

**Announce at start:** "I'm using the executing-plans skill to implement this plan."

**Two tracking surfaces, both required:** the plan document is the durable record — checkboxes, deviation notes, and the completion summary live there, and it survives across sessions. The session task list (TaskCreate/TaskUpdate) is how your partner watches progress live in the session — create it at the start and keep statuses current throughout; never skip it. If the two ever disagree, the plan document wins.

## The Process

### Step 1: Load and Review Plan
1. Read plan file
2. Review critically — identify any questions or concerns about the plan itself (gaps, contradictions, ambiguity)
3. **Check for staleness** — the plan may have been written days ago in a different repo state. Verify its factual claims against current reality: the files it says to modify exist where it says, pinned signatures/shapes still match the code, commands it specifies still run. A plan that was right when written can be wrong now.
4. If concerns: raise them with your human partner before starting
5. If no concerns: create tasks with TaskCreate (mirroring the plan's tasks) and proceed

### Step 2: Execute Tasks

For each task:
1. Mark the task as `in_progress` with TaskUpdate
2. Follow each step exactly (plan has bite-sized steps)
3. Run verifications as specified, commit as the task directs
4. Kick off the `review-with-codex` skill — it runs codex CLI against the just-committed work for a second-opinion review. Wait for it to finish before moving on (blocking checkpoint).
5. Address any **must fix** findings codex returns as a fixup commit. If the changes are non-trivial, re-run `review-with-codex` to confirm. Advisory findings: note and move on.
6. Mark the task as `completed` with TaskUpdate and check off its steps in the plan document

### Deviations: What You May Decide vs. When to Stop

Plans meet reality; small mismatches are normal and are yours to handle:

- **Decide yourself** when the deviation preserves the design's intent: the right file turns out to be a sibling of the one named, a pinned signature needs one extra parameter, a command needs a flag the plan didn't know about. Make the call, then **record it as a one-line note in the plan document under that task** (`> Deviation: ...` — what changed and why).
- **Stop and ask** when the deviation would change the approved design's shape: a different approach, a new dependency, a changed interface other tasks rely on, scope growth.

Deviation notes surface in the completion summary, so your partner can veto after the fact at a glance — same contract as fastplan's "decide and surface" rule, mirrored to execution.

## When to Stop and Ask for Help

**STOP executing immediately when:**
- Hit a blocker (missing dependency, test fails, instruction unclear)
- Plan has critical gaps preventing starting
- You don't understand an instruction
- Verification fails repeatedly
- A deviation would change the approved design (see above)

**Ask for clarification rather than guessing.**

## When to Revisit Earlier Steps

**Return to Review (Step 1) when:**
- Partner updates the plan based on your feedback
- Fundamental approach needs rethinking

**Don't force through blockers** — stop and ask.

## Step 3: Final Verification and Completion

Per-task tests passing is not the same as the feature working. Before declaring the plan done:

1. Run the **full test suite**, not just the tests the plan touched
2. **Exercise the built feature once end-to-end** — actually drive it the way a user would (use the `verify` skill if available). Task-level green plus integration-level broken is the classic failure mode this step exists to catch
3. Then, in the plan document:
   - Mark the whole plan as completed
   - Write a short, concise summary at the end: what was implemented, any issues encountered, and all deviation notes gathered in one place
   - Add one line: **what the plan could have specified better** (or "nothing" if it held up) — this feeds back into how future plans get written

## Remember
- Review plan critically first — including whether it's stale
- Follow plan steps exactly; small intent-preserving deviations are yours, design-shape changes are not
- Every task ends with a blocking codex review — address must-fix findings before starting the next task
- Don't skip verifications; finish with the full suite and an end-to-end pass
- Reference skills when plan says to
- Stop when blocked, don't guess
- Keep both tracking surfaces current: TaskUpdate for the live session view, the plan document as the durable record
