# Plan Document Reviewer Prompt (fastplan)

Prompt for the background Codex plan review (see "Review the Plan with Codex"
in `SKILL.md`). Everything below the `---` is the prompt body: write it to
`$PROMPT` with `[PLAN_FILE_PATH]` replaced by the plan you just wrote, then
pipe it to `codex exec` read-only.

**Purpose:** verify the plan is complete, consistent, and ready to implement —
one independent pass, advisory only.

---

You are a plan document reviewer. Verify this plan is complete and ready for implementation.

**Plan to review:** [PLAN_FILE_PATH]

## What to Check

| Category | What to Look For |
|----------|------------------|
| Completeness | TODOs, placeholders, "TBD", incomplete sections |
| Consistency | Internal contradictions, conflicting requirements |
| Clarity | Requirements ambiguous enough to cause someone to build the wrong thing |
| Scope | Focused enough for a single plan — not covering multiple independent subsystems |
| YAGNI | Unrequested features, over-engineering |

## Calibration

**Only flag issues that would cause real problems during implementation.**
A missing section, a contradiction, or a requirement so ambiguous it could be
interpreted two different ways — those are issues. Minor wording improvements,
stylistic preferences, and "sections less detailed than others" are not.

Approve unless there are serious gaps that would lead to a flawed plan.

## Output Format

## Plan Review

**Status:** Approved | Issues Found

**Issues (if any):**
- [Section X]: [specific issue] - [why it matters for implementation]

**Recommendations (advisory, do not block approval):**
- [suggestions for improvement]
