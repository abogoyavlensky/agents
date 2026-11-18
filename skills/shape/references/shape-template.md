# Shape template

Copy the skeleton, then replace every comment. Keep the headings — `impl` relies on them. Delete a section only if it would be empty and it is not Items, Exit criteria or Result.

```markdown
# Shape: <short name of the batch>

**Status: shaped** · base `<short sha>` · <YYYY-MM-DD>
<!-- impl changes this to "in progress on `<branch>`", then "done: <PR URL>". -->

Origin: <where this came from: a discussion on <date>, backlog entries by path, a roadmap item>.

## Why

<!-- 2–3 sentences: what is wrong or missing for the user today. -->

## Items

- [ ] **<Behaviour>.** <The rule that matters: who, what it affects, what must hold. Facts the implementer needs. At most ~4 lines.>
- [ ] **<Behaviour>.** <…>

## UX principles

<!-- Cross-cutting expectations: feedback, failure, permissions, where things live, translations, consistency with existing patterns. Only what is not already in the project's own docs. -->
- <…>

## Decisions

<!-- Settled; do not relitigate. One line of why each. -->
- **<Decision>**: <why>.

## Non-goals

<!-- The fence, including options rejected in discussion and why. -->
- <…>

## Open

<!-- Small calls the implementer makes and records under the item. Nothing that changes behaviour. -->
- <…>

## Exit criteria

- **Check**: `<command>` passes (what CI runs).
- **Tests**: <conventions for new tests near the touched code>.
- **End to end**: <harness command, where specs live, which new flows need one, where screenshots land and what to compare them with — or, without a harness, how the agent drives the app itself>.
- **Owner checks**: <what only a human can verify, e.g. a real device or third-party client>.

## Result

_Filled in by `impl`: what shipped, deviations from this shape, owner checks still to do, follow-ups (with backlog paths), and one line on what this shape could have said better._
```

## A filled item, for register

Good — behaviour, rule, permission, facts:

> - [ ] **Edit a budget's name, colour and icon.** The admin edits them in a "This budget" section at the top of Settings, using the category colour and icon pickers. Budgets have no colour or icon yet (every badge is a hard-coded violet house); existing budgets keep that look by default. Members see the section but cannot edit it.

Too thin — no rule, no permission, nothing to verify:

> - [ ] Budget editing.

Too deep — this is a plan:

> - [ ] Add `color` and `icon` columns in migration 00006, add `UpdateBudget` to the store, add `PATCH /api/budgets/{id}` in `internal/httpapi/budgets.go`, then create `BudgetBadge.tsx` and replace the six call sites…
